mod term;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(term::TerminalState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            term::open_term,
            term::write_term,
            term::close_term
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
