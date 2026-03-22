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
    pub reply_to: Option<String>,
    pub list_unsubscribe: Option<String>,
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
pub struct Label {
    pub label_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id: Option<i64>,
    pub name: String,
    pub enabled: bool,
    pub conditions: String, // JSON
    pub actions: String,    // JSON
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEmail {
    pub schedule_id: Option<i64>,
    pub to_addr: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub ref_headers: Option<String>,
    pub send_at: i64,
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
                reply_to     TEXT,
                list_unsubscribe TEXT,
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

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS snoozed (
                uid         INTEGER NOT NULL,
                folder      TEXT NOT NULL,
                wake_at     INTEGER NOT NULL,
                PRIMARY KEY (uid, folder)
            );

            CREATE INDEX IF NOT EXISTS idx_snoozed_wake ON snoozed(wake_at);

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

            CREATE TABLE IF NOT EXISTS templates (
                template_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                body        TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS rules (
                rule_id     INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL DEFAULT '',
                enabled     INTEGER NOT NULL DEFAULT 1,
                conditions  TEXT NOT NULL DEFAULT '[]',
                actions     TEXT NOT NULL DEFAULT '[]',
                created_at  INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS scheduled (
                schedule_id INTEGER PRIMARY KEY AUTOINCREMENT,
                to_addr     TEXT NOT NULL,
                cc          TEXT NOT NULL DEFAULT '',
                bcc         TEXT NOT NULL DEFAULT '',
                subject     TEXT NOT NULL DEFAULT '',
                body_text   TEXT NOT NULL DEFAULT '',
                body_html   TEXT,
                in_reply_to TEXT,
                ref_headers TEXT,
                send_at     INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_scheduled_send_at ON scheduled(send_at);
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

        // Migration: add reply_to column if missing (existing databases)
        let has_reply_to: bool = self.conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name='reply_to'")?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|c| c > 0)
            .unwrap_or(false);

        if !has_reply_to {
            self.conn.execute_batch(
                "ALTER TABLE messages ADD COLUMN reply_to TEXT;"
            )?;
        }

        // Migration: add list_unsubscribe column if missing (existing databases)
        let has_list_unsubscribe: bool = self.conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name='list_unsubscribe'")?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|c| c > 0)
            .unwrap_or(false);

        if !has_list_unsubscribe {
            self.conn.execute_batch(
                "ALTER TABLE messages ADD COLUMN list_unsubscribe TEXT;"
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
                 date, date_epoch, body_text, body_html, flags, thread_id, ref_headers, in_reply_to, reply_to, list_unsubscribe)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
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
                in_reply_to = excluded.in_reply_to,
                reply_to    = excluded.reply_to,
                list_unsubscribe = excluded.list_unsubscribe",
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
                msg.reply_to,
                msg.list_unsubscribe,
            ],
        )?;
        Ok(())
    }

    pub fn get_message(&self, uid: u32, folder: &str) -> Result<Message, StoreError> {
        self.conn
            .query_row(
                "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc,
                        date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to, reply_to, list_unsubscribe
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
                        reply_to: row.get(14)?,
                        list_unsubscribe: row.get(15)?,
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
                    date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to, reply_to, list_unsubscribe
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
                reply_to: row.get(14)?,
                list_unsubscribe: row.get(15)?,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    /// Like `get_messages_by_folder` but also returns the `date_epoch` value
    /// for each message so callers can do cross-account sorting.
    pub fn get_messages_with_epoch(
        &self,
        folder: &str,
    ) -> Result<Vec<(Message, i64)>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc,
                    date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to, reply_to, list_unsubscribe,
                    date_epoch
             FROM messages WHERE folder = ?1 ORDER BY date_epoch DESC",
        )?;
        let rows = stmt.query_map(params![folder], |row| {
            let msg = Message {
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
                reply_to: row.get(14)?,
                list_unsubscribe: row.get(15)?,
            };
            let epoch: i64 = row.get(16)?;
            Ok((msg, epoch))
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn get_messages_by_folder_paged(&self, folder: &str, limit: i64, offset: i64) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc,
                    date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to, reply_to, list_unsubscribe
             FROM messages WHERE folder = ?1 ORDER BY date_epoch DESC LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(params![folder, limit, offset], |row| {
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
                reply_to: row.get(14)?,
                list_unsubscribe: row.get(15)?,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    /// Like `get_messages_by_folder_paged` but returns NULL for body_text and
    /// body_html, dramatically reducing memory and transfer time for the list view.
    pub fn get_messages_headers_paged(&self, folder: &str, limit: i64, offset: i64) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc,
                    date, NULL as body_text, NULL as body_html, flags, thread_id, ref_headers, in_reply_to, reply_to, list_unsubscribe
             FROM messages WHERE folder = ?1 ORDER BY date_epoch DESC LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(params![folder, limit, offset], |row| {
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
                reply_to: row.get(14)?,
                list_unsubscribe: row.get(15)?,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn get_message_count(&self, folder: &str) -> Result<i64, StoreError> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE folder = ?1",
            params![folder],
            |row| row.get(0),
        ).map_err(StoreError::Db)
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
                    date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to, reply_to, list_unsubscribe
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
                reply_to: row.get(14)?,
                list_unsubscribe: row.get(15)?,
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

    // -----------------------------------------------------------------------
    // Settings CRUD
    // -----------------------------------------------------------------------

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        match self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StoreError::Db(e)),
        }
    }

    // -----------------------------------------------------------------------
    // Snooze CRUD
    // -----------------------------------------------------------------------

    pub fn snooze_message(&self, uid: u32, folder: &str, wake_at: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO snoozed (uid, folder, wake_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(uid, folder) DO UPDATE SET wake_at = excluded.wake_at",
            params![uid, folder, wake_at],
        )?;
        Ok(())
    }

    pub fn get_due_snoozed(&self, now: i64) -> Result<Vec<(u32, String)>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, folder FROM snoozed WHERE wake_at <= ?1",
        )?;
        let rows = stmt.query_map(params![now], |row| {
            Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn unsnooze(&self, uid: u32, folder: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM snoozed WHERE uid = ?1 AND folder = ?2",
            params![uid, folder],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Label CRUD
    // -----------------------------------------------------------------------

    pub fn create_label(&self, name: &str, _color: &str) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO labels (name) VALUES (?1)",
            params![name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_labels(&self) -> Result<Vec<Label>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT label_id, name FROM labels ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Label {
                label_id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        let mut labels = Vec::new();
        for row in rows {
            labels.push(row?);
        }
        Ok(labels)
    }

    pub fn delete_label(&self, label_id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM message_labels WHERE label_id = ?1",
            params![label_id],
        )?;
        self.conn.execute(
            "DELETE FROM labels WHERE label_id = ?1",
            params![label_id],
        )?;
        Ok(())
    }

    pub fn add_label_to_message(&self, uid: u32, folder: &str, label_id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO message_labels (uid, folder, label_id) VALUES (?1, ?2, ?3)",
            params![uid, folder, label_id],
        )?;
        Ok(())
    }

    pub fn remove_label_from_message(&self, uid: u32, folder: &str, label_id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM message_labels WHERE uid = ?1 AND folder = ?2 AND label_id = ?3",
            params![uid, folder, label_id],
        )?;
        Ok(())
    }

    pub fn get_labels_for_message(&self, uid: u32, folder: &str) -> Result<Vec<Label>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT l.label_id, l.name
             FROM labels l
             JOIN message_labels ml ON l.label_id = ml.label_id
             WHERE ml.uid = ?1 AND ml.folder = ?2
             ORDER BY l.name",
        )?;
        let rows = stmt.query_map(params![uid, folder], |row| {
            Ok(Label {
                label_id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        let mut labels = Vec::new();
        for row in rows {
            labels.push(row?);
        }
        Ok(labels)
    }

    pub fn get_messages_by_label(&self, label_id: i64) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT m.uid, m.message_id, m.folder, m.subject, m.from_addr, m.to_addr, m.cc,
                    m.date, m.body_text, m.body_html, m.flags, m.thread_id, m.ref_headers, m.in_reply_to, m.reply_to, m.list_unsubscribe
             FROM messages m
             JOIN message_labels ml ON m.uid = ml.uid AND m.folder = ml.folder
             WHERE ml.label_id = ?1
             ORDER BY m.date_epoch DESC",
        )?;
        let rows = stmt.query_map(params![label_id], |row| {
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
                reply_to: row.get(14)?,
                list_unsubscribe: row.get(15)?,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn is_snoozed(&self, uid: u32, folder: &str) -> bool {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM snoozed WHERE uid = ?1 AND folder = ?2",
                params![uid, folder],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Template CRUD
    // -----------------------------------------------------------------------

    pub fn save_template(&self, name: &str, body: &str) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO templates (name, body) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET body = excluded.body",
            params![name, body],
        )?;
        let id = self.conn.query_row(
            "SELECT template_id FROM templates WHERE name = ?1",
            params![name],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(id)
    }

    pub fn get_template(&self, name: &str) -> Result<(i64, String, String), StoreError> {
        self.conn
            .query_row(
                "SELECT template_id, name, body FROM templates WHERE name = ?1",
                params![name],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
                other => StoreError::Db(other),
            })
    }

    pub fn get_templates(&self) -> Result<Vec<(i64, String, String)>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT template_id, name, body FROM templates ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut templates = Vec::new();
        for row in rows {
            templates.push(row?);
        }
        Ok(templates)
    }

    pub fn delete_template(&self, name: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM templates WHERE name = ?1",
            params![name],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Rule CRUD
    // -----------------------------------------------------------------------

    /// Save a rule. If `rule.rule_id` is `None` or 0, inserts a new row
    /// and returns the new ID. Otherwise updates the existing row.
    pub fn save_rule(&self, rule: &Rule) -> Result<i64, StoreError> {
        let is_new = rule.rule_id.is_none() || rule.rule_id == Some(0);
        if is_new {
            self.conn.execute(
                "INSERT INTO rules (name, enabled, conditions, actions, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    rule.name,
                    rule.enabled as i32,
                    rule.conditions,
                    rule.actions,
                    rule.created_at,
                ],
            )?;
            Ok(self.conn.last_insert_rowid())
        } else {
            let id = rule.rule_id.unwrap();
            self.conn.execute(
                "UPDATE rules SET name = ?1, enabled = ?2, conditions = ?3, actions = ?4, created_at = ?5
                 WHERE rule_id = ?6",
                params![
                    rule.name,
                    rule.enabled as i32,
                    rule.conditions,
                    rule.actions,
                    rule.created_at,
                    id,
                ],
            )?;
            Ok(id)
        }
    }

    pub fn get_rules(&self) -> Result<Vec<Rule>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT rule_id, name, enabled, conditions, actions, created_at
             FROM rules ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Rule {
                rule_id: row.get(0)?,
                name: row.get(1)?,
                enabled: row.get::<_, i32>(2)? != 0,
                conditions: row.get(3)?,
                actions: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        Ok(rules)
    }

    pub fn get_rule(&self, rule_id: i64) -> Result<Rule, StoreError> {
        self.conn
            .query_row(
                "SELECT rule_id, name, enabled, conditions, actions, created_at
                 FROM rules WHERE rule_id = ?1",
                params![rule_id],
                |row| {
                    Ok(Rule {
                        rule_id: row.get(0)?,
                        name: row.get(1)?,
                        enabled: row.get::<_, i32>(2)? != 0,
                        conditions: row.get(3)?,
                        actions: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
                other => StoreError::Db(other),
            })
    }

    pub fn delete_rule(&self, rule_id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM rules WHERE rule_id = ?1",
            params![rule_id],
        )?;
        Ok(())
    }

    pub fn toggle_rule(&self, rule_id: i64, enabled: bool) -> Result<(), StoreError> {
        let rows = self.conn.execute(
            "UPDATE rules SET enabled = ?1 WHERE rule_id = ?2",
            params![enabled as i32, rule_id],
        )?;
        if rows == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    pub fn get_enabled_rules(&self) -> Result<Vec<Rule>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT rule_id, name, enabled, conditions, actions, created_at
             FROM rules WHERE enabled = 1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Rule {
                rule_id: row.get(0)?,
                name: row.get(1)?,
                enabled: row.get::<_, i32>(2)? != 0,
                conditions: row.get(3)?,
                actions: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        Ok(rules)
    }

    // -----------------------------------------------------------------------
    // Scheduled Email CRUD
    // -----------------------------------------------------------------------

    pub fn schedule_email(&self, email: &ScheduledEmail) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO scheduled (to_addr, cc, bcc, subject, body_text, body_html, in_reply_to, ref_headers, send_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                email.to_addr,
                email.cc,
                email.bcc,
                email.subject,
                email.body_text,
                email.body_html,
                email.in_reply_to,
                email.ref_headers,
                email.send_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_due_scheduled(&self, now: i64) -> Result<Vec<ScheduledEmail>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT schedule_id, to_addr, cc, bcc, subject, body_text, body_html, in_reply_to, ref_headers, send_at
             FROM scheduled WHERE send_at <= ?1 ORDER BY send_at ASC",
        )?;
        let rows = stmt.query_map(params![now], |row| {
            Ok(ScheduledEmail {
                schedule_id: row.get(0)?,
                to_addr: row.get(1)?,
                cc: row.get(2)?,
                bcc: row.get(3)?,
                subject: row.get(4)?,
                body_text: row.get(5)?,
                body_html: row.get(6)?,
                in_reply_to: row.get(7)?,
                ref_headers: row.get(8)?,
                send_at: row.get(9)?,
            })
        })?;
        let mut emails = Vec::new();
        for row in rows {
            emails.push(row?);
        }
        Ok(emails)
    }

    pub fn delete_scheduled(&self, schedule_id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM scheduled WHERE schedule_id = ?1",
            params![schedule_id],
        )?;
        Ok(())
    }

    pub fn get_scheduled(&self) -> Result<Vec<ScheduledEmail>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT schedule_id, to_addr, cc, bcc, subject, body_text, body_html, in_reply_to, ref_headers, send_at
             FROM scheduled ORDER BY send_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ScheduledEmail {
                schedule_id: row.get(0)?,
                to_addr: row.get(1)?,
                cc: row.get(2)?,
                bcc: row.get(3)?,
                subject: row.get(4)?,
                body_text: row.get(5)?,
                body_html: row.get(6)?,
                in_reply_to: row.get(7)?,
                ref_headers: row.get(8)?,
                send_at: row.get(9)?,
            })
        })?;
        let mut emails = Vec::new();
        for row in rows {
            emails.push(row?);
        }
        Ok(emails)
    }

    /// Create a backup of the database using VACUUM INTO.
    pub fn backup(&self, backup_path: &str) -> Result<(), StoreError> {
        self.conn.execute_batch(&format!(
            "VACUUM INTO '{}'",
            backup_path.replace('\'', "''")
        ))?;
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
            reply_to: None,
            list_unsubscribe: None,
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
    fn test_events_crud() {
        let db = test_db();
        // Need a message first for foreign-key-like semantics
        let msg = sample_message(1, "INBOX");
        db.upsert_message(&msg).unwrap();

        let event = crate::calendar::parser::CalendarEvent {
            event_uid: "evt-1@example.com".to_string(),
            summary: Some("Team Meeting".to_string()),
            dtstart: 1774432800,
            dtend: Some(1774436400),
            location: Some("Room A".to_string()),
            description: Some("Weekly sync".to_string()),
            organizer: Some("alice@example.com".to_string()),
            attendees: "[]".to_string(),
            sequence: 0,
            method: Some("REQUEST".to_string()),
            raw_ics: "BEGIN:VCALENDAR...".to_string(),
        };

        db.upsert_event(&event, 1, "INBOX", "needs-action").unwrap();

        // get_events
        let events = db.get_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_uid, "evt-1@example.com");
        assert_eq!(events[0].summary.as_deref(), Some("Team Meeting"));
        assert_eq!(events[0].status, "needs-action");

        // get_event
        let ev = db.get_event("evt-1@example.com").unwrap();
        assert_eq!(ev.dtstart, 1774432800);
        assert_eq!(ev.location.as_deref(), Some("Room A"));

        // update_event_status
        db.update_event_status("evt-1@example.com", "accepted").unwrap();
        let ev = db.get_event("evt-1@example.com").unwrap();
        assert_eq!(ev.status, "accepted");

        // get_event for non-existent returns NotFound
        let result = db.get_event("nonexistent");
        assert!(matches!(result, Err(StoreError::NotFound)));
    }

    #[test]
    fn test_labels_crud() {
        let db = test_db();
        let msg = sample_message(1, "INBOX");
        db.upsert_message(&msg).unwrap();

        // Create label
        let label_id = db.create_label("Important", "").unwrap();
        assert!(label_id > 0);

        // Get labels
        let labels = db.get_labels().unwrap();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name, "Important");

        // Add label to message
        db.add_label_to_message(1, "INBOX", label_id).unwrap();

        // Get labels for message
        let msg_labels = db.get_labels_for_message(1, "INBOX").unwrap();
        assert_eq!(msg_labels.len(), 1);
        assert_eq!(msg_labels[0].name, "Important");

        // Get messages by label
        let msgs = db.get_messages_by_label(label_id).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].uid, 1);

        // Remove label from message
        db.remove_label_from_message(1, "INBOX", label_id).unwrap();
        let msg_labels = db.get_labels_for_message(1, "INBOX").unwrap();
        assert_eq!(msg_labels.len(), 0);

        // Messages by label should now be empty
        let msgs = db.get_messages_by_label(label_id).unwrap();
        assert_eq!(msgs.len(), 0);
    }

    #[test]
    fn test_drafts_crud() {
        let db = test_db();

        let draft = Draft {
            draft_id: None,
            to_addr: "bob@example.com".to_string(),
            cc: "".to_string(),
            bcc: "".to_string(),
            subject: "Test Draft".to_string(),
            body: "Draft body".to_string(),
            in_reply_to: None,
            ref_headers: None,
            reply_mode: "compose".to_string(),
            updated_at: 1000,
        };

        // Save draft
        let id = db.save_draft(&draft).unwrap();
        assert!(id > 0);

        // Get drafts
        let drafts = db.get_drafts().unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].subject, "Test Draft");

        // Get draft by id
        let d = db.get_draft(id).unwrap();
        assert_eq!(d.to_addr, "bob@example.com");
        assert_eq!(d.body, "Draft body");

        // Update draft
        let updated = Draft {
            draft_id: Some(id),
            to_addr: "carol@example.com".to_string(),
            cc: "".to_string(),
            bcc: "".to_string(),
            subject: "Updated Draft".to_string(),
            body: "Updated body".to_string(),
            in_reply_to: None,
            ref_headers: None,
            reply_mode: "compose".to_string(),
            updated_at: 2000,
        };
        let same_id = db.save_draft(&updated).unwrap();
        assert_eq!(same_id, id);

        let d = db.get_draft(id).unwrap();
        assert_eq!(d.subject, "Updated Draft");

        // Delete draft
        db.delete_draft(id).unwrap();
        let result = db.get_draft(id);
        assert!(matches!(result, Err(StoreError::NotFound)));
    }

    #[test]
    fn test_scheduled_crud() {
        let db = test_db();

        let email = ScheduledEmail {
            schedule_id: None,
            to_addr: "bob@example.com".to_string(),
            cc: "".to_string(),
            bcc: "".to_string(),
            subject: "Scheduled".to_string(),
            body_text: "Hello".to_string(),
            body_html: None,
            in_reply_to: None,
            ref_headers: None,
            send_at: 5000,
        };

        let id = db.schedule_email(&email).unwrap();
        assert!(id > 0);

        // Not yet due
        let due = db.get_due_scheduled(4999).unwrap();
        assert_eq!(due.len(), 0);

        // Now due
        let due = db.get_due_scheduled(5000).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].subject, "Scheduled");

        // Get all scheduled
        let all = db.get_scheduled().unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        db.delete_scheduled(id).unwrap();
        let all = db.get_scheduled().unwrap();
        assert_eq!(all.len(), 0);
    }

    #[test]
    fn test_rules_crud() {
        let db = test_db();

        let rule = Rule {
            rule_id: None,
            name: "Auto-label".to_string(),
            enabled: true,
            conditions: r#"[{"field":"from","op":"contains","value":"alice"}]"#.to_string(),
            actions: r#"[{"type":"add_label","value":"Alice"}]"#.to_string(),
            created_at: 1000,
        };

        // Save rule
        let id = db.save_rule(&rule).unwrap();
        assert!(id > 0);

        // Get rules
        let rules = db.get_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Auto-label");
        assert!(rules[0].enabled);

        // Toggle rule off
        db.toggle_rule(id, false).unwrap();
        let r = db.get_rule(id).unwrap();
        assert!(!r.enabled);

        // Enabled rules should be empty
        let enabled = db.get_enabled_rules().unwrap();
        assert_eq!(enabled.len(), 0);

        // Toggle back on
        db.toggle_rule(id, true).unwrap();
        let enabled = db.get_enabled_rules().unwrap();
        assert_eq!(enabled.len(), 1);

        // Delete rule
        db.delete_rule(id).unwrap();
        let rules = db.get_rules().unwrap();
        assert_eq!(rules.len(), 0);
    }

    #[test]
    fn test_pagination() {
        let db = test_db();

        // Insert 10 messages with different dates for ordering
        for i in 1..=10u32 {
            let mut msg = sample_message(i, "INBOX");
            msg.date = Some(format!("2026-01-{:02}T10:00:00Z", i));
            msg.subject = Some(format!("Message {}", i));
            db.upsert_message(&msg).unwrap();
        }

        // Total count
        let count = db.get_message_count("INBOX").unwrap();
        assert_eq!(count, 10);

        // Page 1: limit=3, offset=0
        let page1 = db.get_messages_by_folder_paged("INBOX", 3, 0).unwrap();
        assert_eq!(page1.len(), 3);
        // Ordered by date DESC, so message 10, 9, 8
        assert_eq!(page1[0].uid, 10);
        assert_eq!(page1[1].uid, 9);
        assert_eq!(page1[2].uid, 8);

        // Page 2: limit=3, offset=3
        let page2 = db.get_messages_by_folder_paged("INBOX", 3, 3).unwrap();
        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].uid, 7);

        // Page 4: limit=3, offset=9 — only 1 message left
        let page4 = db.get_messages_by_folder_paged("INBOX", 3, 9).unwrap();
        assert_eq!(page4.len(), 1);
        assert_eq!(page4[0].uid, 1);

        // Beyond range
        let empty = db.get_messages_by_folder_paged("INBOX", 3, 10).unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_settings() {
        let db = test_db();

        // Initially no setting
        let val = db.get_setting("theme").unwrap();
        assert!(val.is_none());

        // Set
        db.set_setting("theme", "dark").unwrap();
        let val = db.get_setting("theme").unwrap();
        assert_eq!(val.as_deref(), Some("dark"));

        // Overwrite
        db.set_setting("theme", "light").unwrap();
        let val = db.get_setting("theme").unwrap();
        assert_eq!(val.as_deref(), Some("light"));
    }

    #[test]
    fn test_backup() {
        let db = test_db();
        let msg = sample_message(1, "INBOX");
        db.upsert_message(&msg).unwrap();

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let backup_path = tmp.path().to_str().unwrap().to_string();
        // Remove the temp file so VACUUM INTO can create it
        drop(tmp);

        db.backup(&backup_path).unwrap();

        // Open the backup and verify data
        let backup_db = Database::open(&backup_path).unwrap();
        let fetched = backup_db.get_message(1, "INBOX").unwrap();
        assert_eq!(fetched.uid, 1);
        assert_eq!(fetched.subject.as_deref(), Some("Test subject"));

        // Cleanup
        let _ = std::fs::remove_file(&backup_path);
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
