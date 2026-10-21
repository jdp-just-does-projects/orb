//! Tauri shell for orb: IPC commands, progress channels, thumbnail protocol.

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    orb_core::init_tracing();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::app_info])
        .run(tauri::generate_context!())
        .expect("error while running orb");
}
