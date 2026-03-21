use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::accounts::manager::{AccountConfig, AccountManager};
use crate::calendar::parser::build_ics_reply;
use crate::imap::client::{self as imap_client, ImapConfig};
use crate::smtp::client::ComposeEmail;
use crate::store::db::{AttachmentInfo, Contact, Database, Folder, Message, StoredEvent};
use crate::store::search::SearchIndex;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

pub struct AppState {
    pub account_manager: Arc<Mutex<AccountManager>>,
    pub active_account: Arc<Mutex<Option<String>>>,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AddAccountRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct SendEmailRequest {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

// ---------------------------------------------------------------------------
// Helper: get active account ID or error
// ---------------------------------------------------------------------------

async fn get_active_id(state: &State<'_, AppState>) -> Result<String, String> {
    let active = state.active_account.lock().await;
    active
        .clone()
        .ok_or_else(|| "no active account set".to_string())
}

/// Open the SQLite database for the active account.
async fn open_db(state: &State<'_, AppState>) -> Result<Database, String> {
    let id = get_active_id(state).await?;
    let mgr = state.account_manager.lock().await;
    let db_path = mgr.db_path(&id);
    let db_path_str = db_path.to_string_lossy().to_string();
    Database::open(&db_path_str).map_err(|e| e.to_string())
}

/// Open the Tantivy search index for the active account.
async fn open_search_index(state: &State<'_, AppState>) -> Result<SearchIndex, String> {
    let id = get_active_id(state).await?;
    let mgr = state.account_manager.lock().await;
    let idx_path = mgr.search_index_path(&id);
    SearchIndex::open(&idx_path).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Account commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn add_account(
    state: State<'_, AppState>,
    req: AddAccountRequest,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let config = AccountConfig {
        id: id.clone(),
        name: req.name,
        email: req.email,
        imap_host: req.imap_host,
        imap_port: req.imap_port,
        smtp_host: req.smtp_host,
        smtp_port: req.smtp_port,
        username: req.username,
    };
    let mut mgr = state.account_manager.lock().await;
    mgr.add_account(config, &req.password)
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountConfig>, String> {
    let mgr = state.account_manager.lock().await;
    Ok(mgr.list_accounts().to_vec())
}

#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut mgr = state.account_manager.lock().await;
    mgr.remove_account(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_account(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Verify the account exists.
    {
        let mgr = state.account_manager.lock().await;
        mgr.get_account(&id).map_err(|e| e.to_string())?;
    }
    let mut active = state.active_account.lock().await;
    *active = Some(id);
    Ok(())
}

#[tauri::command]
pub async fn get_provider_defaults(
    provider: String,
) -> Result<Option<(String, u16, String, u16)>, String> {
    Ok(crate::accounts::manager::provider_defaults(&provider))
}

// ---------------------------------------------------------------------------
// Folder counts
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_folder_counts(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64, i64)>, String> {
    let db = open_db(&state).await?;
    db.get_folder_counts().map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Connection test
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn test_imap_connection(
    host: String,
    port: u16,
    username: String,
    password: String,
) -> Result<Vec<String>, String> {
    let config = ImapConfig {
        host,
        port,
        username,
        password,
    };
    let mut session = imap_client::connect(&config)
        .await
        .map_err(|e| e.to_string())?;
    let folders = imap_client::list_folders(&mut session)
        .await
        .map_err(|e| e.to_string())?;
    session.logout().await.map_err(|e| e.to_string())?;
    Ok(folders)
}

// ---------------------------------------------------------------------------
// Mail commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn sync_folder(
    state: State<'_, AppState>,
    folder: String,
) -> Result<Vec<Message>, String> {
    let id = get_active_id(&state).await?;

    // Gather paths and config while holding the lock, then release it.
    let (imap_config, db_path, idx_path) = {
        let mgr = state.account_manager.lock().await;
        let imap_cfg = mgr.get_imap_config(&id).map_err(|e| e.to_string())?;
        let db_p = mgr.db_path(&id).to_string_lossy().to_string();
        let idx_p = mgr.search_index_path(&id);
        (imap_cfg, db_p, idx_p)
    };

    // Run the IMAP sync + DB work on a blocking thread so that the
    // non-Send `Database` does not cross an await boundary in the
    // outer (Send-required) Tauri command future.
    let handle = tokio::runtime::Handle::current();
    let messages = tokio::task::spawn_blocking(move || {
        handle.block_on(async move {
            let mut session = imap_client::connect(&imap_config)
                .await
                .map_err(|e| e.to_string())?;

            let db = Database::open(&db_path).map_err(|e| e.to_string())?;

            let messages =
                crate::imap::sync::sync_folder(&mut session, &db, &folder)
                    .await
                    .map_err(|e| e.to_string())?;

            // Index messages in Tantivy.
            let search_index =
                SearchIndex::open(&idx_path).map_err(|e| e.to_string())?;
            let writer = search_index.writer().map_err(|e| e.to_string())?;
            for msg in &messages {
                let _ = search_index.index_message(
                    &writer,
                    msg.uid as i64,
                    &msg.folder,
                    msg.subject.as_deref().unwrap_or(""),
                    msg.from_addr.as_deref().unwrap_or(""),
                    msg.to_addr.as_deref().unwrap_or(""),
                    msg.body_text.as_deref().unwrap_or(""),
                );
            }
            search_index.commit(writer).map_err(|e| e.to_string())?;

            let _ = session.logout().await;

            Ok::<Vec<Message>, String>(messages)
        })
    })
    .await
    .map_err(|e| format!("sync task panicked: {}", e))??;

    Ok(messages)
}

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    folder: String,
) -> Result<Vec<Message>, String> {
    let db = open_db(&state).await?;
    db.get_messages_by_folder(&folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_message(
    state: State<'_, AppState>,
    uid: u32,
    folder: String,
) -> Result<Message, String> {
    let db = open_db(&state).await?;
    db.get_message(uid, &folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_thread_messages(
    state: State<'_, AppState>,
    thread_id: String,
) -> Result<Vec<Message>, String> {
    let db = open_db(&state).await?;
    db.get_thread_messages(&thread_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_folders(
    state: State<'_, AppState>,
) -> Result<Vec<Folder>, String> {
    let db = open_db(&state).await?;
    db.get_folders().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_messages(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<Message>, String> {
    let limit = limit.unwrap_or(50);
    let search_index = open_search_index(&state).await?;
    let results = search_index
        .search(&query, limit)
        .map_err(|e| e.to_string())?;

    let db = open_db(&state).await?;
    let mut messages = Vec::with_capacity(results.len());
    for r in results {
        if let Ok(msg) = db.get_message(r.uid as u32, &r.folder) {
            messages.push(msg);
        }
    }
    Ok(messages)
}

// ---------------------------------------------------------------------------
// Send
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn send_email(
    state: State<'_, AppState>,
    req: SendEmailRequest,
) -> Result<(), String> {
    let id = get_active_id(&state).await?;
    let (smtp_config, from_email) = {
        let mgr = state.account_manager.lock().await;
        let smtp = mgr.get_smtp_config(&id).map_err(|e| e.to_string())?;
        let account = mgr.get_account(&id).map_err(|e| e.to_string())?;
        (smtp, account.email.clone())
    };

    let compose = ComposeEmail {
        from: from_email,
        to: req.to,
        cc: req.cc,
        bcc: req.bcc,
        subject: req.subject,
        body_text: req.body_text,
        body_html: req.body_html,
        in_reply_to: req.in_reply_to,
        references: req.references,
    };

    crate::smtp::client::send_email(&smtp_config, &compose)
        .await
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Flags
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn update_flags(
    state: State<'_, AppState>,
    uid: u32,
    folder: String,
    flags: String,
) -> Result<(), String> {
    let db = open_db(&state).await?;
    db.update_flags(uid, &folder, &flags).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_message(
    state: State<'_, AppState>,
    uid: u32,
    folder: String,
) -> Result<(), String> {
    let db = open_db(&state).await?;
    db.delete_message(uid, &folder).map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Move
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn move_message(
    state: State<'_, AppState>,
    uid: u32,
    from_folder: String,
    to_folder: String,
) -> Result<(), String> {
    let id = get_active_id(&state).await?;
    let imap_config = {
        let mgr = state.account_manager.lock().await;
        mgr.get_imap_config(&id).map_err(|e| e.to_string())?
    };

    // Move on IMAP server (uses spawn_blocking like sync_folder to avoid Send issues)
    let from = from_folder.clone();
    let to = to_folder.clone();
    let handle = tokio::runtime::Handle::current();
    tokio::task::spawn_blocking(move || {
        handle.block_on(async move {
            let mut session = imap_client::connect(&imap_config)
                .await
                .map_err(|e| e.to_string())?;
            imap_client::move_message(&mut session, uid, &from, &to)
                .await
                .map_err(|e| e.to_string())?;
            let _ = session.logout().await;
            Ok::<(), String>(())
        })
    })
    .await
    .map_err(|e| format!("move task panicked: {}", e))??;

    // Update local DB: delete from old folder (message will appear in new folder on next sync)
    let db = open_db(&state).await?;
    db.delete_message(uid, &from_folder).map_err(|e| e.to_string())?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Contacts
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn search_contacts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<Contact>, String> {
    let db = open_db(&state).await?;
    db.search_contacts(&query, limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_contacts(
    state: State<'_, AppState>,
) -> Result<Vec<Contact>, String> {
    let db = open_db(&state).await?;
    db.get_all_contacts().map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Calendar commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_events(
    state: State<'_, AppState>,
) -> Result<Vec<StoredEvent>, String> {
    let db = open_db(&state).await?;
    db.get_events().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_events_in_range(
    state: State<'_, AppState>,
    start: i64,
    end: i64,
) -> Result<Vec<StoredEvent>, String> {
    let db = open_db(&state).await?;
    db.get_events_in_range(start, end).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_events_for_message(
    state: State<'_, AppState>,
    uid: u32,
    folder: String,
) -> Result<Vec<StoredEvent>, String> {
    let db = open_db(&state).await?;
    db.get_events_for_message(uid, &folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn respond_to_invite(
    state: State<'_, AppState>,
    event_uid: String,
    response: String,
) -> Result<(), String> {
    let partstat = match response.as_str() {
        "accepted" => "ACCEPTED",
        "declined" => "DECLINED",
        "tentative" => "TENTATIVE",
        _ => return Err(format!("invalid response: {}", response)),
    };

    let db = open_db(&state).await?;
    let event = db.get_event(&event_uid).map_err(|e| e.to_string())?;

    let organizer = event
        .organizer
        .as_deref()
        .ok_or_else(|| "event has no organizer".to_string())?
        .to_string();

    let id = get_active_id(&state).await?;
    let (smtp_config, from_email) = {
        let mgr = state.account_manager.lock().await;
        let smtp = mgr.get_smtp_config(&id).map_err(|e| e.to_string())?;
        let account = mgr.get_account(&id).map_err(|e| e.to_string())?;
        (smtp, account.email.clone())
    };

    // Build the calendar event from the stored event for the reply builder
    let cal_event = crate::calendar::parser::CalendarEvent {
        event_uid: event.event_uid.clone(),
        summary: event.summary.clone(),
        dtstart: event.dtstart,
        dtend: event.dtend,
        location: event.location.clone(),
        description: event.description.clone(),
        organizer: event.organizer.clone(),
        attendees: event.attendees.clone().unwrap_or_else(|| "[]".to_string()),
        sequence: event.sequence,
        method: None,
        raw_ics: event.raw_ics.clone().unwrap_or_default(),
    };

    let ics_reply = build_ics_reply(&cal_event, &from_email, partstat);

    let subject = format!(
        "{}: {}",
        response,
        event.summary.as_deref().unwrap_or("Calendar Event")
    );

    let compose = ComposeEmail {
        from: from_email,
        to: vec![organizer],
        cc: vec![],
        bcc: vec![],
        subject,
        body_text: ics_reply,
        body_html: None,
        in_reply_to: None,
        references: vec![],
    };

    crate::smtp::client::send_email(&smtp_config, &compose)
        .await
        .map_err(|e| e.to_string())?;

    db.update_event_status(&event_uid, &response)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Attachment commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_attachments(
    state: State<'_, AppState>,
    uid: u32,
    folder: String,
) -> Result<Vec<AttachmentInfo>, String> {
    let db = open_db(&state).await?;
    db.get_attachments_for_message(uid, &folder)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_attachment(
    state: State<'_, AppState>,
    attachment_id: i64,
) -> Result<String, String> {
    let db = open_db(&state).await?;
    let (data, filename) = db
        .get_attachment_data(attachment_id)
        .map_err(|e| e.to_string())?;
    let downloads =
        dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap().join("Downloads"));
    let target_name = filename.unwrap_or_else(|| "attachment".to_string());
    let path = downloads.join(&target_name);
    std::fs::write(&path, &data).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
