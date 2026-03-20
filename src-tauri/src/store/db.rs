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
pub struct Contact {
    pub email: String,
    pub name: Option<String>,
    pub frequency: i64,
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
            CREATE INDEX IF NOT EXISTS idx_messages_date      ON messages(date);
            ",
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Message CRUD
    // -----------------------------------------------------------------------

    pub fn upsert_message(&self, msg: &Message) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO messages
                (uid, folder, message_id, subject, from_addr, to_addr, cc,
                 date, body_text, body_html, flags, thread_id, ref_headers, in_reply_to)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
             ON CONFLICT(uid, folder) DO UPDATE SET
                message_id  = excluded.message_id,
                subject     = excluded.subject,
                from_addr   = excluded.from_addr,
                to_addr     = excluded.to_addr,
                cc          = excluded.cc,
                date        = excluded.date,
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
             FROM messages WHERE folder = ?1 ORDER BY date DESC",
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
