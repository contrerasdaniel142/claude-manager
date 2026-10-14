use crate::git::clone_repo;
use crate::i18n;
use crate::models::{AppState, Lang, Scope};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    /// Ruta de la carpeta del skill, relativa a la raíz del repo clonado —
    /// la usamos después como identificador para importarlo.
    pub rel_path: String,
    pub name: String,
    pub description: String,
    pub extra_files: Vec<String>,
}

fn parse_skill_md(path: &Path) -> (String, String) {
    let raw = fs::read_to_string(path).unwrap_or_default();
    let (yaml, _body) = crate::fsutil::split_frontmatter(&raw);
    let fm: crate::models::SkillFrontmatter = yaml
        .as_deref()
        .and_then(|y| serde_yaml::from_str(y).ok())
        .unwrap_or_default();
    let folder_name = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    (fm.name.unwrap_or(folder_name), fm.description.unwrap_or_default())
}

fn extra_files(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != "SKILL.md" && name != ".git" {
                out.push(name);
            }
        }
    }
    out.sort();
    out
}

#[tauri::command]
pub async fn list_remote_skills(state: State<'_, AppState>, repo_url: String) -> Result<Vec<RemoteSkill>, String> {
    // `git clone` and the directory walk block for real — this must run on
    // its own thread (spawn_blocking), or Tauri's command dispatch for
    // other invocations in flight serializes behind it. See import_skill
    // for the full explanation; the symptom was two "Instalar" clicks
    // where the second didn't even show as in-progress until the first
    // finished.
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || list_remote_skills_impl(lang, &repo_url))
        .await
        .map_err(|e| e.to_string())?
}

/// Every `SKILL.md` in a cloned repo, skipping any dotted directory (.git,
/// but also tool-specific config dirs like .openclaw, .cursor, .vscode...) —
/// repos that support several AI coding tools at once often keep a
/// near-duplicate skill set per tool in one of these, which would otherwise
/// show up as duplicates. Shared by the listing and the marketplace installer
/// (which needs to locate one skill by id inside an already-cloned repo).
fn walk_skill_md(tmp: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    walkdir::WalkDir::new(tmp)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !e.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name() == "SKILL.md")
}

fn list_remote_skills_impl(lang: Lang, repo_url: &str) -> Result<Vec<RemoteSkill>, String> {
    let tmp = clone_repo(lang, repo_url)?;
    let mut out = Vec::new();
    for entry in walk_skill_md(&tmp) {
        let dir = entry.path().parent().unwrap_or(&tmp);
        let rel_path = dir.strip_prefix(&tmp).unwrap_or(dir).to_string_lossy().replace('\\', "/");
        let (name, description) = parse_skill_md(entry.path());
        out.push(RemoteSkill {
            rel_path,
            name,
            description,
            extra_files: extra_files(dir),
        });
    }
    let _ = fs::remove_dir_all(&tmp);
    out.sort_by(|a, b| a.name.cmp(&b.name));
    if out.is_empty() {
        return Err(i18n::no_skill_md_in_repo(lang));
    }
    Ok(out)
}

/// Finds the `SKILL.md` folder whose name matches `skill_id` inside an
/// already-cloned repo — used by the marketplace installer, which only gets
/// a skill id (e.g. "webapp-testing") from the search result, not its path
/// inside the repo.
pub(crate) fn find_skill_rel_path(tmp: &Path, skill_id: &str) -> Option<String> {
    for entry in walk_skill_md(tmp) {
        let dir = entry.path().parent().unwrap_or(tmp);
        if dir.file_name().map(|n| n == skill_id).unwrap_or(false) {
            return Some(dir.strip_prefix(tmp).unwrap_or(dir).to_string_lossy().replace('\\', "/"));
        }
    }
    None
}

pub(crate) fn copy_dir_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let src = entry.path();
        let dst = to.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src, &dst)?;
        } else {
            fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn import_skill(
    state: State<'_, AppState>,
    repo_url: String,
    rel_path: String,
    scope: Scope,
) -> Result<String, String> {
    // Same reasoning as list_remote_skills: this does a real `git clone`
    // plus file copies, so it has to run off the command-dispatch thread
    // for multiple "Instalar" clicks to actually run concurrently instead
    // of queuing behind each other.
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || import_skill_impl(&state, repo_url, rel_path, scope))
        .await
        .map_err(|e| e.to_string())?
}

fn import_skill_impl(
    state: &AppState,
    repo_url: String,
    rel_path: String,
    scope: Scope,
) -> Result<String, String> {
    let lang = state.lang();
    let tmp = clone_repo(lang, &repo_url)?;
    let result = install_from_tmp(state, lang, &tmp, &rel_path, scope);
    let _ = fs::remove_dir_all(&tmp);
    result
}

/// Copies the skill at `rel_path` inside an already-cloned repo (`tmp`) into
/// `scope`'s skills dir. Shared by `import_skill` (which clones by explicit
/// rel_path) and the marketplace installer (which clones by repo + skill id,
/// then locates rel_path via `find_skill_rel_path`) — neither owns `tmp`,
/// the caller is the one who cloned it and is the one who cleans it up.
pub(crate) fn install_from_tmp(
    state: &AppState,
    lang: Lang,
    tmp: &Path,
    rel_path: &str,
    scope: Scope,
) -> Result<String, String> {
    let src = tmp.join(rel_path);
    if !src.join("SKILL.md").exists() {
        return Err(i18n::skill_md_disappeared(lang, rel_path));
    }

    let folder_name = src
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "skill-importado".into());
    let dest_base = state.claude_dir(scope).join("skills");
    fs::create_dir_all(&dest_base).map_err(|e| e.to_string())?;
    let dest = dest_base.join(&folder_name);

    if dest.exists() {
        return Err(i18n::skill_already_exists_on_import(lang, &folder_name));
    }

    copy_dir_recursive(&src, &dest).map_err(|e| e.to_string())?;
    Ok(folder_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercises against a real local git repo (git clone accepts local paths
    // as a URL), so this covers the actual `git clone` + walk + frontmatter
    // parse path, not just the pure-Rust pieces.
    const TEST_REPO: &str = "C:/Users/Daniel/AppData/Local/Temp/fake-skills-repo";

    #[test]
    fn finds_both_skills_in_nested_folders_and_ignores_dotted_dir_duplicates() {
        // The repo also has .openclaw/skills/skill-one/SKILL.md — a near
        // duplicate kept for a different tool — which must NOT show up here.
        // Regression test for repos like github.com/DietrichGebert/ponytail,
        // which doubled every result because only ".git" was excluded.
        let skills = list_remote_skills_impl(Lang::En, TEST_REPO).expect("should find skills");
        assert_eq!(skills.len(), 3, "dotted-dir duplicates must be excluded: {skills:?}");
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"skill-one"));
        assert!(names.contains(&"skill-two"));
        assert!(names.contains(&"skill-three"));
        let one = skills.iter().find(|s| s.name == "skill-one").unwrap();
        assert_eq!(one.rel_path, "skill-one", "must be the top-level copy, not .openclaw's");
        let two = skills.iter().find(|s| s.name == "skill-two").unwrap();
        assert_eq!(two.rel_path, "nested/skill-two");
        assert_eq!(two.extra_files, vec!["reference.md".to_string()]);
    }

    #[test]
    fn find_skill_rel_path_locates_a_nested_skill_by_folder_name() {
        let tmp = clone_repo(Lang::En, TEST_REPO).unwrap();
        let found = find_skill_rel_path(&tmp, "skill-two");
        fs::remove_dir_all(&tmp).unwrap();
        assert_eq!(found, Some("nested/skill-two".to_string()));
    }

    #[test]
    fn find_skill_rel_path_returns_none_for_an_unknown_id() {
        let tmp = clone_repo(Lang::En, TEST_REPO).unwrap();
        let found = find_skill_rel_path(&tmp, "does-not-exist");
        fs::remove_dir_all(&tmp).unwrap();
        assert_eq!(found, None);
    }

    #[test]
    fn import_copies_the_right_folder_and_skips_git_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().join("project");
        let global_root = dir.path().join("global");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&global_root).unwrap();

        let tmp = clone_repo(Lang::En, TEST_REPO).unwrap();
        let src = tmp.join("nested/skill-two");
        let dest_base = project_root.join(".claude").join("skills");
        fs::create_dir_all(&dest_base).unwrap();
        copy_dir_recursive(&src, &dest_base.join("skill-two")).unwrap();
        fs::remove_dir_all(&tmp).unwrap();

        let copied = dest_base.join("skill-two");
        assert!(copied.join("SKILL.md").exists());
        assert!(copied.join("reference.md").exists());
        assert!(!copied.join(".git").exists());
    }

    #[test]
    fn installing_three_different_skills_at_once_all_succeed() {
        // Reproduces clicking "Instalar" on 3 different list rows in quick
        // succession: three real `git clone` + copy runs in flight at the
        // same time, same as the UI now allows (independent per-row state).
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().join("project");
        let global_root = dir.path().join("global");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&global_root).unwrap();
        let state = AppState::new(project_root, global_root);

        let rel_paths = ["skill-one", "nested/skill-two", "skill-three"];
        let results: Vec<Result<String, String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = rel_paths
                .iter()
                .map(|rel_path| {
                    let state = &state;
                    scope.spawn(move || {
                        import_skill_impl(state, TEST_REPO.to_string(), rel_path.to_string(), Scope::Project)
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        for (rel_path, result) in rel_paths.iter().zip(results.iter()) {
            assert!(result.is_ok(), "installing {rel_path} concurrently with the others failed: {result:?}");
        }

        for name in ["skill-one", "skill-two", "skill-three"] {
            assert!(
                skills_installed_dir(&state).join(name).join("SKILL.md").exists(),
                "{name} should have landed on disk"
            );
        }
    }

    fn skills_installed_dir(state: &AppState) -> std::path::PathBuf {
        state.claude_dir(Scope::Project).join("skills")
    }
}
