use crate::fsutil::silent_command;
use crate::i18n;
use crate::models::Lang;
use std::path::PathBuf;
use std::sync::OnceLock;

static GIT_BIN: OnceLock<bool> = OnceLock::new();

fn detect_git() -> bool {
    *GIT_BIN.get_or_init(|| {
        silent_command("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// Clones `repo_url` into a fresh temp dir and returns its path. Depth-1,
/// quiet clone — callers walk the result and delete it themselves when done.
/// Shared by every "import from a repo" feature (skills, MCP servers, ...).
pub fn clone_repo(lang: Lang, repo_url: &str) -> Result<PathBuf, String> {
    if !detect_git() {
        return Err(i18n::git_not_found(lang));
    }
    // A fresh, genuinely unique directory per call — not derived from the
    // process id, which is shared by every test (and every command call)
    // running in this same process and caused real collisions under
    // `cargo test`'s default parallel execution.
    let tmp = tempfile::Builder::new()
        .prefix("clman-git-import-")
        .tempdir()
        .map_err(|e| e.to_string())?
        .keep();

    let output = silent_command("git")
        .args(["clone", "--depth", "1", "--quiet", "--", repo_url])
        .arg(&tmp)
        .output()
        .map_err(|e| i18n::git_exec_failed(lang, &e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(i18n::git_clone_failed(lang, stderr.trim()));
    }
    Ok(tmp)
}
