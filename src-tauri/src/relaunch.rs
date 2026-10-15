use crate::fsutil::silent_command;

/// `tauri-plugin-process`'s `relaunch()` respawns `current_exe()` inheriting
/// *this* process's own environment block — captured once, whenever Explorer
/// originally launched clman, and never refreshed since. If PATH changed
/// after that (e.g. the `claude` CLI moved or was reinstalled), every
/// self-update relaunch keeps carrying the stale PATH forward forever; only
/// a fresh launch from Explorer re-reads the registry. .NET's
/// `EnvironmentVariableTarget` bypasses our stale inherited snapshot and
/// reads the registry directly, so we shell out to it once here instead of
/// trusting our own inherited `PATH`.
#[cfg(windows)]
fn fresh_windows_path() -> Option<String> {
    let output = silent_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Relaunches clman into the just-installed update. Replaces
/// `@tauri-apps/plugin-process`'s `relaunch()` — see module docs for why.
#[tauri::command]
pub fn relaunch_app() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(&exe);

    #[cfg(windows)]
    {
        if let Some(fresh_path) = fresh_windows_path() {
            cmd.env("PATH", fresh_path);
        }
    }

    cmd.spawn().map_err(|e| e.to_string())?;
    std::process::exit(0);
}