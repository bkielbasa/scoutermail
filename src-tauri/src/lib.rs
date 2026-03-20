pub mod accounts;
pub mod commands;
pub mod imap;
pub mod parser;
pub mod smtp;
pub mod store;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::accounts::manager::AccountManager;
use crate::commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = dirs::data_dir()
        .expect("could not determine data directory")
        .join("com.scoutermail");

    std::fs::create_dir_all(&data_dir).expect("failed to create data directory");

    let account_manager =
        AccountManager::new(data_dir).expect("failed to initialise AccountManager");

    let app_state = AppState {
        account_manager: Arc::new(Mutex::new(account_manager)),
        active_account: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::add_account,
            commands::list_accounts,
            commands::remove_account,
            commands::set_active_account,
            commands::get_provider_defaults,
            commands::test_imap_connection,
            commands::sync_folder,
            commands::get_messages,
            commands::get_message,
            commands::get_thread_messages,
            commands::get_folders,
            commands::search_messages,
            commands::send_email,
            commands::update_flags,
            commands::delete_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
