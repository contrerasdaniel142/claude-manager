use crate::i18n;
use crate::models::Lang;
use std::path::Path;
use std::process::Command;

/// Runs `claude <subcommand> <args...>` and returns (stdout, stderr, exit_code).
/// Shared by every clman feature that defers to the real `claude` CLI instead
/// of reimplementing state Claude Code already owns (plugins, MCP servers) —
/// clman is a GUI in front of the already-trusted CLI, nothing more.
pub fn run(lang: Lang, subcommand: &str, args: &[&str]) -> Result<(String, String, i32), String> {
    run_in(lang, None, subcommand, args)
}

/// Like `run`, but rooted at `dir` — needed for project-scoped commands
/// (`claude mcp ... --scope project`), which resolve "this project" from the
/// process's current directory, not from any argument we can pass it.
pub fn run_in(lang: Lang, dir: Option<&Path>, subcommand: &str, args: &[&str]) -> Result<(String, String, i32), String> {
    let mut cmd = Command::new("claude");
    cmd.arg(subcommand).args(args);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            i18n::claude_cli_not_found(lang)
        } else {
            i18n::claude_command_exec_failed(lang, &e.to_string())
        }
    })?;
    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
    ))
}

/// For a command whose only output is a human-readable success/failure line
/// (no `--json`, e.g. `claude mcp add-json`/`remove`) — exit 0 means success,
/// and stdout already reads fine as the success message. Takes a `dir` like
/// `run_in`; pass `None` when the command doesn't need one (e.g. a plugin
/// marketplace command, which never resolves to "this project" by cwd).
pub fn run_text_command_in(
    lang: Lang,
    dir: Option<&Path>,
    subcommand: &str,
    args: &[&str],
) -> Result<String, String> {
    let (stdout, stderr, code) = run_in(lang, dir, subcommand, args)?;
    if code == 0 {
        return Ok(stdout.trim().to_string());
    }
    let reason = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
    Err(i18n::claude_returned_error(lang, reason))
}
