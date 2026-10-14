use crate::models::AppState;
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Watches both the project's and the global `.claude` directories for
/// changes made outside the app (hand-editing a file, `git pull`, another
/// tool) and tells the frontend to refresh. One `claude-files-changed`
/// event per burst: a single save can fire several raw fs events (a hook
/// save rewrites a whole `_clman` directory), so we drain whatever follows
/// the first event for a short quiet window before emitting once.
pub fn start(app: AppHandle, state: &AppState) {
    let project_dir = state.claude_dir(crate::models::Scope::Project);
    let global_dir = state.claude_dir(crate::models::Scope::Global);
    // The watched path has to exist up front for `notify` to accept it —
    // both are created lazily on first save otherwise (confirmed: a fresh
    // install has no ~/.claude at all until something is saved globally).
    let _ = std::fs::create_dir_all(&project_dir);
    let _ = std::fs::create_dir_all(&global_dir);

    // The project selector calls `start` again on every switch — bumping
    // the generation here (not inside the thread) means a switch always
    // wins immediately, even if the new thread hasn't scheduled yet.
    let generation = state.next_watcher_generation();
    let state = state.clone();
    std::thread::spawn(move || watch_loop(app, project_dir, global_dir, generation, state));
}

fn watch_loop(app: AppHandle, project_dir: PathBuf, global_dir: PathBuf, generation: u64, state: AppState) {
    let (tx, rx) = channel();
    let Ok(mut watcher) = notify::recommended_watcher(tx) else {
        return;
    };
    let _ = watcher.watch(&project_dir, RecursiveMode::Recursive);
    if global_dir != project_dir {
        let _ = watcher.watch(&global_dir, RecursiveMode::Recursive);
    }

    while rx.recv().is_ok() {
        // A newer watcher has taken over (the project changed) — stop for
        // good instead of emitting stale refreshes or lingering forever.
        if state.watcher_generation() != generation {
            return;
        }
        loop {
            match rx.recv_timeout(Duration::from_millis(400)) {
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        if state.watcher_generation() != generation {
            return;
        }
        let _ = app.emit("claude-files-changed", ());
    }
}
