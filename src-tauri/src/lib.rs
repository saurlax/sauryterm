mod pty;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(pty::PtyState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            pty::open_pty,
            pty::write_pty,
            pty::close_pty
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
