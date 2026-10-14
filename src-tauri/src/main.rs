// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Capture the directory the user was standing in when they ran `clman`,
    // before Tauri/webview init can touch the working directory.
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    claude_manager_lib::run(project_root);
}
