use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use mailparse::dateparse;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("not found")]
    NotFound,
}

// ---------------------------------------------------------------------------
// Domain structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub uid: u32,
    pub message_id: Option<String>,
    pub folder: String,
    pub subject: Option<String>,
    pub from_addr: Option<String>,
    pub to_addr: Option<String>,
    pub cc: Option<String>,
    pub date: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub flags: Option<String>,
    pub thread_id: Option<String>,
    pub ref_headers: Option<String>,
    pub in_reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub thread_id: String,
    pub subject: Option<String>,
    pub last_date: Option<String>,
    pub message_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub name: String,
    pub uidvalidity: Option<u32>,
    pub uidnext: Option<u32>,
    pub last_sync: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub event_uid: String,
    pub message_uid: u32,
    pub folder: String,
    pub summary: Option<String>,
    pub dtstart: i64,
    pub dtend: Option<i64>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub organizer: Option<String>,
    pub attendees: Option<String>,
    pub sequence: i32,
    pub status: String,
    pub raw_ics: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub email: String,
    pub name: Option<String>,
    pub frequency: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub attachment_id: i64,
    pub uid: u32,
    pub folder: String,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub draft_id: Option<i64>,
    pub to_addr: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body: String,
    pub in_reply_to: Option<String>,
    pub ref_headers: Option<String>,
    pub reply_mode: String,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// Date parsing
// ---------------------------------------------------------------------------

/// Parse an email date string into a unix epoch timestamp for sorting.
/// Handles RFC 2822, RFC 3339, and common variants.
fn parse_date_to_epoch(date: Option<&str>) -> i64 {
    let date = match date {
        Some(d) if !d.is_empty() => d,
        _ => return 0,
    };

    // Try mailparse's dateparse first (handles RFC 2822 / email dates)
    if let Ok(epoch) = dateparse(date) {
        return epoch;
    }

    // Try RFC 3339 / ISO 8601 (from INTERNALDATE)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date) {
        return dt.timestamp();
    }

    // Try RFC 2822 via chrono
    if let Ok(dt) = DateTime::parse_from_rfc2822(date) {
        return dt.timestamp();
    }

    0
}

// ---------------------------------------------------------------------------
// Database
// ---------------------------------------------------------------------------

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) a SQLite database at `path`.
    /// Pass `:memory:` for an in-memory database (useful for tests).
    pub fn open(path: &str) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    // -----------------------------------------------------------------------
    // Migrations
    // -----------------------------------------------------------------------

    fn run_migrations(&self) -> Result<(), StoreError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS folders (
                name         TEXT PRIMARY KEY,
                uidvalidity  INTEGER,
                uidnext      INTEGER,
                last_sync    TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                uid          INTEGER NOT NULL,
                folder       TEXT    NOT NULL,
                message_id   TEXT,
                subject      TEXT,
                from_addr    TEXT,
                to_addr      TEXT,
                cc           TEXT,
                date         TEXT,
                date_epoch   INTEGER NOT NULL DEFAULT 0,
                body_text    TEXT,
                body_html    TEXT,
                flags        TEXT,
                thread_id    TEXT,
                ref_headers  TEXT,
                in_reply_to  TEXT,
                PRIMARY KEY (uid, folder)
            );

            CREATE TABLE IF NOT EXISTS threads (
                thread_id     TEXT PRIMARY KEY,
                subject       TEXT,
                last_date     TEXT,
                message_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS contacts (
                email     TEXT PRIMARY KEY,
                name      TEXT,
                frequency INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS labels (
                label_id  INTEGER PRIMARY KEY AUTOINCREMENT,
                name      TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS message_labels (
                uid       INTEGER NOT NULL,
                folder    TEXT    NOT NULL,
                label_id  INTEGER NOT NULL,
                PRIMARY KEY (uid, folder, label_id),
                FOREIGN KEY (uid, folder) REFERENCES messages(uid, folder) ON DELETE CASCADE,
                FOREIGN KEY (label_id) REFERENCES labels(label_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS attachments (
                attachment_id INTEGER PRIMARY KEY AUTOINCREMENT,
                uid           INTEGER NOT NULL,
                folder        TEXT    NOT NULL,
                filename      TEXT,
                mime_type     TEXT,
                size          INTEGER,
                content       BLOB,
                FOREIGN KEY (uid, folder) REFERENCES messages(uid, folder) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_folder    ON messages(folder);
            CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages(thread_id);
            CREATE INDEX IF NOT EXISTS idx_messages_date      ON messages(date_epoch DESC);

            CREATE TABLE IF NOT EXISTS events (
                event_uid   TEXT PRIMARY KEY,
                message_uid INTEGER,
                folder      TEXT,
                summary     TEXT,
                dtstart     INTEGER NOT NULL,
                dtend       INTEGER,
                location    TEXT,
                description TEXT,
                organizer   TEXT,
                attendees   TEXT,
                sequence    INTEGER NOT NULL DEFAULT 0,
                status      TEXT NOT NULL DEFAULT 'needs-action',
                raw_ics     TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_events_dtstart ON events(dtstart);

            CREATE TABLE IF NOT EXISTS drafts (
                draft_id    INTEGER PRIMARY KEY AUTOINCREMENT,
                to_addr     TEXT NOT NULL DEFAULT '',
                cc          TEXT NOT NULL DEFAULT '',
                bcc         TEXT NOT NULL DEFAULT '',
                subject     TEXT NOT NULL DEFAULT '',
                body        TEXT NOT NULL DEFAULT '',
                in_reply_to TEXT,
                ref_headers TEXT,
                reply_mode  TEXT NOT NULL DEFAULT 'compose',
                updated_at  INTEGER NOT NULL DEFAULT 0
            );
            ",
        )?;

        // Migration: add date_epoch column if missing (existing databases)
        let has_date_epoch: bool = self.conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name='date_epoch'")?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|c| c > 0)
            .unwrap_or(false);

        if !has_date_epoch {
            self.conn.execute_batch(
                "ALTER TABLE messages ADD COLUMN date_epoch INTEGER NOT NULL DEFAULT 0;"
            )?;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Message CRUD
    // -----------------------------------------------------------------------

    pub fn upsert_message(&self, msg: &Message) -> Result<(), StoreError> {
        let epoch = parse_date_to_epoch(msg.date.as_deref());
        self.conn.execute(
            "INSERT INTO messages
                (uid, folder, message_id, subject, from_addr, to_addr, cc,
                 date, date_epoch, body_text, body_html, flags, thread_id, ref_headers, in_reply_to)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)
             ON CONFLICT(uid, folder) DO UPDATE SET
                message_id  = excluded.message_id,
                subject     = excluded.subject,
                from_addr   = excluded.from_addr,
                to_addr     = excluded.to_addr,
                cc          = excluded.cc,
                date        = excluded.date,
                date_epoch  = excluded.date_epoch,
                body_text   = excluded.body_text,
                body_html   = excluded.body_html,
                flags       = excluded.flags,
                thread_id   = excluded.thread_id,
                ref_headers = excluded.ref_headers,
                in_reply_to = excluded.in_reply_to",
            params![
                msg.uid,
                msg.folder,
                msg.message_id,
                msg.subject,
                msg.from_addr,
                msg.to_addr,
                msg.cc,
                msg.date,
                epoch,
                msg.body_text,
                msg.body_html,
                msg.flags,
                msg.thread_id,
                msg.ref_headers,
                msg.in_reply_to,
            ],
        )?;
        Ok(())
    }

    pub fn get_message(&self, uid: u32, folder: &str) -> Result<Message, StoreError> {
        self.conn
            .query_row(
                "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc,
                        date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to
                 FROM messages WHERE uid = ?1 AND folder = ?2",
                params![uid, folder],
                |row| {
                    Ok(Message {
                        uid: row.get(0)?,
                        message_id: row.get(1)?,
                        folder: row.get(2)?,
                        subject: row.get(3)?,
                        from_addr: row.get(4)?,
                        to_addr: row.get(5)?,
                        cc: row.get(6)?,
                        date: row.get(7)?,
                        body_text: row.get(8)?,
                        body_html: row.get(9)?,
                        flags: row.get(10)?,
                        thread_id: row.get(11)?,
                        ref_headers: row.get(12)?,
                        in_reply_to: row.get(13)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
                other => StoreError::Db(other),
            })
    }

    pub fn get_messages_by_folder(&self, folder: &str) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc,
                    date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to
             FROM messages WHERE folder = ?1 ORDER BY date_epoch DESC",
        )?;
        let rows = stmt.query_map(params![folder], |row| {
            Ok(Message {
                uid: row.get(0)?,
                message_id: row.get(1)?,
                folder: row.get(2)?,
                subject: row.get(3)?,
                from_addr: row.get(4)?,
                to_addr: row.get(5)?,
                cc: row.get(6)?,
                date: row.get(7)?,
                body_text: row.get(8)?,
                body_html: row.get(9)?,
                flags: row.get(10)?,
                thread_id: row.get(11)?,
                ref_headers: row.get(12)?,
                in_reply_to: row.get(13)?,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn get_threads_by_folder(&self, folder: &str) -> Result<Vec<Thread>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT m.thread_id, m.subject, MAX(m.date) as last_date, COUNT(*) as cnt
             FROM messages m
             WHERE m.folder = ?1 AND m.thread_id IS NOT NULL
             GROUP BY m.thread_id
             ORDER BY last_date DESC",
        )?;
        let rows = stmt.query_map(params![folder], |row| {
            Ok(Thread {
                thread_id: row.get(0)?,
                subject: row.get(1)?,
                last_date: row.get(2)?,
                message_count: row.get(3)?,
            })
        })?;
        let mut threads = Vec::new();
        for row in rows {
            threads.push(row?);
        }
        Ok(threads)
    }

    pub fn get_thread_messages(&self, thread_id: &str) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc,
                    date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to
             FROM messages WHERE thread_id = ?1 ORDER BY date ASC",
        )?;
        let rows = stmt.query_map(params![thread_id], |row| {
            Ok(Message {
                uid: row.get(0)?,
                message_id: row.get(1)?,
                folder: row.get(2)?,
                subject: row.get(3)?,
                from_addr: row.get(4)?,
                to_addr: row.get(5)?,
                cc: row.get(6)?,
                date: row.get(7)?,
                body_text: row.get(8)?,
                body_html: row.get(9)?,
                flags: row.get(10)?,
                thread_id: row.get(11)?,
                ref_headers: row.get(12)?,
                in_reply_to: row.get(13)?,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn delete_message(&self, uid: u32, folder: &str) -> Result<bool, StoreError> {
        let rows = self.conn.execute(
            "DELETE FROM messages WHERE uid = ?1 AND folder = ?2",
            params![uid, folder],
        )?;
        Ok(rows > 0)
    }

    pub fn update_flags(&self, uid: u32, folder: &str, flags: &str) -> Result<(), StoreError> {
        let rows = self.conn.execute(
            "UPDATE messages SET flags = ?1 WHERE uid = ?2 AND folder = ?3",
            params![flags, uid, folder],
        )?;
        if rows == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Folder CRUD
    // -----------------------------------------------------------------------

    pub fn upsert_folder(&self, folder: &Folder) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO folders (name, uidvalidity, uidnext, last_sync)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(name) DO UPDATE SET
                uidvalidity = excluded.uidvalidity,
                uidnext     = excluded.uidnext,
                last_sync   = excluded.last_sync",
            params![folder.name, folder.uidvalidity, folder.uidnext, folder.last_sync],
        )?;
        Ok(())
    }

    pub fn get_unread_count(&self, folder: &str) -> Result<i64, StoreError> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE folder = ?1 AND flags NOT LIKE '%Seen%'",
                params![folder],
                |row| row.get(0),
            )
            .map_err(StoreError::Db)
    }

    pub fn get_folder_counts(&self) -> Result<Vec<(String, i64, i64)>, StoreError> {
        // Returns (folder_name, total_count, unread_count)
        let mut stmt = self.conn.prepare(
            "SELECT folder, COUNT(*), SUM(CASE WHEN flags NOT LIKE '%Seen%' THEN 1 ELSE 0 END)
             FROM messages GROUP BY folder ORDER BY folder",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_folders(&self) -> Result<Vec<Folder>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, uidvalidity, uidnext, last_sync FROM folders ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Folder {
                name: row.get(0)?,
                uidvalidity: row.get(1)?,
                uidnext: row.get(2)?,
                last_sync: row.get(3)?,
            })
        })?;
        let mut folders = Vec::new();
        for row in rows {
            folders.push(row?);
        }
        Ok(folders)
    }

    // -----------------------------------------------------------------------
    // Contact CRUD
    // -----------------------------------------------------------------------

    pub fn search_contacts(&self, query: &str, limit: usize) -> Result<Vec<Contact>, StoreError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT email, name, frequency FROM contacts
             WHERE email LIKE ?1 OR name LIKE ?1
             ORDER BY frequency DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(Contact {
                email: row.get(0)?,
                name: row.get(1)?,
                frequency: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_all_contacts(&self) -> Result<Vec<Contact>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT email, name, frequency FROM contacts ORDER BY frequency DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Contact {
                email: row.get(0)?,
                name: row.get(1)?,
                frequency: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_contact(&self, email: &str, name: Option<&str>) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO contacts (email, name, frequency)
             VALUES (?1, ?2, 1)
             ON CONFLICT(email) DO UPDATE SET
                name      = COALESCE(excluded.name, contacts.name),
                frequency = contacts.frequency + 1",
            params![email, name],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Event CRUD
    // -----------------------------------------------------------------------

    pub fn upsert_event(
        &self,
        event: &crate::calendar::parser::CalendarEvent,
        message_uid: u32,
        folder: &str,
        status: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO events
                (event_uid, message_uid, folder, summary, dtstart, dtend,
                 location, description, organizer, attendees, sequence, status, raw_ics)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
             ON CONFLICT(event_uid) DO UPDATE SET
                message_uid = excluded.message_uid,
                folder      = excluded.folder,
                summary     = excluded.summary,
                dtstart     = excluded.dtstart,
                dtend       = excluded.dtend,
                location    = excluded.location,
                description = excluded.description,
                organizer   = excluded.organizer,
                attendees   = excluded.attendees,
                sequence    = excluded.sequence,
                status      = excluded.status,
                raw_ics     = excluded.raw_ics",
            params![
                event.event_uid,
                message_uid,
                folder,
                event.summary,
                event.dtstart,
                event.dtend,
                event.location,
                event.description,
                event.organizer,
                event.attendees,
                event.sequence,
                status,
                event.raw_ics,
            ],
        )?;
        Ok(())
    }

    pub fn get_events(&self) -> Result<Vec<StoredEvent>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT event_uid, message_uid, folder, summary, dtstart, dtend,
                    location, description, organizer, attendees, sequence, status, raw_ics
             FROM events ORDER BY dtstart ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(StoredEvent {
                event_uid: row.get(0)?,
                message_uid: row.get(1)?,
                folder: row.get(2)?,
                summary: row.get(3)?,
                dtstart: row.get(4)?,
                dtend: row.get(5)?,
                location: row.get(6)?,
                description: row.get(7)?,
                organizer: row.get(8)?,
                attendees: row.get(9)?,
                sequence: row.get(10)?,
                status: row.get(11)?,
                raw_ics: row.get(12)?,
            })
        })?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    pub fn get_events_in_range(
        &self,
        start: i64,
        end: i64,
    ) -> Result<Vec<StoredEvent>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT event_uid, message_uid, folder, summary, dtstart, dtend,
                    location, description, organizer, attendees, sequence, status, raw_ics
             FROM events WHERE dtstart >= ?1 AND dtstart <= ?2
             ORDER BY dtstart ASC",
        )?;
        let rows = stmt.query_map(params![start, end], |row| {
            Ok(StoredEvent {
                event_uid: row.get(0)?,
                message_uid: row.get(1)?,
                folder: row.get(2)?,
                summary: row.get(3)?,
                dtstart: row.get(4)?,
                dtend: row.get(5)?,
                location: row.get(6)?,
                description: row.get(7)?,
                organizer: row.get(8)?,
                attendees: row.get(9)?,
                sequence: row.get(10)?,
                status: row.get(11)?,
                raw_ics: row.get(12)?,
            })
        })?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    pub fn get_event(&self, event_uid: &str) -> Result<StoredEvent, StoreError> {
        self.conn
            .query_row(
                "SELECT event_uid, message_uid, folder, summary, dtstart, dtend,
                        location, description, organizer, attendees, sequence, status, raw_ics
                 FROM events WHERE event_uid = ?1",
                params![event_uid],
                |row| {
                    Ok(StoredEvent {
                        event_uid: row.get(0)?,
                        message_uid: row.get(1)?,
                        folder: row.get(2)?,
                        summary: row.get(3)?,
                        dtstart: row.get(4)?,
                        dtend: row.get(5)?,
                        location: row.get(6)?,
                        description: row.get(7)?,
                        organizer: row.get(8)?,
                        attendees: row.get(9)?,
                        sequence: row.get(10)?,
                        status: row.get(11)?,
                        raw_ics: row.get(12)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
                other => StoreError::Db(other),
            })
    }

    pub fn update_event_status(
        &self,
        event_uid: &str,
        status: &str,
    ) -> Result<(), StoreError> {
        let rows = self.conn.execute(
            "UPDATE events SET status = ?1 WHERE event_uid = ?2",
            params![status, event_uid],
        )?;
        if rows == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Attachment CRUD
    // -----------------------------------------------------------------------

    pub fn insert_attachment(
        &self,
        uid: u32,
        folder: &str,
        filename: Option<&str>,
        mime_type: Option<&str>,
        size: Option<i64>,
        content: &[u8],
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO attachments (uid, folder, filename, mime_type, size, content)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![uid, folder, filename, mime_type, size, content],
        )?;
        Ok(())
    }

    pub fn get_attachments_for_message(
        &self,
        uid: u32,
        folder: &str,
    ) -> Result<Vec<AttachmentInfo>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT attachment_id, uid, folder, filename, mime_type, size
             FROM attachments WHERE uid = ?1 AND folder = ?2",
        )?;
        let rows = stmt.query_map(params![uid, folder], |row| {
            Ok(AttachmentInfo {
                attachment_id: row.get(0)?,
                uid: row.get(1)?,
                folder: row.get(2)?,
                filename: row.get(3)?,
                mime_type: row.get(4)?,
                size: row.get(5)?,
            })
        })?;
        let mut attachments = Vec::new();
        for row in rows {
            attachments.push(row?);
        }
        Ok(attachments)
    }

    pub fn get_attachment_data(
        &self,
        attachment_id: i64,
    ) -> Result<(Vec<u8>, Option<String>), StoreError> {
        self.conn
            .query_row(
                "SELECT content, filename FROM attachments WHERE attachment_id = ?1",
                params![attachment_id],
                |row| {
                    let data: Vec<u8> = row.get(0)?;
                    let filename: Option<String> = row.get(1)?;
                    Ok((data, filename))
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
                other => StoreError::Db(other),
            })
    }

    pub fn delete_attachments_for_message(
        &self,
        uid: u32,
        folder: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM attachments WHERE uid = ?1 AND folder = ?2",
            params![uid, folder],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Draft CRUD
    // -----------------------------------------------------------------------

    /// Save a draft. If `draft.draft_id` is `None` or 0, inserts a new row
    /// and returns the new ID. Otherwise updates the existing row.
    pub fn save_draft(&self, draft: &Draft) -> Result<i64, StoreError> {
        let is_new = draft.draft_id.is_none() || draft.draft_id == Some(0);
        if is_new {
            self.conn.execute(
                "INSERT INTO drafts (to_addr, cc, bcc, subject, body, in_reply_to, ref_headers, reply_mode, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    draft.to_addr,
                    draft.cc,
                    draft.bcc,
                    draft.subject,
                    draft.body,
                    draft.in_reply_to,
                    draft.ref_headers,
                    draft.reply_mode,
                    draft.updated_at,
                ],
            )?;
            Ok(self.conn.last_insert_rowid())
        } else {
            let id = draft.draft_id.unwrap();
            self.conn.execute(
                "UPDATE drafts SET to_addr = ?1, cc = ?2, bcc = ?3, subject = ?4, body = ?5,
                    in_reply_to = ?6, ref_headers = ?7, reply_mode = ?8, updated_at = ?9
                 WHERE draft_id = ?10",
                params![
                    draft.to_addr,
                    draft.cc,
                    draft.bcc,
                    draft.subject,
                    draft.body,
                    draft.in_reply_to,
                    draft.ref_headers,
                    draft.reply_mode,
                    draft.updated_at,
                    id,
                ],
            )?;
            Ok(id)
        }
    }

    pub fn get_drafts(&self) -> Result<Vec<Draft>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT draft_id, to_addr, cc, bcc, subject, body, in_reply_to, ref_headers, reply_mode, updated_at
             FROM drafts ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Draft {
                draft_id: row.get(0)?,
                to_addr: row.get(1)?,
                cc: row.get(2)?,
                bcc: row.get(3)?,
                subject: row.get(4)?,
                body: row.get(5)?,
                in_reply_to: row.get(6)?,
                ref_headers: row.get(7)?,
                reply_mode: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;
        let mut drafts = Vec::new();
        for row in rows {
            drafts.push(row?);
        }
        Ok(drafts)
    }

    pub fn get_draft(&self, draft_id: i64) -> Result<Draft, StoreError> {
        self.conn
            .query_row(
                "SELECT draft_id, to_addr, cc, bcc, subject, body, in_reply_to, ref_headers, reply_mode, updated_at
                 FROM drafts WHERE draft_id = ?1",
                params![draft_id],
                |row| {
                    Ok(Draft {
                        draft_id: row.get(0)?,
                        to_addr: row.get(1)?,
                        cc: row.get(2)?,
                        bcc: row.get(3)?,
                        subject: row.get(4)?,
                        body: row.get(5)?,
                        in_reply_to: row.get(6)?,
                        ref_headers: row.get(7)?,
                        reply_mode: row.get(8)?,
                        updated_at: row.get(9)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
                other => StoreError::Db(other),
            })
    }

    pub fn delete_draft(&self, draft_id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM drafts WHERE draft_id = ?1",
            params![draft_id],
        )?;
        Ok(())
    }

    pub fn get_events_for_message(
        &self,
        message_uid: u32,
        folder: &str,
    ) -> Result<Vec<StoredEvent>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT event_uid, message_uid, folder, summary, dtstart, dtend,
                    location, description, organizer, attendees, sequence, status, raw_ics
             FROM events WHERE message_uid = ?1 AND folder = ?2
             ORDER BY dtstart ASC",
        )?;
        let rows = stmt.query_map(params![message_uid, folder], |row| {
            Ok(StoredEvent {
                event_uid: row.get(0)?,
                message_uid: row.get(1)?,
                folder: row.get(2)?,
                summary: row.get(3)?,
                dtstart: row.get(4)?,
                dtend: row.get(5)?,
                location: row.get(6)?,
                description: row.get(7)?,
                organizer: row.get(8)?,
                attendees: row.get(9)?,
                sequence: row.get(10)?,
                status: row.get(11)?,
                raw_ics: row.get(12)?,
            })
        })?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        Database::open(":memory:").expect("failed to open in-memory db")
    }

    fn sample_message(uid: u32, folder: &str) -> Message {
        Message {
            uid,
            message_id: Some(format!("<msg-{}@example.com>", uid)),
            folder: folder.to_string(),
            subject: Some("Test subject".to_string()),
            from_addr: Some("alice@example.com".to_string()),
            to_addr: Some("bob@example.com".to_string()),
            cc: None,
            date: Some("2026-01-15T10:00:00Z".to_string()),
            body_text: Some("Hello".to_string()),
            body_html: None,
            flags: Some("\\Seen".to_string()),
            thread_id: Some("thread-1".to_string()),
            ref_headers: None,
            in_reply_to: None,
        }
    }

    #[test]
    fn test_create_and_get_message() {
        let db = test_db();
        let msg = sample_message(1, "INBOX");
        db.upsert_message(&msg).unwrap();

        let fetched = db.get_message(1, "INBOX").unwrap();
        assert_eq!(fetched.uid, 1);
        assert_eq!(fetched.folder, "INBOX");
        assert_eq!(fetched.subject.as_deref(), Some("Test subject"));
        assert_eq!(fetched.from_addr.as_deref(), Some("alice@example.com"));
    }

    #[test]
    fn test_folder_crud() {
        let db = test_db();
        let folder = Folder {
            name: "INBOX".to_string(),
            uidvalidity: Some(12345),
            uidnext: Some(100),
            last_sync: Some("2026-01-15T10:00:00Z".to_string()),
        };
        db.upsert_folder(&folder).unwrap();

        let folders = db.get_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "INBOX");
        assert_eq!(folders[0].uidvalidity, Some(12345));

        // Update
        let updated = Folder {
            name: "INBOX".to_string(),
            uidvalidity: Some(12345),
            uidnext: Some(200),
            last_sync: Some("2026-01-16T10:00:00Z".to_string()),
        };
        db.upsert_folder(&updated).unwrap();
        let folders = db.get_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].uidnext, Some(200));
    }

    #[test]
    fn test_messages_by_folder() {
        let db = test_db();

        let mut msg1 = sample_message(1, "INBOX");
        msg1.date = Some("2026-01-15T10:00:00Z".to_string());
        let mut msg2 = sample_message(2, "INBOX");
        msg2.date = Some("2026-01-16T10:00:00Z".to_string());
        let msg3 = sample_message(3, "Sent");

        db.upsert_message(&msg1).unwrap();
        db.upsert_message(&msg2).unwrap();
        db.upsert_message(&msg3).unwrap();

        let inbox = db.get_messages_by_folder("INBOX").unwrap();
        assert_eq!(inbox.len(), 2);
        // Ordered by date DESC — msg2 first
        assert_eq!(inbox[0].uid, 2);
        assert_eq!(inbox[1].uid, 1);

        let sent = db.get_messages_by_folder("Sent").unwrap();
        assert_eq!(sent.len(), 1);
    }

    #[test]
    fn test_delete_message() {
        let db = test_db();
        let msg = sample_message(1, "INBOX");
        db.upsert_message(&msg).unwrap();

        let deleted = db.delete_message(1, "INBOX").unwrap();
        assert!(deleted);

        let result = db.get_message(1, "INBOX");
        assert!(matches!(result, Err(StoreError::NotFound)));

        // Deleting non-existent returns false
        let deleted = db.delete_message(99, "INBOX").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_contact_upsert_frequency_increment() {
        let db = test_db();

        db.upsert_contact("alice@example.com", Some("Alice")).unwrap();

        // Check frequency is 1
        let freq: i64 = db
            .conn
            .query_row(
                "SELECT frequency FROM contacts WHERE email = ?1",
                params!["alice@example.com"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(freq, 1);

        // Upsert again — frequency should increment
        db.upsert_contact("alice@example.com", Some("Alice")).unwrap();
        let freq: i64 = db
            .conn
            .query_row(
                "SELECT frequency FROM contacts WHERE email = ?1",
                params!["alice@example.com"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(freq, 2);

        // Third time
        db.upsert_contact("alice@example.com", None).unwrap();
        let freq: i64 = db
            .conn
            .query_row(
                "SELECT frequency FROM contacts WHERE email = ?1",
                params!["alice@example.com"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(freq, 3);

        // Name should still be Alice (COALESCE keeps existing when new is NULL)
        let name: Option<String> = db
            .conn
            .query_row(
                "SELECT name FROM contacts WHERE email = ?1",
                params!["alice@example.com"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name.as_deref(), Some("Alice"));
    }
}
