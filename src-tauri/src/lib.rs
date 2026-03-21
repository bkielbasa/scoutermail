pub mod accounts;
pub mod calendar;
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

    accounts::keychain::init(&data_dir);

    let account_manager =
        AccountManager::new(data_dir).expect("failed to initialise AccountManager");

    let app_state = AppState {
        account_manager: Arc::new(Mutex::new(account_manager)),
        active_account: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri::plugin::Builder::<tauri::Wry, ()>::new("open-external-links")
                .on_navigation(|_webview, url| {
                    let s = url.as_str();
                    if s.starts_with("tauri://")
                        || s.starts_with("http://localhost")
                        || s.starts_with("about:")
                        || s.starts_with("javascript:")
                    {
                        return true;
                    }
                    let _ = open::that(s);
                    false
                })
                .build(),
        )
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
            commands::get_unified_messages,
            commands::get_message,
            commands::get_thread_messages,
            commands::get_folders,
            commands::search_messages,
            commands::send_email,
            commands::update_flags,
            commands::delete_message,
            commands::search_contacts,
            commands::get_all_contacts,
            commands::get_events,
            commands::get_events_in_range,
            commands::get_events_for_message,
            commands::respond_to_invite,
            commands::get_folder_counts,
            commands::move_message,
            commands::get_attachments,
            commands::save_attachment,
            commands::save_draft,
            commands::get_drafts,
            commands::get_draft,
            commands::delete_draft,
            commands::set_setting,
            commands::get_setting,
            commands::snooze_message,
            commands::check_snoozed,
            commands::unsnooze_message,
            commands::create_label,
            commands::get_labels,
            commands::delete_label,
            commands::label_message,
            commands::unlabel_message,
            commands::get_message_labels,
            commands::get_messages_by_label,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
