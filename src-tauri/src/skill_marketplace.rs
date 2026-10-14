//! Discovery for skills.sh (vercel-labs/skills) — the same community-wide
//! skill index its own official `skills` CLI uses for `skills find`. We hit
//! its `/api/search` endpoint directly (same one the CLI calls — confirmed
//! by inspecting the published package) instead of that CLI, since the CLI's
//! own install flow assumes a node_modules/package.json project, which would
//! force an npm dependency onto projects clman manages that aren't Node at
//! all. Install reuses the existing git-clone-and-copy path from
//! skill_import.rs instead.
use crate::git::clone_repo;
use crate::i18n;
use crate::models::{AppState, Lang, Scope};
use crate::skill_import::{find_skill_rel_path, install_from_tmp};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::State;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketplaceSkillSummary {
    /// "owner/repo" on GitHub — what we clone to install it.
    pub source: String,
    /// The skill's folder name inside that repo, also its install id.
    pub skill_id: String,
    pub name: String,
    pub installs: u64,
}

fn parse_search_response(body: &str) -> Vec<MarketplaceSkillSummary> {
    let doc: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    doc.get("skills")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|v| MarketplaceSkillSummary {
                    source: v.get("source").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                    skill_id: v.get("skillId").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                    name: v.get("name").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                    installs: v.get("installs").and_then(|x| x.as_u64()).unwrap_or(0),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// skills.sh itself rejects anything under 2 chars — mirrored here so we
/// don't even make the request for a query that will just error.
fn search_marketplace_skills_impl(lang: Lang, query: &str) -> Result<Vec<MarketplaceSkillSummary>, String> {
    let query = query.trim();
    if query.chars().count() < 2 {
        return Ok(Vec::new());
    }
    let body = ureq::get("https://skills.sh/api/search")
        .query("q", query)
        .call()
        .map_err(|e| i18n::marketplace_request_failed(lang, &e.to_string()))?
        .into_string()
        .map_err(|e| i18n::marketplace_request_failed(lang, &e.to_string()))?;
    Ok(parse_search_response(&body))
}

#[tauri::command]
pub async fn search_marketplace_skills(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<MarketplaceSkillSummary>, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || search_marketplace_skills_impl(lang, &query))
        .await
        .map_err(|e| e.to_string())?
}

fn install_marketplace_skill_impl(
    state: &AppState,
    source: String,
    skill_id: String,
    scope: Scope,
) -> Result<String, String> {
    let lang = state.lang();
    let repo_url = format!("https://github.com/{source}");
    let tmp = clone_repo(lang, &repo_url)?;
    let result = (|| {
        let rel_path =
            find_skill_rel_path(&tmp, &skill_id).ok_or_else(|| i18n::skill_not_found_in_repo(lang, &skill_id))?;
        install_from_tmp(state, lang, &tmp, &rel_path, scope)
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketplaceSkillPreview {
    pub description: String,
    pub when_to_use: String,
    pub body: String,
}

fn find_path_in_tree(tree_body: &str, skill_id: &str) -> Option<String> {
    let doc: serde_json::Value = serde_json::from_str(tree_body).ok()?;
    let suffix = format!("/{skill_id}/SKILL.md");
    let root = format!("{skill_id}/SKILL.md");
    doc.get("tree")?.as_array()?.iter().find_map(|e| {
        let p = e.get("path").and_then(|p| p.as_str())?;
        (p == root || p.ends_with(&suffix)).then(|| p.to_string())
    })
}

/// A 404 from GitHub here means the repo itself (or the file in it) is
/// gone — skills.sh's index lagging behind a repo that was since deleted,
/// renamed or made private — which reads very differently from "I can't
/// reach GitHub at all", so it gets its own message instead of a generic one.
fn github_error(lang: Lang, source: &str, err: ureq::Error) -> String {
    match err {
        ureq::Error::Status(404, _) => i18n::skill_repo_gone(lang, source),
        other => i18n::github_request_failed(lang, &other.to_string()),
    }
}

/// Fetches just the one `SKILL.md` the user clicked to preview — a GitHub
/// tree lookup (to locate it, since a search result only has the skill's
/// folder name, not its path inside the repo) plus one raw-content fetch.
/// Deliberately not a full `git clone` like install does: previewing is a
/// read of a few KB of markdown, cloning the whole repo for that would be
/// wasteful for something the user might click through several of per search.
fn preview_marketplace_skill_impl(lang: Lang, source: &str, skill_id: &str) -> Result<MarketplaceSkillPreview, String> {
    let tree_response = ureq::get(&format!("https://api.github.com/repos/{source}/git/trees/HEAD?recursive=1"))
        .set("User-Agent", "clman")
        .call()
        .map_err(|e| github_error(lang, source, e))?;
    let tree_body = tree_response
        .into_string()
        .map_err(|e| i18n::github_request_failed(lang, &e.to_string()))?;
    let path = find_path_in_tree(&tree_body, skill_id).ok_or_else(|| i18n::skill_not_found_in_repo(lang, skill_id))?;

    let raw_response = ureq::get(&format!("https://raw.githubusercontent.com/{source}/HEAD/{path}"))
        .call()
        .map_err(|e| github_error(lang, source, e))?;
    let raw = raw_response
        .into_string()
        .map_err(|e| i18n::github_request_failed(lang, &e.to_string()))?;

    let (yaml, body) = crate::fsutil::split_frontmatter(&raw);
    let fm: crate::models::SkillFrontmatter = yaml.as_deref().and_then(|y| serde_yaml::from_str(y).ok()).unwrap_or_default();
    Ok(MarketplaceSkillPreview {
        description: fm.description.unwrap_or_default(),
        when_to_use: fm.when_to_use.unwrap_or_default(),
        body,
    })
}

#[tauri::command]
pub async fn preview_marketplace_skill(
    state: State<'_, AppState>,
    source: String,
    skill_id: String,
) -> Result<MarketplaceSkillPreview, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || preview_marketplace_skill_impl(lang, &source, &skill_id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn install_marketplace_skill(
    state: State<'_, AppState>,
    source: String,
    skill_id: String,
    scope: Scope,
) -> Result<String, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || install_marketplace_skill_impl(&state, source, skill_id, scope))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_search_response_reads_the_real_skills_sh_shape() {
        let body = r#"{"query":"test","searchType":"fuzzy","searchVersion":"algolia","skills":[
            {"id":"anthropics/skills/webapp-testing","source":"anthropics/skills","skillId":"webapp-testing","name":"webapp-testing","installs":177038}
        ]}"#;
        let skills = parse_search_response(body);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source, "anthropics/skills");
        assert_eq!(skills[0].skill_id, "webapp-testing");
        assert_eq!(skills[0].installs, 177038);
    }

    #[test]
    fn parse_search_response_handles_an_empty_or_malformed_body() {
        assert!(parse_search_response("not json").is_empty());
        assert!(parse_search_response(r#"{"skills":[]}"#).is_empty());
    }

    #[test]
    fn find_path_in_tree_matches_a_nested_skill_by_folder_name() {
        let tree = r#"{"tree":[
            {"path":"README.md","type":"blob"},
            {"path":"skills/webapp-testing/SKILL.md","type":"blob"},
            {"path":"skills/webapp-testing/reference.md","type":"blob"}
        ]}"#;
        assert_eq!(
            find_path_in_tree(tree, "webapp-testing"),
            Some("skills/webapp-testing/SKILL.md".to_string())
        );
        assert_eq!(find_path_in_tree(tree, "does-not-exist"), None);
    }

    #[test]
    fn find_path_in_tree_matches_a_root_level_skill() {
        let tree = r#"{"tree":[{"path":"my-skill/SKILL.md","type":"blob"}]}"#;
        assert_eq!(find_path_in_tree(tree, "my-skill"), Some("my-skill/SKILL.md".to_string()));
    }

    #[test]
    fn github_error_tells_a_gone_repo_apart_from_a_real_connectivity_failure() {
        let response = ureq::Response::new(404, "Not Found", "{}").unwrap();
        let msg = github_error(Lang::En, "marvinrichter/clarc", ureq::Error::Status(404, response));
        assert!(msg.contains("marvinrichter/clarc"), "should name the repo: {msg}");
        assert!(
            !msg.to_lowercase().contains("couldn't reach") && !msg.to_lowercase().contains("couldn't connect"),
            "a 404 (gone repo) isn't a connectivity failure, the message shouldn't read like one: {msg}"
        );
    }

    #[test]
    fn search_marketplace_skills_impl_short_circuits_a_too_short_query_without_a_request() {
        // Mirrors skills.sh's own "Query must be at least 2 characters" rule
        // so we don't burn a network round trip on every single keystroke.
        assert_eq!(search_marketplace_skills_impl(Lang::En, "a").unwrap(), Vec::new());
        assert_eq!(search_marketplace_skills_impl(Lang::En, " ").unwrap(), Vec::new());
    }
}
