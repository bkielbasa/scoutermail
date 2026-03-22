use futures::StreamExt;
use log::{info, warn};

use crate::parser::mime::parse_email;
use crate::parser::threading::assign_threads;
use crate::store::db::{Database, Folder, Message};

use super::client::{ImapError, ImapSession};

// ---------------------------------------------------------------------------
// Sync engine
// ---------------------------------------------------------------------------

/// Synchronise a single IMAP folder into the local database.
///
/// Strategy:
/// - SELECT the mailbox and read uidvalidity / uid_next.
/// - Compare with the stored folder state.
/// - If uidvalidity changed, do a full resync (1:*).
/// - Otherwise fetch only from stored uid_next to *.
/// - For each fetched message, parse it and upsert into the DB.
/// - Auto-extract contacts from the From header.
/// - Run threading on all messages in the folder.
/// - Update the folder sync state.
pub async fn sync_folder(
    session: &mut ImapSession,
    db: &Database,
    folder_name: &str,
) -> Result<Vec<Message>, ImapError> {
    // 1. SELECT the mailbox
    let mailbox = session
        .select(folder_name)
        .await
        .map_err(|e| ImapError::Imap(format!("select {}: {}", folder_name, e)))?;

    let server_uidvalidity = mailbox.uid_validity.unwrap_or(0);
    let server_uidnext = mailbox.uid_next.unwrap_or(1);

    // 2. Load stored folder state
    let stored_folder = db
        .get_folders()
        .map_err(|e| ImapError::Imap(e.to_string()))?
        .into_iter()
        .find(|f| f.name == folder_name);

    // 3. Determine fetch range
    let fetch_range = match &stored_folder {
        Some(sf) if sf.uidvalidity == Some(server_uidvalidity) => {
            // Same uidvalidity — incremental sync
            let from_uid = sf.uidnext.unwrap_or(1);
            if from_uid >= server_uidnext {
                // Nothing new to fetch
                info!("folder {} is up to date", folder_name);
                update_folder_state(db, folder_name, server_uidvalidity, server_uidnext)?;
                let messages = db
                    .get_messages_by_folder(folder_name)
                    .map_err(|e| ImapError::Imap(e.to_string()))?;
                return Ok(messages);
            }
            format!("{}:*", from_uid)
        }
        _ => {
            // No stored state or uidvalidity changed — full resync
            if stored_folder.is_some() {
                warn!(
                    "uidvalidity changed for {}, performing full resync",
                    folder_name
                );
            }
            "1:*".to_string()
        }
    };

    // 4. FETCH messages
    info!(
        "fetching UIDs {} from folder {}",
        fetch_range, folder_name
    );

    let fetch_query = "(UID FLAGS BODY.PEEK[HEADER] BODY.PEEK[TEXT] INTERNALDATE)";
    let fetches_stream = session
        .uid_fetch(&fetch_range, fetch_query)
        .await
        .map_err(|e| ImapError::Imap(format!("uid_fetch: {}", e)))?;

    let fetches: Vec<_> = fetches_stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    // 5. Parse and store each fetched message
    for fetch in &fetches {
        let uid = match fetch.uid {
            Some(u) => u,
            None => continue,
        };

        // Combine header + text for full parsing
        let header_bytes = fetch.header().unwrap_or_default();
        let text_bytes = fetch.text().unwrap_or_default();
        let mut raw = Vec::with_capacity(header_bytes.len() + 2 + text_bytes.len());
        raw.extend_from_slice(header_bytes);
        if !header_bytes.is_empty() && !header_bytes.ends_with(b"\r\n\r\n") {
            raw.extend_from_slice(b"\r\n");
        }
        raw.extend_from_slice(text_bytes);

        let parsed_email = match parse_email(&raw) {
            Ok(p) => p,
            Err(e) => {
                warn!("failed to parse message UID {}: {}", uid, e);
                // Still store a minimal message
                let msg = Message {
                    uid,
                    message_id: None,
                    folder: folder_name.to_string(),
                    subject: None,
                    from_addr: None,
                    to_addr: None,
                    cc: None,
                    date: fetch.internal_date().map(|d| d.to_rfc3339()),
                    body_text: None,
                    body_html: None,
                    flags: Some(format_flags(fetch)),
                    thread_id: None,
                    ref_headers: None,
                    in_reply_to: None,
                    reply_to: None,
                    list_unsubscribe: None,
                };
                db.upsert_message(&msg)
                    .map_err(|e| ImapError::Imap(format!("upsert_message: {}", e)))?;
                continue;
            }
        };

        // Build the Message struct from parsed email
        let date = parsed_email
            .date
            .or_else(|| fetch.internal_date().map(|d| d.to_rfc3339()));
        let ref_headers = if parsed_email.references.is_empty() {
            None
        } else {
            Some(parsed_email.references.join(" "))
        };

        let msg = Message {
            uid,
            message_id: parsed_email.message_id,
            folder: folder_name.to_string(),
            subject: parsed_email.subject,
            from_addr: parsed_email.from,
            to_addr: parsed_email.to,
            cc: parsed_email.cc,
            date,
            body_text: parsed_email.body_text,
            body_html: parsed_email.body_html,
            flags: Some(format_flags(fetch)),
            thread_id: None,
            ref_headers,
            in_reply_to: parsed_email.in_reply_to,
            reply_to: parsed_email.reply_to,
            list_unsubscribe: parsed_email.list_unsubscribe,
        };

        // Auto-extract contacts from From header
        if let Some(ref from) = msg.from_addr {
            if let Err(e) = db.upsert_contact(from, None) {
                warn!("failed to upsert contact '{}': {}", from, e);
            }
        }

        // Store message
        db.upsert_message(&msg)
            .map_err(|e| ImapError::Imap(format!("upsert_message: {}", e)))?;

        // Store attachments
        if !parsed_email.attachments.is_empty() {
            // Remove old attachments for this message (in case of re-sync)
            if let Err(e) = db.delete_attachments_for_message(uid, folder_name) {
                warn!("failed to delete old attachments for uid={}: {}", uid, e);
            }
            for att in &parsed_email.attachments {
                if let Err(e) = db.insert_attachment(
                    uid,
                    folder_name,
                    Some(&att.filename),
                    Some(&att.content_type),
                    Some(att.size as i64),
                    &att.data,
                ) {
                    warn!("failed to insert attachment '{}' for uid={}: {}", att.filename, uid, e);
                }
            }
        }

        // Detect and store calendar events from ICS parts
        if !parsed_email.calendar_data.is_empty() {
            for ics_data in &parsed_email.calendar_data {
                let cal_events = crate::calendar::parser::parse_ics(ics_data);
                for cal_event in &cal_events {
                    let status = match cal_event.method.as_deref() {
                        Some("CANCEL") => "cancelled",
                        _ => "needs-action",
                    };
                    if let Err(e) = db.upsert_event(cal_event, uid, folder_name, status) {
                        warn!("failed to upsert calendar event '{}' for uid={}: {}", cal_event.event_uid, uid, e);
                    }
                }
            }
        }
    }

    // 6. Run threading on all messages in the folder
    let mut all_messages = db
        .get_messages_by_folder(folder_name)
        .map_err(|e| ImapError::Imap(e.to_string()))?;

    assign_threads(&mut all_messages);

    // Update thread IDs in the database
    for msg in &all_messages {
        db.upsert_message(msg)
            .map_err(|e| ImapError::Imap(format!("upsert thread: {}", e)))?;
    }

    // 7. Update folder sync state
    update_folder_state(db, folder_name, server_uidvalidity, server_uidnext)?;

    info!(
        "synced {} messages in folder {}",
        all_messages.len(),
        folder_name
    );

    Ok(all_messages)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a single IMAP FETCH response into a domain `Message`.
#[allow(dead_code)]
fn parse_fetched(fetch: &async_imap::types::Fetch, folder: &str) -> Option<Message> {
    let uid = fetch.uid?;

    // Combine header + text into full raw email for the parser
    let header_bytes = fetch.header().unwrap_or_default();
    let text_bytes = fetch.text().unwrap_or_default();

    let mut raw = Vec::with_capacity(header_bytes.len() + 2 + text_bytes.len());
    raw.extend_from_slice(header_bytes);
    if !header_bytes.is_empty() && !header_bytes.ends_with(b"\r\n\r\n") {
        raw.extend_from_slice(b"\r\n");
    }
    raw.extend_from_slice(text_bytes);

    // Parse using the MIME parser
    let parsed = match parse_email(&raw) {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to parse message UID {}: {}", uid, e);
            // Still create a minimal message entry
            return Some(Message {
                uid,
                message_id: None,
                folder: folder.to_string(),
                subject: None,
                from_addr: None,
                to_addr: None,
                cc: None,
                date: fetch
                    .internal_date()
                    .map(|d| d.to_rfc3339()),
                body_text: None,
                body_html: None,
                flags: Some(format_flags(fetch)),
                thread_id: None,
                ref_headers: None,
                in_reply_to: None,
                reply_to: None,
                list_unsubscribe: None,
            });
        }
    };

    // Use INTERNALDATE as fallback for date
    let date = parsed
        .date
        .or_else(|| fetch.internal_date().map(|d| d.to_rfc3339()));

    let ref_headers = if parsed.references.is_empty() {
        None
    } else {
        Some(parsed.references.join(" "))
    };

    Some(Message {
        uid,
        message_id: parsed.message_id,
        folder: folder.to_string(),
        subject: parsed.subject,
        from_addr: parsed.from,
        to_addr: parsed.to,
        cc: parsed.cc,
        date,
        body_text: parsed.body_text,
        body_html: parsed.body_html,
        flags: Some(format_flags(fetch)),
        thread_id: None, // Will be set by assign_threads
        ref_headers,
        in_reply_to: parsed.in_reply_to,
        reply_to: parsed.reply_to,
        list_unsubscribe: parsed.list_unsubscribe,
    })
}

/// Format IMAP flags into a space-separated string (e.g. `\Seen \Answered`).
fn format_flags(fetch: &async_imap::types::Fetch) -> String {
    fetch
        .flags()
        .map(|f| format!("{:?}", f))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Update the folder sync state in the database.
fn update_folder_state(
    db: &Database,
    folder_name: &str,
    uidvalidity: u32,
    uidnext: u32,
) -> Result<(), ImapError> {
    let now = chrono::Utc::now().to_rfc3339();
    let folder = Folder {
        name: folder_name.to_string(),
        uidvalidity: Some(uidvalidity),
        uidnext: Some(uidnext),
        last_sync: Some(now),
    };
    db.upsert_folder(&folder)
        .map_err(|e| ImapError::Imap(format!("upsert_folder: {}", e)))?;
    Ok(())
}
