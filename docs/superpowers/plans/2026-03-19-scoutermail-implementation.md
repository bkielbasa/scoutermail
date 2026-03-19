# ScouterMail Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a lightweight, vim-driven desktop email client for macOS using Tauri + Svelte + Rust.

**Architecture:** Rust backend handles IMAP/SMTP, email parsing, SQLite storage, and Tantivy search. Svelte frontend provides a minimal dark-themed two-pane UI with vim keybindings. Tauri IPC bridges them. Each account gets its own SQLite database.

**Tech Stack:** Rust, Tauri v2, Svelte 5, SQLite (rusqlite), Tantivy, async-imap, mailparse, lettre (SMTP), security-framework (macOS Keychain)

---

## File Structure

```
src-tauri/
  Cargo.toml
  tauri.conf.json
  src/
    main.rs                   — Tauri app entry point
    commands.rs               — All Tauri IPC command handlers
    lib.rs                    — Module declarations
    store/
      mod.rs                  — Storage module root
      db.rs                   — SQLite schema, migrations, CRUD operations
      search.rs               — Tantivy index management + query
    imap/
      mod.rs                  — IMAP module root
      client.rs               — IMAP connection, auth, folder listing
      sync.rs                 — Sync engine: headers-first, background body fetch
      idle.rs                 — IMAP IDLE push listener
    smtp/
      mod.rs                  — SMTP module root
      client.rs               — SMTP connection, send, TLS
    parser/
      mod.rs                  — Parser module root
      mime.rs                 — MIME parsing: multipart, attachments, inline images
      threading.rs            — RFC 5256 threading + subject fallback
    accounts/
      mod.rs                  — Account module root
      manager.rs              — Account CRUD, config, multi-account state
      keychain.rs             — macOS Keychain credential storage
frontend/
  package.json
  svelte.config.js
  vite.config.ts
  src/
    main.ts                   — Svelte app mount
    App.svelte                — Root layout: status bar, two-pane, hint bar
    app.css                   — Global dark theme styles
    lib/
      stores/
        accounts.ts           — Account list, active account store
        messages.ts           — Message list, selected message, threads store
        ui.ts                 — UI state: mode, focus pane, search open, etc.
      keybindings/
        engine.ts             — Vim keymap engine: modes, key sequences, dispatch
        bindings.ts           — Default binding definitions
      components/
        StatusBar.svelte      — Top bar: account:folder, mode, help
        HintBar.svelte        — Bottom bar: contextual hints, command input
        MessageList.svelte    — Left pane: message/thread list
        ReadingPane.svelte    — Right pane: email view with thread
        ComposeView.svelte    — Compose overlay replacing reading pane
        AccountSetup.svelte   — First-run / add account form
        SearchBar.svelte      — Search input overlay
        HelpOverlay.svelte    — ? keybinding reference overlay
```

---

### Task 1: Project Scaffold — Tauri + Svelte

Remove old Go code, initialize Tauri v2 + Svelte 5 project, verify it builds and launches.

**Files:**
- Delete: `app.go`, `email.go`, `go.mod`, `go.sum`, `frontend/` (old)
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `frontend/package.json`, `frontend/svelte.config.js`, `frontend/vite.config.ts`, `frontend/src/main.ts`, `frontend/src/App.svelte`, `frontend/src/app.css`

- [ ] **Step 1: Remove old Go code**

```bash
rm -f app.go email.go go.mod go.sum
rm -rf frontend/
```

- [ ] **Step 2: Initialize Tauri v2 project with Svelte**

```bash
npm create tauri-app@latest . -- --template svelte-ts --manager npm
```

If the interactive prompt doesn't support `--template`, use:

```bash
# Initialize manually
cargo install create-tauri-app
cargo create-tauri-app . --template svelte-ts
```

- [ ] **Step 3: Verify project structure exists**

Confirm these files exist: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`, `frontend/package.json`, `frontend/src/App.svelte`.

- [ ] **Step 4: Add Rust dependencies to `src-tauri/Cargo.toml`**

Add under `[dependencies]`:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.31", features = ["bundled"] }
tantivy = "0.22"
async-imap = "0.10"
async-native-tls = "0.5"
mailparse = "0.15"
lettre = { version = "0.11", features = ["tokio1-native-tls"] }
security-framework = "2"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
log = "0.4"
env_logger = "0.11"
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 5: Set up `src-tauri/src/lib.rs` with module declarations**

```rust
pub mod accounts;
pub mod commands;
pub mod imap;
pub mod parser;
pub mod smtp;
pub mod store;
```

- [ ] **Step 6: Create empty module files**

Create directory structure and `mod.rs` for each module:
- `src-tauri/src/store/mod.rs`
- `src-tauri/src/imap/mod.rs`
- `src-tauri/src/smtp/mod.rs`
- `src-tauri/src/parser/mod.rs`
- `src-tauri/src/accounts/mod.rs`
- `src-tauri/src/commands.rs`

Each `mod.rs` is empty for now. `commands.rs` is empty.

- [ ] **Step 7: Replace `frontend/src/App.svelte` with dark shell**

```svelte
<main>
  <p>ScouterMail</p>
</main>

<style>
  :global(body) {
    margin: 0;
    background: #0d0d14;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  }
  main {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
  }
</style>
```

- [ ] **Step 8: Build and launch**

```bash
cd frontend && npm install && cd ..
cd src-tauri && cargo build 2>&1 | tail -5
cd .. && npm run tauri dev
```

Expected: App window opens with dark background and "ScouterMail" text.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "scaffold: initialize Tauri v2 + Svelte 5 project, remove old Go code"
```

---

### Task 2: SQLite Storage Layer

Schema creation, migrations, and CRUD operations for messages, threads, folders, contacts, labels, attachments.

**Files:**
- Create: `src-tauri/src/store/db.rs`
- Modify: `src-tauri/src/store/mod.rs`

- [ ] **Step 1: Write `src-tauri/src/store/db.rs` — schema and Database struct**

```rust
use rusqlite::{Connection, Result, params};
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub uid: i64,
    pub message_id: String,
    pub folder: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub date: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub flags: String,
    pub thread_id: Option<String>,
    pub references: String,
    pub in_reply_to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thread {
    pub thread_id: String,
    pub subject: String,
    pub last_date: String,
    pub message_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub name: String,
    pub uidvalidity: i64,
    pub uidnext: i64,
    pub last_sync: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS folders (
                name TEXT PRIMARY KEY,
                uidvalidity INTEGER NOT NULL DEFAULT 0,
                uidnext INTEGER NOT NULL DEFAULT 0,
                last_sync TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                uid INTEGER NOT NULL,
                message_id TEXT NOT NULL DEFAULT '',
                folder TEXT NOT NULL,
                subject TEXT NOT NULL DEFAULT '',
                from_addr TEXT NOT NULL DEFAULT '',
                to_addr TEXT NOT NULL DEFAULT '',
                cc TEXT NOT NULL DEFAULT '',
                date TEXT NOT NULL DEFAULT '',
                body_text TEXT,
                body_html TEXT,
                flags TEXT NOT NULL DEFAULT '',
                thread_id TEXT,
                ref_headers TEXT NOT NULL DEFAULT '',
                in_reply_to TEXT NOT NULL DEFAULT '',
                PRIMARY KEY (uid, folder)
            );

            CREATE TABLE IF NOT EXISTS threads (
                thread_id TEXT PRIMARY KEY,
                subject TEXT NOT NULL DEFAULT '',
                last_date TEXT NOT NULL DEFAULT '',
                message_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS contacts (
                email TEXT PRIMARY KEY,
                name TEXT NOT NULL DEFAULT '',
                last_seen TEXT NOT NULL DEFAULT '',
                frequency INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS labels (
                label_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#7c3aed'
            );

            CREATE TABLE IF NOT EXISTS message_labels (
                message_uid INTEGER NOT NULL,
                folder TEXT NOT NULL,
                label_id INTEGER NOT NULL,
                PRIMARY KEY (message_uid, folder, label_id),
                FOREIGN KEY (label_id) REFERENCES labels(label_id)
            );

            CREATE TABLE IF NOT EXISTS attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_uid INTEGER NOT NULL,
                folder TEXT NOT NULL,
                filename TEXT NOT NULL DEFAULT '',
                content_type TEXT NOT NULL DEFAULT '',
                size INTEGER NOT NULL DEFAULT 0,
                data BLOB
            );

            CREATE INDEX IF NOT EXISTS idx_messages_folder ON messages(folder);
            CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id);
            CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date DESC);
        ")?;
        Ok(())
    }

    pub fn upsert_message(&self, msg: &Message) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO messages
             (uid, message_id, folder, subject, from_addr, to_addr, cc, date,
              body_text, body_html, flags, thread_id, ref_headers, in_reply_to)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                msg.uid, msg.message_id, msg.folder, msg.subject, msg.from,
                msg.to, msg.cc, msg.date, msg.body_text, msg.body_html,
                msg.flags, msg.thread_id, msg.references, msg.in_reply_to,
            ],
        )?;
        Ok(())
    }

    pub fn get_messages_by_folder(&self, folder: &str) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc, date,
                    body_text, body_html, flags, thread_id, ref_headers, in_reply_to
             FROM messages WHERE folder = ?1 ORDER BY date DESC"
        )?;
        let rows = stmt.query_map(params![folder], |row| {
            Ok(Message {
                uid: row.get(0)?,
                message_id: row.get(1)?,
                folder: row.get(2)?,
                subject: row.get(3)?,
                from: row.get(4)?,
                to: row.get(5)?,
                cc: row.get(6)?,
                date: row.get(7)?,
                body_text: row.get(8)?,
                body_html: row.get(9)?,
                flags: row.get(10)?,
                thread_id: row.get(11)?,
                references: row.get(12)?,
                in_reply_to: row.get(13)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_message(&self, uid: i64, folder: &str) -> Result<Message, StoreError> {
        self.conn.query_row(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc, date,
                    body_text, body_html, flags, thread_id, ref_headers, in_reply_to
             FROM messages WHERE uid = ?1 AND folder = ?2",
            params![uid, folder],
            |row| {
                Ok(Message {
                    uid: row.get(0)?,
                    message_id: row.get(1)?,
                    folder: row.get(2)?,
                    subject: row.get(3)?,
                    from: row.get(4)?,
                    to: row.get(5)?,
                    cc: row.get(6)?,
                    date: row.get(7)?,
                    body_text: row.get(8)?,
                    body_html: row.get(9)?,
                    flags: row.get(10)?,
                    thread_id: row.get(11)?,
                    references: row.get(12)?,
                    in_reply_to: row.get(13)?,
                })
            },
        ).map_err(|_| StoreError::NotFound(format!("message uid={} folder={}", uid, folder)))
    }

    pub fn get_threads_by_folder(&self, folder: &str) -> Result<Vec<Thread>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.thread_id, t.subject, t.last_date, t.message_count
             FROM threads t
             WHERE t.thread_id IN (SELECT DISTINCT thread_id FROM messages WHERE folder = ?1)
             ORDER BY t.last_date DESC"
        )?;
        let rows = stmt.query_map(params![folder], |row| {
            Ok(Thread {
                thread_id: row.get(0)?,
                subject: row.get(1)?,
                last_date: row.get(2)?,
                message_count: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_thread_messages(&self, thread_id: &str) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT uid, message_id, folder, subject, from_addr, to_addr, cc, date,
                    body_text, body_html, flags, thread_id, ref_headers, in_reply_to
             FROM messages WHERE thread_id = ?1 ORDER BY date ASC"
        )?;
        let rows = stmt.query_map(params![thread_id], |row| {
            Ok(Message {
                uid: row.get(0)?,
                message_id: row.get(1)?,
                folder: row.get(2)?,
                subject: row.get(3)?,
                from: row.get(4)?,
                to: row.get(5)?,
                cc: row.get(6)?,
                date: row.get(7)?,
                body_text: row.get(8)?,
                body_html: row.get(9)?,
                flags: row.get(10)?,
                thread_id: row.get(11)?,
                references: row.get(12)?,
                in_reply_to: row.get(13)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_folder(&self, folder: &Folder) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO folders (name, uidvalidity, uidnext, last_sync)
             VALUES (?1, ?2, ?3, ?4)",
            params![folder.name, folder.uidvalidity, folder.uidnext, folder.last_sync],
        )?;
        Ok(())
    }

    pub fn get_folders(&self) -> Result<Vec<Folder>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT name, uidvalidity, uidnext, last_sync FROM folders")?;
        let rows = stmt.query_map([], |row| {
            Ok(Folder {
                name: row.get(0)?,
                uidvalidity: row.get(1)?,
                uidnext: row.get(2)?,
                last_sync: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_message(&self, uid: i64, folder: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM messages WHERE uid = ?1 AND folder = ?2",
            params![uid, folder],
        )?;
        Ok(())
    }

    pub fn update_flags(&self, uid: i64, folder: &str, flags: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE messages SET flags = ?1 WHERE uid = ?2 AND folder = ?3",
            params![flags, uid, folder],
        )?;
        Ok(())
    }

    pub fn upsert_contact(&self, email: &str, name: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO contacts (email, name, last_seen, frequency)
             VALUES (?1, ?2, datetime('now'), 1)
             ON CONFLICT(email) DO UPDATE SET
               name = CASE WHEN ?2 != '' THEN ?2 ELSE contacts.name END,
               last_seen = datetime('now'),
               frequency = contacts.frequency + 1",
            params![email, name],
        )?;
        Ok(())
    }
}
```

- [ ] **Step 2: Update `src-tauri/src/store/mod.rs`**

```rust
pub mod db;
pub mod search;
```

Create empty `src-tauri/src/store/search.rs`:

```rust
// Tantivy search — implemented in Task 6
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

Expected: compiles with no errors (warnings about unused are fine).

- [ ] **Step 4: Write unit tests for Database**

Add to bottom of `src-tauri/src/store/db.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_db() -> Database {
        Database::open(Path::new(":memory:")).unwrap()
    }

    #[test]
    fn test_create_and_get_message() {
        let db = test_db();
        let msg = Message {
            uid: 1,
            message_id: "<test@example.com>".into(),
            folder: "INBOX".into(),
            subject: "Test Subject".into(),
            from: "alice@example.com".into(),
            to: "bob@example.com".into(),
            cc: "".into(),
            date: "2026-03-19T10:00:00Z".into(),
            body_text: Some("Hello world".into()),
            body_html: None,
            flags: "\\Seen".into(),
            thread_id: Some("thread-1".into()),
            references: "".into(),
            in_reply_to: "".into(),
        };
        db.upsert_message(&msg).unwrap();
        let fetched = db.get_message(1, "INBOX").unwrap();
        assert_eq!(fetched.subject, "Test Subject");
        assert_eq!(fetched.from, "alice@example.com");
    }

    #[test]
    fn test_folder_crud() {
        let db = test_db();
        let folder = Folder {
            name: "INBOX".into(),
            uidvalidity: 12345,
            uidnext: 100,
            last_sync: None,
        };
        db.upsert_folder(&folder).unwrap();
        let folders = db.get_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "INBOX");
    }

    #[test]
    fn test_messages_by_folder() {
        let db = test_db();
        for i in 1..=3 {
            let msg = Message {
                uid: i,
                message_id: format!("<test{}@example.com>", i),
                folder: "INBOX".into(),
                subject: format!("Subject {}", i),
                from: "alice@example.com".into(),
                to: "bob@example.com".into(),
                cc: "".into(),
                date: format!("2026-03-19T{:02}:00:00Z", i),
                body_text: None,
                body_html: None,
                flags: "".into(),
                thread_id: None,
                references: "".into(),
                in_reply_to: "".into(),
            };
            db.upsert_message(&msg).unwrap();
        }
        let msgs = db.get_messages_by_folder("INBOX").unwrap();
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn test_delete_message() {
        let db = test_db();
        let msg = Message {
            uid: 1,
            message_id: "<del@example.com>".into(),
            folder: "INBOX".into(),
            subject: "Delete me".into(),
            from: "a@b.com".into(),
            to: "c@d.com".into(),
            cc: "".into(),
            date: "2026-03-19T10:00:00Z".into(),
            body_text: None,
            body_html: None,
            flags: "".into(),
            thread_id: None,
            references: "".into(),
            in_reply_to: "".into(),
        };
        db.upsert_message(&msg).unwrap();
        db.delete_message(1, "INBOX").unwrap();
        assert!(db.get_message(1, "INBOX").is_err());
    }

    #[test]
    fn test_contact_upsert_increments_frequency() {
        let db = test_db();
        db.upsert_contact("alice@example.com", "Alice").unwrap();
        db.upsert_contact("alice@example.com", "Alice Smith").unwrap();
        let mut stmt = db.conn.prepare("SELECT name, frequency FROM contacts WHERE email = ?1").unwrap();
        let (name, freq): (String, i64) = stmt.query_row(params!["alice@example.com"], |r| {
            Ok((r.get(0)?, r.get(1)?))
        }).unwrap();
        assert_eq!(name, "Alice Smith");
        assert_eq!(freq, 2);
    }
}
```

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test -- --nocapture 2>&1 | tail -20
```

Expected: all 5 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/store/
git commit -m "feat: add SQLite storage layer with schema, CRUD, and tests"
```

---

### Task 3: Email Parser — MIME + Threading

Parse raw email into structured data. Build threading from References/In-Reply-To headers.

**Files:**
- Create: `src-tauri/src/parser/mime.rs`
- Create: `src-tauri/src/parser/threading.rs`
- Modify: `src-tauri/src/parser/mod.rs`

- [ ] **Step 1: Write `src-tauri/src/parser/mime.rs`**

```rust
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedEmail {
    pub message_id: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub date: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub references: Vec<String>,
    pub in_reply_to: String,
    pub attachments: Vec<Attachment>,
    pub inline_images: Vec<InlineImage>,
    pub raw_headers: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineImage {
    pub content_id: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

pub fn parse_email(raw: &[u8]) -> Result<ParsedEmail, String> {
    let parsed = parse_mail(raw).map_err(|e| e.to_string())?;

    let headers = &parsed.headers;
    let message_id = headers.get_first_value("Message-ID").unwrap_or_default();
    let subject = headers.get_first_value("Subject").unwrap_or_default();
    let from = headers.get_first_value("From").unwrap_or_default();
    let to = headers.get_first_value("To").unwrap_or_default();
    let cc = headers.get_first_value("Cc").unwrap_or_default();
    let date = headers.get_first_value("Date").unwrap_or_default();
    let in_reply_to = headers.get_first_value("In-Reply-To").unwrap_or_default();
    let references_str = headers.get_first_value("References").unwrap_or_default();
    let references: Vec<String> = references_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let raw_headers = headers.iter()
        .map(|h| format!("{}: {}", h.get_key(), h.get_value()))
        .collect::<Vec<_>>()
        .join("\n");

    let mut body_text = None;
    let mut body_html = None;
    let mut attachments = Vec::new();
    let mut inline_images = Vec::new();

    extract_parts(&parsed, &mut body_text, &mut body_html, &mut attachments, &mut inline_images);

    Ok(ParsedEmail {
        message_id,
        subject,
        from,
        to,
        cc,
        date,
        body_text,
        body_html,
        references,
        in_reply_to,
        attachments,
        inline_images,
        raw_headers,
    })
}

fn extract_parts(
    part: &ParsedMail,
    body_text: &mut Option<String>,
    body_html: &mut Option<String>,
    attachments: &mut Vec<Attachment>,
    inline_images: &mut Vec<InlineImage>,
) {
    let content_type = part.ctype.mimetype.as_str();

    if part.subparts.is_empty() {
        let content_disposition = part.headers.get_first_value("Content-Disposition")
            .unwrap_or_default();
        let content_id = part.headers.get_first_value("Content-ID")
            .unwrap_or_default()
            .trim_matches(|c| c == '<' || c == '>')
            .to_string();

        if content_disposition.starts_with("attachment") {
            let filename = part.ctype.params.get("name")
                .cloned()
                .unwrap_or_else(|| "unnamed".to_string());
            let data = part.get_body_raw().unwrap_or_default();
            attachments.push(Attachment {
                filename,
                content_type: content_type.to_string(),
                size: data.len(),
                data,
            });
        } else if content_type.starts_with("image/") && !content_id.is_empty() {
            let data = part.get_body_raw().unwrap_or_default();
            inline_images.push(InlineImage {
                content_id,
                content_type: content_type.to_string(),
                data,
            });
        } else if content_type == "text/plain" && body_text.is_none() {
            *body_text = part.get_body().ok();
        } else if content_type == "text/html" && body_html.is_none() {
            *body_html = part.get_body().ok();
        }
    } else {
        for subpart in &part.subparts {
            extract_parts(subpart, body_text, body_html, attachments, inline_images);
        }
    }
}

/// Resolve cid: references in HTML with base64 data URIs from inline images.
pub fn resolve_cid_images(html: &str, inline_images: &[InlineImage]) -> String {
    let mut result = html.to_string();
    for img in inline_images {
        let data_uri = format!(
            "data:{};base64,{}",
            img.content_type,
            base64_encode(&img.data)
        );
        result = result.replace(
            &format!("cid:{}", img.content_id),
            &data_uri,
        );
    }
    result
}

fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    let mut encoder = base64_writer(&mut buf);
    encoder.write_all(data).unwrap_or_default();
    drop(encoder);
    String::from_utf8(buf).unwrap_or_default()
}

// Simple base64 without extra dependency — can swap for `base64` crate if needed
fn base64_writer(output: &mut Vec<u8>) -> impl Write + '_ {
    struct B64Writer<'a>(&'a mut Vec<u8>, Vec<u8>);
    impl<'a> Write for B64Writer<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.1.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    impl<'a> Drop for B64Writer<'a> {
        fn drop(&mut self) {
            const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let data = &self.1;
            let mut i = 0;
            while i + 2 < data.len() {
                let n = ((data[i] as u32) << 16) | ((data[i+1] as u32) << 8) | (data[i+2] as u32);
                self.0.push(CHARS[((n >> 18) & 0x3F) as usize]);
                self.0.push(CHARS[((n >> 12) & 0x3F) as usize]);
                self.0.push(CHARS[((n >> 6) & 0x3F) as usize]);
                self.0.push(CHARS[(n & 0x3F) as usize]);
                i += 3;
            }
            let rem = data.len() - i;
            if rem == 1 {
                let n = (data[i] as u32) << 16;
                self.0.push(CHARS[((n >> 18) & 0x3F) as usize]);
                self.0.push(CHARS[((n >> 12) & 0x3F) as usize]);
                self.0.extend_from_slice(b"==");
            } else if rem == 2 {
                let n = ((data[i] as u32) << 16) | ((data[i+1] as u32) << 8);
                self.0.push(CHARS[((n >> 18) & 0x3F) as usize]);
                self.0.push(CHARS[((n >> 12) & 0x3F) as usize]);
                self.0.push(CHARS[((n >> 6) & 0x3F) as usize]);
                self.0.push(b'=');
            }
        }
    }
    B64Writer(output, Vec::new())
}
```

- [ ] **Step 2: Write `src-tauri/src/parser/threading.rs`**

```rust
use std::collections::HashMap;
use crate::store::db::Message;

/// Build thread_id assignments for a set of messages.
/// Uses References/In-Reply-To headers (RFC 5256 style).
/// Falls back to normalized subject grouping.
pub fn assign_threads(messages: &mut Vec<Message>) {
    // Map message_id -> thread_id
    let mut id_to_thread: HashMap<String, String> = HashMap::new();
    let mut subject_to_thread: HashMap<String, String> = HashMap::new();

    for msg in messages.iter_mut() {
        let mut thread_id: Option<String> = None;

        // Check References and In-Reply-To for existing thread
        let ref_ids: Vec<&str> = msg.references
            .split_whitespace()
            .chain(std::iter::once(msg.in_reply_to.as_str()))
            .filter(|s| !s.is_empty())
            .collect();

        for ref_id in &ref_ids {
            if let Some(tid) = id_to_thread.get(*ref_id) {
                thread_id = Some(tid.clone());
                break;
            }
        }

        // Fallback: normalized subject
        if thread_id.is_none() {
            let norm = normalize_subject(&msg.subject);
            if !norm.is_empty() {
                if let Some(tid) = subject_to_thread.get(&norm) {
                    thread_id = Some(tid.clone());
                }
            }
        }

        // New thread if nothing found
        let tid = thread_id.unwrap_or_else(|| {
            if !msg.message_id.is_empty() {
                msg.message_id.clone()
            } else {
                uuid::Uuid::new_v4().to_string()
            }
        });

        // Register this message's ID under the thread
        if !msg.message_id.is_empty() {
            id_to_thread.insert(msg.message_id.clone(), tid.clone());
        }
        for ref_id in &ref_ids {
            id_to_thread.insert(ref_id.to_string(), tid.clone());
        }

        let norm = normalize_subject(&msg.subject);
        if !norm.is_empty() {
            subject_to_thread.insert(norm, tid.clone());
        }

        msg.thread_id = Some(tid);
    }
}

/// Strip Re:, Fwd:, etc. and normalize for subject-based threading.
fn normalize_subject(subject: &str) -> String {
    let mut s = subject.trim().to_lowercase();
    loop {
        let before = s.clone();
        for prefix in &["re:", "fwd:", "fw:", "re[", "fwd["] {
            if s.starts_with(prefix) {
                // Handle re[2]: style
                if let Some(rest) = s.strip_prefix("re[").or_else(|| s.strip_prefix("fwd[")) {
                    if let Some(idx) = rest.find("]:") {
                        s = rest[idx + 2..].trim().to_string();
                        continue;
                    }
                }
                s = s[prefix.len()..].trim().to_string();
            }
        }
        if s == before {
            break;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(uid: i64, message_id: &str, subject: &str, refs: &str, in_reply_to: &str) -> Message {
        Message {
            uid,
            message_id: message_id.into(),
            folder: "INBOX".into(),
            subject: subject.into(),
            from: "test@example.com".into(),
            to: "other@example.com".into(),
            cc: "".into(),
            date: "2026-03-19T10:00:00Z".into(),
            body_text: None,
            body_html: None,
            flags: "".into(),
            thread_id: None,
            references: refs.into(),
            in_reply_to: in_reply_to.into(),
        }
    }

    #[test]
    fn test_thread_by_references() {
        let mut msgs = vec![
            make_msg(1, "<a@test>", "Hello", "", ""),
            make_msg(2, "<b@test>", "Re: Hello", "<a@test>", "<a@test>"),
            make_msg(3, "<c@test>", "Re: Re: Hello", "<a@test> <b@test>", "<b@test>"),
        ];
        assign_threads(&mut msgs);
        let t0 = msgs[0].thread_id.as_ref().unwrap();
        assert_eq!(msgs[1].thread_id.as_ref().unwrap(), t0);
        assert_eq!(msgs[2].thread_id.as_ref().unwrap(), t0);
    }

    #[test]
    fn test_thread_by_subject_fallback() {
        let mut msgs = vec![
            make_msg(1, "<x@test>", "Meeting notes", "", ""),
            make_msg(2, "<y@test>", "Re: Meeting notes", "", ""),
        ];
        assign_threads(&mut msgs);
        let t0 = msgs[0].thread_id.as_ref().unwrap();
        assert_eq!(msgs[1].thread_id.as_ref().unwrap(), t0);
    }

    #[test]
    fn test_normalize_subject() {
        assert_eq!(normalize_subject("Re: Hello"), "hello");
        assert_eq!(normalize_subject("Fwd: Re: Hello"), "hello");
        assert_eq!(normalize_subject("  RE: FW: test  "), "test");
    }

    #[test]
    fn test_unrelated_messages_get_different_threads() {
        let mut msgs = vec![
            make_msg(1, "<a@test>", "Topic A", "", ""),
            make_msg(2, "<b@test>", "Topic B", "", ""),
        ];
        assign_threads(&mut msgs);
        assert_ne!(
            msgs[0].thread_id.as_ref().unwrap(),
            msgs[1].thread_id.as_ref().unwrap()
        );
    }
}
```

- [ ] **Step 3: Update `src-tauri/src/parser/mod.rs`**

```rust
pub mod mime;
pub mod threading;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test -- --nocapture 2>&1 | tail -20
```

Expected: all threading + previous tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/parser/
git commit -m "feat: add email MIME parser and RFC 5256 threading engine"
```

---

### Task 4: IMAP Client — Connect, Sync, Idle

IMAP connection with TLS, folder listing, headers-first sync, and IDLE push support.

**Files:**
- Create: `src-tauri/src/imap/client.rs`
- Create: `src-tauri/src/imap/sync.rs`
- Create: `src-tauri/src/imap/idle.rs`
- Modify: `src-tauri/src/imap/mod.rs`

- [ ] **Step 1: Write `src-tauri/src/imap/client.rs`**

```rust
use async_imap::Session;
use async_native_tls::TlsStream;
use tokio::net::TcpStream;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImapError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("auth failed: {0}")]
    Auth(String),
    #[error("imap error: {0}")]
    Imap(String),
}

pub type ImapSession = Session<TlsStream<TcpStream>>;

pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub async fn connect(config: &ImapConfig) -> Result<ImapSession, ImapError> {
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
        .await
        .map_err(|e| ImapError::Connection(e.to_string()))?;

    let tls = async_native_tls::TlsConnector::new();
    let tls_stream = tls.connect(&config.host, tcp)
        .await
        .map_err(|e| ImapError::Connection(e.to_string()))?;

    let client = async_imap::Client::new(tls_stream);
    let session = client
        .login(&config.username, &config.password)
        .await
        .map_err(|(e, _)| ImapError::Auth(e.to_string()))?;

    Ok(session)
}

pub async fn list_folders(session: &mut ImapSession) -> Result<Vec<String>, ImapError> {
    let folders = session
        .list(Some(""), Some("*"))
        .await
        .map_err(|e| ImapError::Imap(e.to_string()))?;

    Ok(folders.iter().map(|f| f.name().to_string()).collect())
}
```

- [ ] **Step 2: Write `src-tauri/src/imap/sync.rs`**

```rust
use crate::imap::client::{ImapSession, ImapError};
use crate::parser::mime;
use crate::store::db::{Database, Message, Folder};
use crate::parser::threading;
use async_imap::types::Fetch;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Sync a single folder: fetch new message headers, then bodies in background.
pub async fn sync_folder(
    session: &mut ImapSession,
    db: &Database,
    folder_name: &str,
) -> Result<Vec<Message>, ImapError> {
    // Select the mailbox
    let mailbox = session
        .select(folder_name)
        .await
        .map_err(|e| ImapError::Imap(e.to_string()))?;

    let uidvalidity = mailbox.uid_validity.unwrap_or(0);
    let uidnext = mailbox.uid_next.unwrap_or(0);

    // Get our stored folder state
    let stored_folders = db.get_folders().map_err(|e| ImapError::Imap(e.to_string()))?;
    let stored = stored_folders.iter().find(|f| f.name == folder_name);
    let fetch_from = stored.map(|f| f.uidnext).unwrap_or(1);

    // If uidvalidity changed, we need a full resync (for now, just fetch all)
    let range = if stored.map(|f| f.uidvalidity).unwrap_or(0) != uidvalidity as i64 {
        "1:*".to_string()
    } else if fetch_from >= uidnext as i64 {
        // No new messages
        return Ok(vec![]);
    } else {
        format!("{}:*", fetch_from)
    };

    // Phase 1: Fetch headers (envelope data)
    let messages_stream = session
        .uid_fetch(&range, "(UID FLAGS BODY.PEEK[HEADER] BODY.PEEK[TEXT] INTERNALDATE)")
        .await
        .map_err(|e| ImapError::Imap(e.to_string()))?;

    let mut new_messages = Vec::new();
    for fetch in messages_stream.iter() {
        if let Some(msg) = parse_fetched(fetch, folder_name) {
            db.upsert_message(&msg).map_err(|e| ImapError::Imap(e.to_string()))?;

            // Auto-extract contacts from From header
            if !msg.from.is_empty() {
                let _ = db.upsert_contact(&msg.from, "");
            }

            new_messages.push(msg);
        }
    }

    // Assign threads
    let mut all_messages = db.get_messages_by_folder(folder_name)
        .map_err(|e| ImapError::Imap(e.to_string()))?;
    threading::assign_threads(&mut all_messages);
    for msg in &all_messages {
        db.upsert_message(msg).map_err(|e| ImapError::Imap(e.to_string()))?;
    }

    // Update folder sync state
    db.upsert_folder(&Folder {
        name: folder_name.to_string(),
        uidvalidity: uidvalidity as i64,
        uidnext: uidnext as i64,
        last_sync: Some(chrono::Utc::now().to_rfc3339()),
    }).map_err(|e| ImapError::Imap(e.to_string()))?;

    Ok(new_messages)
}

fn parse_fetched(fetch: &Fetch, folder: &str) -> Option<Message> {
    let uid = fetch.uid? as i64;
    let header_bytes = fetch.header()?;
    let body_bytes = fetch.text().unwrap_or(&[]);
    let flags = fetch.flags()
        .iter()
        .map(|f| format!("{:?}", f))
        .collect::<Vec<_>>()
        .join(" ");

    // Combine header + body for full parsing
    let mut raw = Vec::new();
    raw.extend_from_slice(header_bytes);
    raw.extend_from_slice(b"\r\n");
    raw.extend_from_slice(body_bytes);

    let parsed = mime::parse_email(&raw).ok()?;

    Some(Message {
        uid,
        message_id: parsed.message_id,
        folder: folder.to_string(),
        subject: parsed.subject,
        from: parsed.from,
        to: parsed.to,
        cc: parsed.cc,
        date: parsed.date,
        body_text: parsed.body_text,
        body_html: parsed.body_html,
        flags,
        thread_id: None, // assigned later by threading pass
        references: parsed.references.join(" "),
        in_reply_to: parsed.in_reply_to,
    })
}
```

- [ ] **Step 3: Write `src-tauri/src/imap/idle.rs`**

```rust
use crate::imap::client::{ImapSession, ImapError};
use std::time::Duration;

/// Start IDLE on a session. Returns when new mail arrives or timeout (29 min per RFC).
/// Caller should re-sync the folder after this returns.
pub async fn idle_wait(session: &mut ImapSession, folder: &str) -> Result<(), ImapError> {
    session
        .select(folder)
        .await
        .map_err(|e| ImapError::Imap(e.to_string()))?;

    let mut idle = session
        .idle();

    idle.init()
        .await
        .map_err(|e| ImapError::Imap(e.to_string()))?;

    // Wait for up to 29 minutes (IMAP IDLE spec recommends < 30 min)
    let _result = idle.wait_with_timeout(Duration::from_secs(29 * 60))
        .await;

    Ok(())
}
```

- [ ] **Step 4: Update `src-tauri/src/imap/mod.rs`**

```rust
pub mod client;
pub mod sync;
pub mod idle;
```

- [ ] **Step 5: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

Expected: compiles (no integration tests here — requires live IMAP server).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/imap/
git commit -m "feat: add IMAP client with sync engine and IDLE support"
```

---

### Task 5: SMTP Client — Send Email

Send emails via SMTP with TLS. Supports reply, reply-all, forward, compose.

**Files:**
- Create: `src-tauri/src/smtp/client.rs`
- Modify: `src-tauri/src/smtp/mod.rs`

- [ ] **Step 1: Write `src-tauri/src/smtp/client.rs`**

```rust
use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, MultiPart, SinglePart, Mailbox},
    transport::smtp::authentication::Credentials,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmtpError {
    #[error("smtp error: {0}")]
    Send(String),
    #[error("invalid address: {0}")]
    Address(String),
}

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub struct ComposeEmail {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

pub fn send_email(config: &SmtpConfig, email: &ComposeEmail) -> Result<(), SmtpError> {
    let from_mailbox: Mailbox = email.from.parse()
        .map_err(|e: lettre::address::AddressError| SmtpError::Address(e.to_string()))?;

    let mut builder = Message::builder()
        .from(from_mailbox)
        .subject(&email.subject);

    for addr in &email.to {
        let mb: Mailbox = addr.parse()
            .map_err(|e: lettre::address::AddressError| SmtpError::Address(e.to_string()))?;
        builder = builder.to(mb);
    }
    for addr in &email.cc {
        let mb: Mailbox = addr.parse()
            .map_err(|e: lettre::address::AddressError| SmtpError::Address(e.to_string()))?;
        builder = builder.cc(mb);
    }
    for addr in &email.bcc {
        let mb: Mailbox = addr.parse()
            .map_err(|e: lettre::address::AddressError| SmtpError::Address(e.to_string()))?;
        builder = builder.bcc(mb);
    }

    if let Some(ref reply_to) = email.in_reply_to {
        builder = builder.in_reply_to(reply_to.clone());
    }
    if !email.references.is_empty() {
        builder = builder.references(email.references.join(" "));
    }

    let message = if let Some(ref html) = email.body_html {
        builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::builder()
                        .content_type(ContentType::TEXT_PLAIN)
                        .body(email.body_text.clone()))
                    .singlepart(SinglePart::builder()
                        .content_type(ContentType::TEXT_HTML)
                        .body(html.clone()))
            )
            .map_err(|e| SmtpError::Send(e.to_string()))?
    } else {
        builder
            .body(email.body_text.clone())
            .map_err(|e| SmtpError::Send(e.to_string()))?
    };

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let mailer = SmtpTransport::starttls_relay(&config.host)
        .map_err(|e| SmtpError::Send(e.to_string()))?
        .port(config.port)
        .credentials(creds)
        .build();

    mailer.send(&message).map_err(|e| SmtpError::Send(e.to_string()))?;
    Ok(())
}
```

- [ ] **Step 2: Update `src-tauri/src/smtp/mod.rs`**

```rust
pub mod client;
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/smtp/
git commit -m "feat: add SMTP client for sending email with TLS"
```

---

### Task 6: Tantivy Full-Text Search

Index messages on sync, query with structured syntax (from:, subject:, body text).

**Files:**
- Modify: `src-tauri/src/store/search.rs`

- [ ] **Step 1: Write `src-tauri/src/store/search.rs`**

```rust
use tantivy::{
    schema::{Schema, TEXT, STORED, STRING},
    Index, IndexWriter, IndexReader,
    collector::TopDocs,
    query::QueryParser,
    doc,
    TantivyError,
};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("search error: {0}")]
    Tantivy(#[from] TantivyError),
    #[error("query parse error: {0}")]
    Query(String),
}

#[derive(Debug)]
pub struct SearchResult {
    pub uid: i64,
    pub folder: String,
    pub score: f32,
}

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    schema: Schema,
}

impl SearchIndex {
    pub fn open(path: &Path) -> Result<Self, SearchError> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_i64_field("uid", tantivy::schema::INDEXED | tantivy::schema::STORED);
        schema_builder.add_text_field("folder", STRING | STORED);
        schema_builder.add_text_field("subject", TEXT | STORED);
        schema_builder.add_text_field("from", TEXT | STORED);
        schema_builder.add_text_field("to", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();

        std::fs::create_dir_all(path).ok();
        let index = Index::create_in_dir(path, schema.clone())
            .or_else(|_| Index::open_in_dir(path))?;
        let reader = index.reader()?;

        Ok(Self { index, reader, schema })
    }

    pub fn writer(&self) -> Result<IndexWriter, SearchError> {
        Ok(self.index.writer(50_000_000)?) // 50MB heap
    }

    pub fn index_message(
        &self,
        writer: &mut IndexWriter,
        uid: i64,
        folder: &str,
        subject: &str,
        from: &str,
        to: &str,
        body: &str,
    ) -> Result<(), SearchError> {
        let uid_field = self.schema.get_field("uid").unwrap();
        let folder_field = self.schema.get_field("folder").unwrap();
        let subject_field = self.schema.get_field("subject").unwrap();
        let from_field = self.schema.get_field("from").unwrap();
        let to_field = self.schema.get_field("to").unwrap();
        let body_field = self.schema.get_field("body").unwrap();

        writer.add_document(doc!(
            uid_field => uid,
            folder_field => folder,
            subject_field => subject,
            from_field => from,
            to_field => to,
            body_field => body,
        ))?;

        Ok(())
    }

    pub fn commit(&self, writer: &mut IndexWriter) -> Result<(), SearchError> {
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let searcher = self.reader.searcher();

        let subject_field = self.schema.get_field("subject").unwrap();
        let from_field = self.schema.get_field("from").unwrap();
        let to_field = self.schema.get_field("to").unwrap();
        let body_field = self.schema.get_field("body").unwrap();

        let query_parser = QueryParser::for_index(
            &self.index,
            vec![subject_field, from_field, to_field, body_field],
        );

        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| SearchError::Query(e.to_string()))?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let uid_field = self.schema.get_field("uid").unwrap();
        let folder_field = self.schema.get_field("folder").unwrap();

        let mut results = Vec::new();
        for (score, doc_addr) in top_docs {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_addr)?;
            let uid = doc.get_first(uid_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let folder = doc.get_first(folder_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            results.push(SearchResult { uid, folder, score });
        }

        Ok(results)
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

- [ ] **Step 3: Write test**

Add to bottom of `search.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_index_and_search() {
        let dir = tempdir().unwrap();
        let idx = SearchIndex::open(dir.path()).unwrap();
        let mut writer = idx.writer().unwrap();

        idx.index_message(&mut writer, 1, "INBOX", "Project update", "alice@test.com", "bob@test.com", "Let's push the deadline").unwrap();
        idx.index_message(&mut writer, 2, "INBOX", "Meeting notes", "bob@test.com", "alice@test.com", "Here are the notes from today").unwrap();

        idx.commit(&mut writer).unwrap();

        let results = idx.search("deadline", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid, 1);

        let results = idx.search("from:alice", 10).unwrap();
        assert_eq!(results.len(), 1);
    }
}
```

Add `tempfile = "3"` to `[dev-dependencies]` in `Cargo.toml`.

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test -- --nocapture 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/store/search.rs src-tauri/Cargo.toml
git commit -m "feat: add Tantivy full-text search index with query support"
```

---

### Task 7: Account Manager + macOS Keychain

Multi-account management with secure credential storage.

**Files:**
- Create: `src-tauri/src/accounts/manager.rs`
- Create: `src-tauri/src/accounts/keychain.rs`
- Modify: `src-tauri/src/accounts/mod.rs`

- [ ] **Step 1: Write `src-tauri/src/accounts/keychain.rs`**

```rust
use security_framework::passwords::{set_generic_password, get_generic_password, delete_generic_password};
use thiserror::Error;

const SERVICE_NAME: &str = "com.scoutermail.accounts";

#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("keychain error: {0}")]
    SecFramework(String),
}

pub fn store_password(account_id: &str, password: &str) -> Result<(), KeychainError> {
    set_generic_password(SERVICE_NAME, account_id, password.as_bytes())
        .map_err(|e| KeychainError::SecFramework(e.to_string()))
}

pub fn get_password(account_id: &str) -> Result<String, KeychainError> {
    let bytes = get_generic_password(SERVICE_NAME, account_id)
        .map_err(|e| KeychainError::SecFramework(e.to_string()))?;
    String::from_utf8(bytes)
        .map_err(|e| KeychainError::SecFramework(e.to_string()))
}

pub fn delete_password(account_id: &str) -> Result<(), KeychainError> {
    delete_generic_password(SERVICE_NAME, account_id)
        .map_err(|e| KeychainError::SecFramework(e.to_string()))
}
```

- [ ] **Step 2: Write `src-tauri/src/accounts/manager.rs`**

```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use crate::accounts::keychain;
use crate::store::db::Database;
use crate::store::search::SearchIndex;
use crate::imap::client::ImapConfig;
use crate::smtp::client::SmtpConfig;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("account not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("keychain error: {0}")]
    Keychain(#[from] keychain::KeychainError),
    #[error("store error: {0}")]
    Store(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
}

/// Known provider defaults for auto-fill
pub fn provider_defaults(provider: &str) -> Option<(&str, u16, &str, u16)> {
    match provider.to_lowercase().as_str() {
        "gmail" => Some(("imap.gmail.com", 993, "smtp.gmail.com", 587)),
        "outlook" | "hotmail" => Some(("outlook.office365.com", 993, "smtp.office365.com", 587)),
        "yahoo" => Some(("imap.mail.yahoo.com", 993, "smtp.mail.yahoo.com", 587)),
        _ => None,
    }
}

pub struct AccountManager {
    data_dir: PathBuf,
    accounts: Vec<AccountConfig>,
}

impl AccountManager {
    pub fn new(data_dir: &Path) -> Result<Self, AccountError> {
        fs::create_dir_all(data_dir)?;
        let config_path = data_dir.join("accounts.json");
        let accounts = if config_path.exists() {
            let data = fs::read_to_string(&config_path)?;
            serde_json::from_str(&data)?
        } else {
            Vec::new()
        };
        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            accounts,
        })
    }

    fn save(&self) -> Result<(), AccountError> {
        let config_path = self.data_dir.join("accounts.json");
        let data = serde_json::to_string_pretty(&self.accounts)?;
        fs::write(config_path, data)?;
        Ok(())
    }

    pub fn add_account(&mut self, config: AccountConfig, password: &str) -> Result<(), AccountError> {
        keychain::store_password(&config.id, password)?;

        // Create account data directory
        let acct_dir = self.data_dir.join(&config.id);
        fs::create_dir_all(&acct_dir)?;

        self.accounts.push(config);
        self.save()
    }

    pub fn remove_account(&mut self, id: &str) -> Result<(), AccountError> {
        keychain::delete_password(id).ok(); // ignore keychain errors on delete
        self.accounts.retain(|a| a.id != id);
        self.save()
    }

    pub fn list_accounts(&self) -> &[AccountConfig] {
        &self.accounts
    }

    pub fn get_account(&self, id: &str) -> Result<&AccountConfig, AccountError> {
        self.accounts.iter().find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))
    }

    pub fn get_imap_config(&self, id: &str) -> Result<ImapConfig, AccountError> {
        let acct = self.get_account(id)?;
        let password = keychain::get_password(id)?;
        Ok(ImapConfig {
            host: acct.imap_host.clone(),
            port: acct.imap_port,
            username: acct.username.clone(),
            password,
        })
    }

    pub fn get_smtp_config(&self, id: &str) -> Result<SmtpConfig, AccountError> {
        let acct = self.get_account(id)?;
        let password = keychain::get_password(id)?;
        Ok(SmtpConfig {
            host: acct.smtp_host.clone(),
            port: acct.smtp_port,
            username: acct.username.clone(),
            password,
        })
    }

    pub fn db_path(&self, id: &str) -> PathBuf {
        self.data_dir.join(id).join("mail.db")
    }

    pub fn search_index_path(&self, id: &str) -> PathBuf {
        self.data_dir.join(id).join("search_index")
    }
}
```

- [ ] **Step 3: Update `src-tauri/src/accounts/mod.rs`**

```rust
pub mod keychain;
pub mod manager;
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/accounts/
git commit -m "feat: add account manager with macOS Keychain credential storage"
```

---

### Task 8: Tauri IPC Commands

Bridge Rust backend to Svelte frontend via Tauri commands.

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write `src-tauri/src/commands.rs`**

```rust
use tauri::State;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::accounts::manager::{AccountManager, AccountConfig, AccountError, provider_defaults};
use crate::store::db::{Database, Message, Folder, StoreError};
use crate::store::search::{SearchIndex, SearchResult};
use crate::imap::client;
use crate::imap::sync as imap_sync;
use crate::smtp::client as smtp_client;
use std::path::Path;

pub struct AppState {
    pub account_manager: Arc<Mutex<AccountManager>>,
    pub active_account: Arc<Mutex<Option<String>>>,
}

// ----- Account commands -----

#[derive(Deserialize)]
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

#[tauri::command]
pub async fn add_account(
    state: State<'_, AppState>,
    req: AddAccountRequest,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
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
    mgr.add_account(config, &req.password).map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> Result<Vec<AccountConfig>, String> {
    let mgr = state.account_manager.lock().await;
    Ok(mgr.list_accounts().to_vec())
}

#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut mgr = state.account_manager.lock().await;
    mgr.remove_account(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_account(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut active = state.active_account.lock().await;
    *active = Some(id);
    Ok(())
}

#[tauri::command]
pub async fn get_provider_defaults(provider: String) -> Result<Option<(String, u16, String, u16)>, String> {
    Ok(provider_defaults(&provider).map(|(ih, ip, sh, sp)| {
        (ih.to_string(), ip, sh.to_string(), sp)
    }))
}

// ----- Test connection -----

#[tauri::command]
pub async fn test_imap_connection(
    host: String, port: u16, username: String, password: String,
) -> Result<Vec<String>, String> {
    let config = client::ImapConfig { host, port, username, password };
    let mut session = client::connect(&config).await.map_err(|e| e.to_string())?;
    let folders = client::list_folders(&mut session).await.map_err(|e| e.to_string())?;
    session.logout().await.ok();
    Ok(folders)
}

// ----- Mail commands -----

#[tauri::command]
pub async fn sync_folder(
    state: State<'_, AppState>,
    folder: String,
) -> Result<Vec<Message>, String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let imap_config = mgr.get_imap_config(account_id).map_err(|e| e.to_string())?;
    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;

    let mut session = client::connect(&imap_config).await.map_err(|e| e.to_string())?;
    let messages = imap_sync::sync_folder(&mut session, &db, &folder).await.map_err(|e| e.to_string())?;
    session.logout().await.ok();

    // Index for search
    let search_path = mgr.search_index_path(account_id);
    if let Ok(idx) = SearchIndex::open(&search_path) {
        if let Ok(mut writer) = idx.writer() {
            for msg in &messages {
                let _ = idx.index_message(
                    &mut writer,
                    msg.uid,
                    &msg.folder,
                    &msg.subject,
                    &msg.from,
                    &msg.to,
                    msg.body_text.as_deref().unwrap_or(""),
                );
            }
            let _ = idx.commit(&mut writer);
        }
    }

    Ok(messages)
}

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    folder: String,
) -> Result<Vec<Message>, String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    db.get_messages_by_folder(&folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_message(
    state: State<'_, AppState>,
    uid: i64,
    folder: String,
) -> Result<Message, String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    db.get_message(uid, &folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_thread_messages(
    state: State<'_, AppState>,
    thread_id: String,
) -> Result<Vec<Message>, String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    db.get_thread_messages(&thread_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_folders(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    db.get_folders().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_messages(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<Message>, String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let search_path = mgr.search_index_path(account_id);
    let idx = SearchIndex::open(&search_path).map_err(|e| e.to_string())?;
    let results = idx.search(&query, limit.unwrap_or(50)).map_err(|e| e.to_string())?;

    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for r in results {
        if let Ok(msg) = db.get_message(r.uid, &r.folder) {
            messages.push(msg);
        }
    }
    Ok(messages)
}

// ----- Send commands -----

#[derive(Deserialize)]
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

#[tauri::command]
pub async fn send_email(
    state: State<'_, AppState>,
    req: SendEmailRequest,
) -> Result<(), String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let acct = mgr.get_account(account_id).map_err(|e| e.to_string())?;
    let smtp_config = mgr.get_smtp_config(account_id).map_err(|e| e.to_string())?;

    let email = smtp_client::ComposeEmail {
        from: acct.email.clone(),
        to: req.to,
        cc: req.cc,
        bcc: req.bcc,
        subject: req.subject,
        body_text: req.body_text,
        body_html: req.body_html,
        in_reply_to: req.in_reply_to,
        references: req.references,
    };

    smtp_client::send_email(&smtp_config, &email).map_err(|e| e.to_string())
}

// ----- Flag commands -----

#[tauri::command]
pub async fn update_flags(
    state: State<'_, AppState>,
    uid: i64,
    folder: String,
    flags: String,
) -> Result<(), String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    db.update_flags(uid, &folder, &flags).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_message(
    state: State<'_, AppState>,
    uid: i64,
    folder: String,
) -> Result<(), String> {
    let mgr = state.account_manager.lock().await;
    let active = state.active_account.lock().await;
    let account_id = active.as_ref().ok_or("no active account")?;

    let db_path = mgr.db_path(account_id);
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    db.delete_message(uid, &folder).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Update `src-tauri/src/main.rs` to register commands and state**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;

mod accounts;
mod commands;
mod imap;
mod parser;
mod smtp;
mod store;

use commands::AppState;
use accounts::manager::AccountManager;

fn main() {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.scoutermail");

    let account_manager = AccountManager::new(&data_dir)
        .expect("Failed to initialize account manager");

    tauri::Builder::default()
        .manage(AppState {
            account_manager: Arc::new(Mutex::new(account_manager)),
            active_account: Arc::new(Mutex::new(None)),
        })
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
```

Add `dirs = "5"` to `[dependencies]` in `Cargo.toml`.

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "feat: add Tauri IPC commands bridging Rust backend to frontend"
```

---

### Task 9: Frontend — Dark Theme + App Shell

Global CSS, root layout with status bar, two-pane container, and hint bar.

**Files:**
- Modify: `frontend/src/app.css`
- Modify: `frontend/src/App.svelte`
- Create: `frontend/src/lib/components/StatusBar.svelte`
- Create: `frontend/src/lib/components/HintBar.svelte`
- Create: `frontend/src/lib/stores/ui.ts`

- [ ] **Step 1: Write `frontend/src/app.css` — global dark theme**

```css
:root {
  --bg-primary: #0d0d14;
  --bg-secondary: #111119;
  --bg-tertiary: #1a1a2e;
  --border: #1a1a2e;
  --accent: #7c3aed;
  --accent-dim: rgba(124, 58, 237, 0.15);
  --text-primary: #e0e0e0;
  --text-secondary: #888;
  --text-dim: #555;
  --text-muted: #333;
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
  --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: 13px;
  overflow: hidden;
  height: 100vh;
}

#app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

::-webkit-scrollbar {
  width: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--text-muted);
  border-radius: 3px;
}

::selection {
  background: var(--accent);
  color: white;
}
```

- [ ] **Step 2: Write `frontend/src/lib/stores/ui.ts`**

```typescript
import { writable } from 'svelte/store';

export type Mode = 'NORMAL' | 'INSERT' | 'VISUAL' | 'COMMAND';
export type FocusPane = 'list' | 'reading';

export const mode = writable<Mode>('NORMAL');
export const focusPane = writable<FocusPane>('list');
export const commandInput = writable('');
export const searchOpen = writable(false);
export const searchQuery = writable('');
export const helpOpen = writable(false);
```

- [ ] **Step 3: Write `frontend/src/lib/components/StatusBar.svelte`**

```svelte
<script lang="ts">
  import { mode } from '$lib/stores/ui';
  import { activeAccount, activeFolder } from '$lib/stores/accounts';

  $: accountName = $activeAccount?.name ?? 'No account';
  $: folderName = $activeFolder ?? 'INBOX';
</script>

<div class="status-bar">
  <span class="context">{accountName}:{folderName}</span>
  <span class="mode" class:normal={$mode === 'NORMAL'} class:insert={$mode === 'INSERT'} class:visual={$mode === 'VISUAL'} class:command={$mode === 'COMMAND'}>
    {$mode}
  </span>
  <span class="help">? help</span>
</div>

<style>
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 11px;
    height: 28px;
    flex-shrink: 0;
    user-select: none;
  }
  .context { color: var(--accent); }
  .mode { color: var(--text-dim); }
  .mode.normal { color: var(--text-dim); }
  .mode.insert { color: #22c55e; }
  .mode.visual { color: #f59e0b; }
  .mode.command { color: var(--accent); }
  .help { color: var(--text-dim); }
</style>
```

- [ ] **Step 4: Write `frontend/src/lib/components/HintBar.svelte`**

```svelte
<script lang="ts">
  import { mode, commandInput } from '$lib/stores/ui';

  const normalHints = 'j/k navigate · enter open · r reply · a archive · d delete · / search · ? help';
  const visualHints = 'j/k extend · a archive · d delete · esc cancel';
  const insertHints = 'esc normal · ctrl+enter send';

  $: hints = $mode === 'VISUAL' ? visualHints : $mode === 'INSERT' ? insertHints : normalHints;
</script>

<div class="hint-bar">
  {#if $mode === 'COMMAND'}
    <span class="command-prefix">:</span>
    <span class="command-text">{$commandInput}</span>
  {:else}
    <span class="hints">{hints}</span>
  {/if}
</div>

<style>
  .hint-bar {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 11px;
    height: 28px;
    flex-shrink: 0;
    user-select: none;
  }
  .hints { color: var(--text-dim); }
  .command-prefix { color: var(--accent); margin-right: 4px; }
  .command-text { color: var(--text-primary); }
</style>
```

- [ ] **Step 5: Write `frontend/src/lib/stores/accounts.ts`**

```typescript
import { writable } from 'svelte/store';

export interface Account {
  id: string;
  name: string;
  email: string;
  imap_host: string;
  imap_port: number;
  smtp_host: string;
  smtp_port: number;
  username: string;
}

export const accounts = writable<Account[]>([]);
export const activeAccount = writable<Account | null>(null);
export const activeFolder = writable<string>('INBOX');
```

- [ ] **Step 6: Update `frontend/src/App.svelte`**

```svelte
<script lang="ts">
  import StatusBar from '$lib/components/StatusBar.svelte';
  import HintBar from '$lib/components/HintBar.svelte';
</script>

<div id="app">
  <StatusBar />
  <main class="content">
    <div class="message-list-pane">
      <p style="color: var(--text-dim); padding: 12px;">Message list</p>
    </div>
    <div class="reading-pane">
      <p style="color: var(--text-dim); padding: 12px;">Reading pane</p>
    </div>
  </main>
  <HintBar />
</div>

<style>
  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .message-list-pane {
    width: 33%;
    min-width: 250px;
    max-width: 500px;
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }
  .reading-pane {
    flex: 1;
    overflow-y: auto;
  }
</style>
```

- [ ] **Step 7: Build and verify**

```bash
cd frontend && npm run build && cd ../src-tauri && cargo build 2>&1 | tail -5
```

Expected: builds successfully.

- [ ] **Step 8: Commit**

```bash
git add frontend/src/
git commit -m "feat: add dark theme app shell with status bar and hint bar"
```

---

### Task 10: Frontend — Vim Keybinding Engine

Modal keyboard input system with NORMAL, INSERT, VISUAL, COMMAND modes and multi-key sequences.

**Files:**
- Create: `frontend/src/lib/keybindings/engine.ts`
- Create: `frontend/src/lib/keybindings/bindings.ts`

- [ ] **Step 1: Write `frontend/src/lib/keybindings/engine.ts`**

```typescript
import { get } from 'svelte/store';
import { mode, commandInput, searchOpen, searchQuery, helpOpen } from '$lib/stores/ui';
import type { Mode } from '$lib/stores/ui';

export type Action = string;
export type KeyHandler = () => void;

interface Binding {
  keys: string;
  action: Action;
  mode: Mode;
}

let keyBuffer = '';
let keyTimeout: ReturnType<typeof setTimeout> | null = null;
const handlers: Map<Action, KeyHandler> = new Map();
let bindings: Binding[] = [];

export function registerHandler(action: Action, handler: KeyHandler) {
  handlers.set(action, handler);
}

export function setBindings(newBindings: Binding[]) {
  bindings = newBindings;
}

export function handleKeyDown(event: KeyboardEvent) {
  const currentMode = get(mode);

  // Command mode: capture into command input
  if (currentMode === 'COMMAND') {
    handleCommandMode(event);
    return;
  }

  // Insert mode: only intercept Esc and Ctrl+Enter
  if (currentMode === 'INSERT') {
    if (event.key === 'Escape') {
      event.preventDefault();
      mode.set('NORMAL');
      return;
    }
    if (event.key === 'Enter' && event.ctrlKey) {
      event.preventDefault();
      execute('send');
      return;
    }
    return; // let normal typing through
  }

  // Search mode
  if (get(searchOpen)) {
    if (event.key === 'Escape') {
      searchOpen.set(false);
      searchQuery.set('');
      return;
    }
    if (event.key === 'Enter') {
      execute('search-execute');
      return;
    }
    return; // let typing into search input
  }

  event.preventDefault();

  // ':' enters command mode
  if (event.key === ':' && currentMode === 'NORMAL') {
    mode.set('COMMAND');
    commandInput.set('');
    return;
  }

  // '/' opens search
  if (event.key === '/' && currentMode === 'NORMAL') {
    searchOpen.set(true);
    searchQuery.set('');
    return;
  }

  // '?' toggles help
  if (event.key === '?' && currentMode === 'NORMAL') {
    helpOpen.update(v => !v);
    return;
  }

  // Build key buffer for multi-key sequences
  const key = event.key;
  keyBuffer += key;

  if (keyTimeout) clearTimeout(keyTimeout);

  // Check for exact match
  const exact = bindings.find(b => b.keys === keyBuffer && b.mode === currentMode);
  if (exact) {
    keyBuffer = '';
    execute(exact.action);
    return;
  }

  // Check for partial match (could become longer sequence)
  const partial = bindings.some(b => b.keys.startsWith(keyBuffer) && b.mode === currentMode);
  if (partial) {
    keyTimeout = setTimeout(() => {
      keyBuffer = '';
    }, 500);
    return;
  }

  // No match
  keyBuffer = '';
}

function handleCommandMode(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault();
    mode.set('NORMAL');
    commandInput.set('');
    return;
  }
  if (event.key === 'Enter') {
    event.preventDefault();
    const cmd = get(commandInput);
    mode.set('NORMAL');
    commandInput.set('');
    executeCommand(cmd);
    return;
  }
  if (event.key === 'Backspace') {
    event.preventDefault();
    commandInput.update(v => v.slice(0, -1));
    return;
  }
  if (event.key.length === 1) {
    event.preventDefault();
    commandInput.update(v => v + event.key);
  }
}

function executeCommand(cmd: string) {
  const parts = cmd.trim().split(/\s+/);
  const name = parts[0];
  const args = parts.slice(1).join(' ');

  execute(`cmd:${name}`, args);
}

function execute(action: Action, args?: string) {
  const handler = handlers.get(action);
  if (handler) handler();
}
```

- [ ] **Step 2: Write `frontend/src/lib/keybindings/bindings.ts`**

```typescript
import type { Mode } from '$lib/stores/ui';

export interface BindingDef {
  keys: string;
  action: string;
  mode: Mode;
  description: string;
}

export const defaultBindings: BindingDef[] = [
  // NORMAL mode - navigation
  { keys: 'j', action: 'list-down', mode: 'NORMAL', description: 'Move down in message list' },
  { keys: 'k', action: 'list-up', mode: 'NORMAL', description: 'Move up in message list' },
  { keys: 'J', action: 'thread-next', mode: 'NORMAL', description: 'Next thread' },
  { keys: 'K', action: 'thread-prev', mode: 'NORMAL', description: 'Previous thread' },
  { keys: 'Enter', action: 'open-message', mode: 'NORMAL', description: 'Open selected message' },
  { keys: 'Tab', action: 'toggle-pane', mode: 'NORMAL', description: 'Toggle focus between panes' },

  // NORMAL mode - actions
  { keys: 'r', action: 'reply', mode: 'NORMAL', description: 'Reply' },
  { keys: 'R', action: 'reply-all', mode: 'NORMAL', description: 'Reply all' },
  { keys: 'f', action: 'forward', mode: 'NORMAL', description: 'Forward' },
  { keys: 'c', action: 'compose', mode: 'NORMAL', description: 'Compose new' },
  { keys: 'a', action: 'archive', mode: 'NORMAL', description: 'Archive' },
  { keys: 'd', action: 'delete', mode: 'NORMAL', description: 'Delete' },
  { keys: 's', action: 'star', mode: 'NORMAL', description: 'Star/flag' },
  { keys: 'u', action: 'mark-unread', mode: 'NORMAL', description: 'Mark unread' },
  { keys: 'h', action: 'toggle-html', mode: 'NORMAL', description: 'Toggle HTML/plain text' },
  { keys: 'H', action: 'show-headers', mode: 'NORMAL', description: 'Show raw headers' },
  { keys: 'v', action: 'enter-visual', mode: 'NORMAL', description: 'Enter visual mode' },

  // NORMAL mode - go-to sequences
  { keys: 'gi', action: 'goto-inbox', mode: 'NORMAL', description: 'Go to Inbox' },
  { keys: 'gs', action: 'goto-sent', mode: 'NORMAL', description: 'Go to Sent' },
  { keys: 'gd', action: 'goto-drafts', mode: 'NORMAL', description: 'Go to Drafts' },
  { keys: 'ga', action: 'goto-archive', mode: 'NORMAL', description: 'Go to Archive' },

  // NORMAL mode - account switching
  { keys: '1', action: 'switch-account-1', mode: 'NORMAL', description: 'Switch to account 1' },
  { keys: '2', action: 'switch-account-2', mode: 'NORMAL', description: 'Switch to account 2' },
  { keys: '3', action: 'switch-account-3', mode: 'NORMAL', description: 'Switch to account 3' },
  { keys: '4', action: 'switch-account-4', mode: 'NORMAL', description: 'Switch to account 4' },
  { keys: '5', action: 'switch-account-5', mode: 'NORMAL', description: 'Switch to account 5' },
  { keys: '6', action: 'switch-account-6', mode: 'NORMAL', description: 'Switch to account 6' },
  { keys: '7', action: 'switch-account-7', mode: 'NORMAL', description: 'Switch to account 7' },
  { keys: '8', action: 'switch-account-8', mode: 'NORMAL', description: 'Switch to account 8' },
  { keys: '9', action: 'switch-account-9', mode: 'NORMAL', description: 'Switch to account 9' },

  // VISUAL mode
  { keys: 'j', action: 'visual-extend-down', mode: 'VISUAL', description: 'Extend selection down' },
  { keys: 'k', action: 'visual-extend-up', mode: 'VISUAL', description: 'Extend selection up' },
  { keys: 'a', action: 'visual-archive', mode: 'VISUAL', description: 'Archive selected' },
  { keys: 'd', action: 'visual-delete', mode: 'VISUAL', description: 'Delete selected' },
  { keys: 'Escape', action: 'exit-visual', mode: 'VISUAL', description: 'Cancel selection' },
];
```

- [ ] **Step 3: Wire keybindings into `App.svelte`**

Add to `App.svelte` `<script>`:

```typescript
import { onMount } from 'svelte';
import { handleKeyDown, setBindings } from '$lib/keybindings/engine';
import { defaultBindings } from '$lib/keybindings/bindings';

onMount(() => {
  setBindings(defaultBindings);
  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
});
```

- [ ] **Step 4: Build and verify**

```bash
cd frontend && npm run build 2>&1 | tail -5
```

Expected: builds successfully.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/lib/keybindings/
git commit -m "feat: add vim-style keybinding engine with modal input"
```

---

### Task 11: Frontend — Message List Component

Left pane showing messages with sender, subject, thread count, timestamp. Keyboard navigable.

**Files:**
- Create: `frontend/src/lib/components/MessageList.svelte`
- Create: `frontend/src/lib/stores/messages.ts`
- Modify: `frontend/src/App.svelte`

- [ ] **Step 1: Write `frontend/src/lib/stores/messages.ts`**

```typescript
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Message {
  uid: number;
  message_id: string;
  folder: string;
  subject: string;
  from: string;
  to: string;
  cc: string;
  date: string;
  body_text: string | null;
  body_html: string | null;
  flags: string;
  thread_id: string | null;
  references: string;
  in_reply_to: string;
}

export const messages = writable<Message[]>([]);
export const selectedIndex = writable(0);
export const selectedMessage = derived(
  [messages, selectedIndex],
  ([$messages, $selectedIndex]) => $messages[$selectedIndex] ?? null
);
export const threadMessages = writable<Message[]>([]);

export async function loadMessages(folder: string) {
  try {
    const msgs = await invoke<Message[]>('get_messages', { folder });
    messages.set(msgs);
    selectedIndex.set(0);
  } catch (e) {
    console.error('Failed to load messages:', e);
  }
}

export async function loadThreadMessages(threadId: string) {
  try {
    const msgs = await invoke<Message[]>('get_thread_messages', { threadId });
    threadMessages.set(msgs);
  } catch (e) {
    console.error('Failed to load thread:', e);
  }
}

export async function syncFolder(folder: string) {
  try {
    await invoke('sync_folder', { folder });
    await loadMessages(folder);
  } catch (e) {
    console.error('Failed to sync:', e);
  }
}
```

- [ ] **Step 2: Write `frontend/src/lib/components/MessageList.svelte`**

```svelte
<script lang="ts">
  import { messages, selectedIndex, selectedMessage, loadThreadMessages } from '$lib/stores/messages';
  import { focusPane } from '$lib/stores/ui';
  import { registerHandler } from '$lib/keybindings/engine';
  import { onMount } from 'svelte';

  function formatDate(dateStr: string): string {
    if (!dateStr) return '';
    try {
      const d = new Date(dateStr);
      const now = new Date();
      if (d.toDateString() === now.toDateString()) {
        return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      }
      return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
    } catch {
      return dateStr;
    }
  }

  function extractName(from: string): string {
    const match = from.match(/^"?([^"<]+)"?\s*</);
    if (match) return match[1].trim();
    return from.split('@')[0];
  }

  onMount(() => {
    registerHandler('list-down', () => {
      selectedIndex.update(i => Math.min(i + 1, $messages.length - 1));
    });
    registerHandler('list-up', () => {
      selectedIndex.update(i => Math.max(i - 1, 0));
    });
    registerHandler('open-message', () => {
      const msg = $selectedMessage;
      if (msg?.thread_id) loadThreadMessages(msg.thread_id);
      focusPane.set('reading');
    });
    registerHandler('toggle-pane', () => {
      focusPane.update(p => p === 'list' ? 'reading' : 'list');
    });
  });

  $: isActive = $focusPane === 'list';
</script>

<div class="message-list" class:active={isActive}>
  {#each $messages as msg, i}
    <div
      class="message-item"
      class:selected={i === $selectedIndex}
      class:unread={!msg.flags.includes('Seen')}
      on:click={() => {
        selectedIndex.set(i);
        if (msg.thread_id) loadThreadMessages(msg.thread_id);
        focusPane.set('reading');
      }}
    >
      <div class="message-from">{extractName(msg.from)}</div>
      <div class="message-subject">{msg.subject || '(no subject)'}</div>
      <div class="message-meta">
        <span class="message-date">{formatDate(msg.date)}</span>
      </div>
    </div>
  {/each}
  {#if $messages.length === 0}
    <div class="empty">No messages</div>
  {/if}
</div>

<style>
  .message-list {
    height: 100%;
    overflow-y: auto;
  }
  .message-item {
    padding: 8px 12px;
    border-bottom: 1px solid var(--bg-tertiary);
    cursor: pointer;
    transition: background 0.1s;
  }
  .message-item:hover {
    background: var(--bg-secondary);
  }
  .message-item.selected {
    background: var(--accent-dim);
    border-left: 2px solid var(--accent);
  }
  .message-item.unread .message-from {
    color: var(--text-primary);
    font-weight: 600;
  }
  .message-from {
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 2px;
    font-family: var(--font-mono);
  }
  .message-subject {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
  }
  .message-item.unread .message-subject {
    color: var(--text-primary);
  }
  .message-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .message-date {
    font-size: 10px;
    color: var(--text-dim);
    font-family: var(--font-mono);
  }
  .empty {
    padding: 24px;
    text-align: center;
    color: var(--text-dim);
  }
  .active {
    outline: none;
  }
</style>
```

- [ ] **Step 3: Update `App.svelte` to use MessageList**

Replace the placeholder `<div class="message-list-pane">` content with:

```svelte
<div class="message-list-pane">
  <MessageList />
</div>
```

Add to script: `import MessageList from '$lib/components/MessageList.svelte';`

- [ ] **Step 4: Build and verify**

```bash
cd frontend && npm run build 2>&1 | tail -5
```

- [ ] **Step 5: Commit**

```bash
git add frontend/src/lib/stores/messages.ts frontend/src/lib/components/MessageList.svelte frontend/src/App.svelte
git commit -m "feat: add message list component with keyboard navigation"
```

---

### Task 12: Frontend — Reading Pane + Thread View

Right pane showing selected email with threaded conversation.

**Files:**
- Create: `frontend/src/lib/components/ReadingPane.svelte`
- Modify: `frontend/src/App.svelte`

- [ ] **Step 1: Write `frontend/src/lib/components/ReadingPane.svelte`**

```svelte
<script lang="ts">
  import { selectedMessage, threadMessages } from '$lib/stores/messages';
  import { registerHandler } from '$lib/keybindings/engine';
  import { onMount } from 'svelte';

  let showHtml = true;
  let showHeaders = false;

  function formatDate(dateStr: string): string {
    if (!dateStr) return '';
    try {
      const d = new Date(dateStr);
      return d.toLocaleString();
    } catch {
      return dateStr;
    }
  }

  function extractName(from: string): string {
    const match = from.match(/^"?([^"<]+)"?\s*</);
    if (match) return match[1].trim();
    return from;
  }

  function sanitizeHtml(html: string): string {
    // Strip script tags and event handlers for safety
    return html
      .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
      .replace(/\bon\w+\s*=\s*"[^"]*"/gi, '')
      .replace(/\bon\w+\s*=\s*'[^']*'/gi, '');
  }

  let expandedMessages: Set<number> = new Set();

  function toggleMessage(uid: number) {
    if (expandedMessages.has(uid)) {
      expandedMessages.delete(uid);
    } else {
      expandedMessages.add(uid);
    }
    expandedMessages = expandedMessages; // trigger reactivity
  }

  onMount(() => {
    registerHandler('toggle-html', () => { showHtml = !showHtml; });
    registerHandler('show-headers', () => { showHeaders = !showHeaders; });
  });

  $: thread = $threadMessages;
  $: currentMsg = $selectedMessage;
</script>

<div class="reading-pane">
  {#if currentMsg}
    <div class="email-header">
      <h2 class="subject">{currentMsg.subject || '(no subject)'}</h2>
      <div class="meta">
        <span class="from">{extractName(currentMsg.from)}</span>
        <span class="to">→ {currentMsg.to}</span>
        <span class="date">{formatDate(currentMsg.date)}</span>
      </div>
    </div>

    {#if showHeaders}
      <pre class="raw-headers">{currentMsg.from}\n{currentMsg.to}\n{currentMsg.date}</pre>
    {/if}

    {#if thread.length > 1}
      <div class="thread">
        {#each thread as msg}
          <div class="thread-message" class:current={msg.uid === currentMsg.uid}>
            <div class="thread-header" on:click={() => toggleMessage(msg.uid)}>
              <span class="thread-toggle">{expandedMessages.has(msg.uid) || msg.uid === currentMsg.uid ? '▼' : '▸'}</span>
              <span class="thread-from">{extractName(msg.from)}</span>
              <span class="thread-date">{formatDate(msg.date)}</span>
            </div>
            {#if expandedMessages.has(msg.uid) || msg.uid === currentMsg.uid}
              <div class="thread-body">
                {#if showHtml && msg.body_html}
                  <iframe
                    srcdoc={sanitizeHtml(msg.body_html)}
                    sandbox=""
                    class="html-frame"
                    title="Email content"
                  ></iframe>
                {:else}
                  <pre class="plain-text">{msg.body_text || '(no content)'}</pre>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <div class="body">
        {#if showHtml && currentMsg.body_html}
          <iframe
            srcdoc={sanitizeHtml(currentMsg.body_html)}
            sandbox=""
            class="html-frame"
            title="Email content"
          ></iframe>
        {:else}
          <pre class="plain-text">{currentMsg.body_text || '(no content)'}</pre>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="empty">
      <p>No message selected</p>
      <p class="hint">j/k to navigate, enter to open</p>
    </div>
  {/if}
</div>

<style>
  .reading-pane {
    height: 100%;
    overflow-y: auto;
    padding: 16px;
  }
  .email-header {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .subject {
    font-size: 16px;
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 8px;
  }
  .meta {
    font-size: 12px;
    color: var(--text-secondary);
  }
  .from { color: var(--text-primary); margin-right: 4px; }
  .to { color: var(--text-dim); margin-right: 12px; }
  .date { color: var(--text-dim); }
  .raw-headers {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-dim);
    background: var(--bg-secondary);
    padding: 8px;
    border-radius: 4px;
    margin-bottom: 12px;
    overflow-x: auto;
  }
  .thread-message {
    border-left: 2px solid var(--bg-tertiary);
    margin-bottom: 8px;
    padding-left: 12px;
  }
  .thread-message.current {
    border-left-color: var(--accent);
  }
  .thread-header {
    cursor: pointer;
    padding: 4px 0;
    font-size: 12px;
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .thread-toggle { color: var(--text-dim); font-size: 10px; }
  .thread-from { color: var(--text-secondary); }
  .thread-date { color: var(--text-dim); font-size: 11px; }
  .thread-body {
    padding: 8px 0;
  }
  .plain-text {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-primary);
    white-space: pre-wrap;
    line-height: 1.5;
  }
  .html-frame {
    width: 100%;
    min-height: 300px;
    border: none;
    background: white;
    border-radius: 4px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-dim);
  }
  .hint {
    font-size: 11px;
    font-family: var(--font-mono);
    margin-top: 8px;
    color: var(--text-muted);
  }
</style>
```

- [ ] **Step 2: Update `App.svelte` — replace reading pane placeholder**

Replace `<div class="reading-pane">` content:

```svelte
<div class="reading-pane">
  <ReadingPane />
</div>
```

Add import: `import ReadingPane from '$lib/components/ReadingPane.svelte';`

- [ ] **Step 3: Build and verify**

```bash
cd frontend && npm run build 2>&1 | tail -5
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/ReadingPane.svelte frontend/src/App.svelte
git commit -m "feat: add reading pane with thread view and HTML rendering"
```

---

### Task 13: Frontend — Compose View

Full-pane compose overlay with To, Subject, Body, Cc/Bcc expansion, and markdown support.

**Files:**
- Create: `frontend/src/lib/components/ComposeView.svelte`
- Modify: `frontend/src/App.svelte`

- [ ] **Step 1: Write `frontend/src/lib/components/ComposeView.svelte`**

```svelte
<script lang="ts">
  import { mode } from '$lib/stores/ui';
  import { selectedMessage } from '$lib/stores/messages';
  import { invoke } from '@tauri-apps/api/core';
  import { registerHandler } from '$lib/keybindings/engine';
  import { onMount, createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let replyMode: 'compose' | 'reply' | 'reply-all' | 'forward' = 'compose';

  let to = '';
  let cc = '';
  let bcc = '';
  let subject = '';
  let body = '';
  let showCc = false;
  let showBcc = false;
  let sending = false;
  let error = '';

  let inReplyTo = '';
  let references: string[] = [];

  onMount(() => {
    mode.set('INSERT');

    const msg = $selectedMessage;
    if (msg && replyMode !== 'compose') {
      if (replyMode === 'reply' || replyMode === 'reply-all') {
        to = msg.from;
        if (replyMode === 'reply-all' && msg.cc) {
          cc = msg.cc;
          showCc = true;
        }
        subject = msg.subject.startsWith('Re:') ? msg.subject : `Re: ${msg.subject}`;
        inReplyTo = msg.message_id;
        references = msg.references ? msg.references.split(' ') : [];
        if (msg.message_id) references.push(msg.message_id);
        body = `\n\nOn ${msg.date}, ${msg.from} wrote:\n> ${(msg.body_text || '').split('\n').join('\n> ')}`;
      } else if (replyMode === 'forward') {
        subject = msg.subject.startsWith('Fwd:') ? msg.subject : `Fwd: ${msg.subject}`;
        body = `\n\n---------- Forwarded message ----------\nFrom: ${msg.from}\nDate: ${msg.date}\nSubject: ${msg.subject}\nTo: ${msg.to}\n\n${msg.body_text || ''}`;
      }
    }
  });

  async function send() {
    if (!to.trim()) {
      error = 'Recipient is required';
      return;
    }
    sending = true;
    error = '';
    try {
      await invoke('send_email', {
        req: {
          to: to.split(',').map((s: string) => s.trim()).filter(Boolean),
          cc: cc.split(',').map((s: string) => s.trim()).filter(Boolean),
          bcc: bcc.split(',').map((s: string) => s.trim()).filter(Boolean),
          subject,
          body_text: body,
          body_html: null,
          in_reply_to: inReplyTo || null,
          references,
        }
      });
      close();
    } catch (e: any) {
      error = e.toString();
    } finally {
      sending = false;
    }
  }

  function close() {
    mode.set('NORMAL');
    dispatch('close');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && e.ctrlKey) {
      e.preventDefault();
      send();
    }
  }

  registerHandler('send', send);
</script>

<div class="compose" on:keydown={handleKeydown}>
  <div class="compose-header">
    <h3>{replyMode === 'compose' ? 'New Message' : replyMode === 'forward' ? 'Forward' : 'Reply'}</h3>
    <button class="close-btn" on:click={close}>Esc</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="field">
    <label>To</label>
    <input bind:value={to} placeholder="recipient@example.com" />
  </div>

  {#if showCc}
    <div class="field">
      <label>Cc</label>
      <input bind:value={cc} />
    </div>
  {/if}

  {#if showBcc}
    <div class="field">
      <label>Bcc</label>
      <input bind:value={bcc} />
    </div>
  {/if}

  <div class="field-toggles">
    {#if !showCc}<button on:click={() => showCc = true}>Cc</button>{/if}
    {#if !showBcc}<button on:click={() => showBcc = true}>Bcc</button>{/if}
  </div>

  <div class="field">
    <label>Subject</label>
    <input bind:value={subject} />
  </div>

  <textarea class="body-input" bind:value={body} placeholder="Write your message..."></textarea>

  <div class="compose-footer">
    <button class="send-btn" on:click={send} disabled={sending}>
      {sending ? 'Sending...' : 'Send (Ctrl+Enter)'}
    </button>
  </div>
</div>

<style>
  .compose {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px;
  }
  .compose-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .compose-header h3 {
    font-size: 14px;
    color: var(--text-primary);
  }
  .close-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-dim);
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
  }
  .error {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    padding: 8px;
    border-radius: 4px;
    font-size: 12px;
    margin-bottom: 8px;
  }
  .field {
    margin-bottom: 8px;
  }
  .field label {
    display: block;
    font-size: 11px;
    color: var(--text-dim);
    margin-bottom: 2px;
    font-family: var(--font-mono);
  }
  .field input {
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 6px 8px;
    font-size: 13px;
    border-radius: 3px;
    outline: none;
  }
  .field input:focus {
    border-color: var(--accent);
  }
  .field-toggles {
    margin-bottom: 8px;
  }
  .field-toggles button {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 11px;
    cursor: pointer;
    margin-right: 8px;
    font-family: var(--font-mono);
  }
  .body-input {
    flex: 1;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 8px;
    font-size: 13px;
    font-family: var(--font-sans);
    border-radius: 3px;
    resize: none;
    outline: none;
    line-height: 1.5;
  }
  .body-input:focus {
    border-color: var(--accent);
  }
  .compose-footer {
    margin-top: 8px;
    display: flex;
    justify-content: flex-end;
  }
  .send-btn {
    background: var(--accent);
    color: white;
    border: none;
    padding: 6px 16px;
    font-size: 12px;
    border-radius: 3px;
    cursor: pointer;
  }
  .send-btn:disabled {
    opacity: 0.5;
  }
</style>
```

- [ ] **Step 2: Update `App.svelte` — add compose state and overlay**

Add to script:

```typescript
import ComposeView from '$lib/components/ComposeView.svelte';

let composing = false;
let composeMode: 'compose' | 'reply' | 'reply-all' | 'forward' = 'compose';

registerHandler('compose', () => { composeMode = 'compose'; composing = true; });
registerHandler('reply', () => { composeMode = 'reply'; composing = true; });
registerHandler('reply-all', () => { composeMode = 'reply-all'; composing = true; });
registerHandler('forward', () => { composeMode = 'forward'; composing = true; });
```

In the reading-pane div:

```svelte
<div class="reading-pane">
  {#if composing}
    <ComposeView replyMode={composeMode} on:close={() => composing = false} />
  {:else}
    <ReadingPane />
  {/if}
</div>
```

- [ ] **Step 3: Build and verify**

```bash
cd frontend && npm run build 2>&1 | tail -5
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/ComposeView.svelte frontend/src/App.svelte
git commit -m "feat: add compose view with reply, forward, and Cc/Bcc support"
```

---

### Task 14: Frontend — Account Setup + Search + Help

First-run account setup, search overlay, and keybinding help overlay.

**Files:**
- Create: `frontend/src/lib/components/AccountSetup.svelte`
- Create: `frontend/src/lib/components/SearchBar.svelte`
- Create: `frontend/src/lib/components/HelpOverlay.svelte`
- Modify: `frontend/src/App.svelte`

- [ ] **Step 1: Write `frontend/src/lib/components/AccountSetup.svelte`**

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { accounts, activeAccount } from '$lib/stores/accounts';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let name = '';
  let email = '';
  let password = '';
  let imapHost = '';
  let imapPort = 993;
  let smtpHost = '';
  let smtpPort = 587;
  let username = '';
  let testing = false;
  let saving = false;
  let error = '';
  let testResult = '';

  async function autoFill(provider: string) {
    const defaults = await invoke<[string, number, string, number] | null>('get_provider_defaults', { provider });
    if (defaults) {
      [imapHost, imapPort, smtpHost, smtpPort] = defaults;
    }
  }

  async function testConnection() {
    testing = true;
    error = '';
    testResult = '';
    try {
      const folders = await invoke<string[]>('test_imap_connection', {
        host: imapHost, port: imapPort, username: username || email, password
      });
      testResult = `Connected! Found ${folders.length} folders.`;
    } catch (e: any) {
      error = e.toString();
    } finally {
      testing = false;
    }
  }

  async function save() {
    saving = true;
    error = '';
    try {
      const id = await invoke<string>('add_account', {
        req: {
          name: name || email,
          email,
          password,
          imap_host: imapHost,
          imap_port: imapPort,
          smtp_host: smtpHost,
          smtp_port: smtpPort,
          username: username || email,
        }
      });
      await invoke('set_active_account', { id });
      const accts = await invoke<any[]>('list_accounts');
      accounts.set(accts);
      activeAccount.set(accts.find(a => a.id === id) ?? null);
      dispatch('done');
    } catch (e: any) {
      error = e.toString();
    } finally {
      saving = false;
    }
  }
</script>

<div class="setup">
  <h2>Add Account</h2>
  <p class="subtitle">Quick setup for common providers:</p>

  <div class="providers">
    <button on:click={() => autoFill('gmail')}>Gmail</button>
    <button on:click={() => autoFill('outlook')}>Outlook</button>
    <button on:click={() => autoFill('yahoo')}>Yahoo</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}
  {#if testResult}
    <div class="success">{testResult}</div>
  {/if}

  <div class="field">
    <label>Display Name</label>
    <input bind:value={name} placeholder="Work, Personal, etc." />
  </div>
  <div class="field">
    <label>Email Address</label>
    <input bind:value={email} type="email" placeholder="you@example.com" />
  </div>
  <div class="field">
    <label>Password / App Password</label>
    <input bind:value={password} type="password" />
  </div>

  <div class="row">
    <div class="field">
      <label>IMAP Server</label>
      <input bind:value={imapHost} placeholder="imap.example.com" />
    </div>
    <div class="field small">
      <label>Port</label>
      <input bind:value={imapPort} type="number" />
    </div>
  </div>

  <div class="row">
    <div class="field">
      <label>SMTP Server</label>
      <input bind:value={smtpHost} placeholder="smtp.example.com" />
    </div>
    <div class="field small">
      <label>Port</label>
      <input bind:value={smtpPort} type="number" />
    </div>
  </div>

  <div class="field">
    <label>Username (if different from email)</label>
    <input bind:value={username} placeholder="Leave blank to use email" />
  </div>

  <div class="actions">
    <button class="test-btn" on:click={testConnection} disabled={testing}>
      {testing ? 'Testing...' : 'Test Connection'}
    </button>
    <button class="save-btn" on:click={save} disabled={saving}>
      {saving ? 'Saving...' : 'Save Account'}
    </button>
  </div>
</div>

<style>
  .setup {
    max-width: 500px;
    margin: 0 auto;
    padding: 40px 20px;
  }
  h2 { font-size: 18px; color: var(--text-primary); margin-bottom: 4px; }
  .subtitle { font-size: 12px; color: var(--text-dim); margin-bottom: 16px; }
  .providers {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }
  .providers button {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 6px 16px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 12px;
  }
  .providers button:hover { border-color: var(--accent); }
  .error {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    padding: 8px;
    border-radius: 4px;
    font-size: 12px;
    margin-bottom: 12px;
  }
  .success {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
    padding: 8px;
    border-radius: 4px;
    font-size: 12px;
    margin-bottom: 12px;
  }
  .field { margin-bottom: 10px; flex: 1; }
  .field.small { max-width: 80px; }
  .field label {
    display: block;
    font-size: 11px;
    color: var(--text-dim);
    margin-bottom: 2px;
    font-family: var(--font-mono);
  }
  .field input {
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 6px 8px;
    font-size: 13px;
    border-radius: 3px;
    outline: none;
  }
  .field input:focus { border-color: var(--accent); }
  .row { display: flex; gap: 8px; }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }
  .test-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 8px 16px;
    border-radius: 3px;
    cursor: pointer;
  }
  .save-btn {
    background: var(--accent);
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 3px;
    cursor: pointer;
  }
  button:disabled { opacity: 0.5; }
</style>
```

- [ ] **Step 2: Write `frontend/src/lib/components/SearchBar.svelte`**

```svelte
<script lang="ts">
  import { searchOpen, searchQuery } from '$lib/stores/ui';
  import { messages } from '$lib/stores/messages';
  import { invoke } from '@tauri-apps/api/core';
  import { registerHandler } from '$lib/keybindings/engine';

  let inputEl: HTMLInputElement;

  $: if ($searchOpen && inputEl) {
    setTimeout(() => inputEl?.focus(), 0);
  }

  registerHandler('search-execute', async () => {
    if (!$searchQuery.trim()) return;
    try {
      const results = await invoke<any[]>('search_messages', { query: $searchQuery, limit: 50 });
      messages.set(results);
    } catch (e) {
      console.error('Search failed:', e);
    }
    searchOpen.set(false);
  });
</script>

{#if $searchOpen}
  <div class="search-bar">
    <span class="search-icon">/</span>
    <input
      bind:this={inputEl}
      bind:value={$searchQuery}
      placeholder="Search messages... (from:alice subject:project)"
      class="search-input"
    />
    <span class="search-hint">enter search · esc cancel</span>
  </div>
{/if}

<style>
  .search-bar {
    display: flex;
    align-items: center;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--accent);
    gap: 8px;
  }
  .search-icon {
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 14px;
  }
  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
  }
  .search-hint {
    color: var(--text-dim);
    font-size: 10px;
    font-family: var(--font-mono);
  }
</style>
```

- [ ] **Step 3: Write `frontend/src/lib/components/HelpOverlay.svelte`**

```svelte
<script lang="ts">
  import { helpOpen } from '$lib/stores/ui';
  import { defaultBindings } from '$lib/keybindings/bindings';

  $: normalBindings = defaultBindings.filter(b => b.mode === 'NORMAL');
  $: visualBindings = defaultBindings.filter(b => b.mode === 'VISUAL');
</script>

{#if $helpOpen}
  <div class="overlay" on:click={() => helpOpen.set(false)}>
    <div class="help-panel" on:click|stopPropagation>
      <h2>Keybindings</h2>

      <h3>Normal Mode</h3>
      <div class="bindings">
        {#each normalBindings as b}
          <div class="binding">
            <kbd>{b.keys}</kbd>
            <span>{b.description}</span>
          </div>
        {/each}
        <div class="binding"><kbd>/</kbd><span>Search</span></div>
        <div class="binding"><kbd>:</kbd><span>Command mode</span></div>
        <div class="binding"><kbd>?</kbd><span>Toggle this help</span></div>
      </div>

      <h3>Visual Mode</h3>
      <div class="bindings">
        {#each visualBindings as b}
          <div class="binding">
            <kbd>{b.keys}</kbd>
            <span>{b.description}</span>
          </div>
        {/each}
      </div>

      <h3>Insert Mode (Composing)</h3>
      <div class="bindings">
        <div class="binding"><kbd>Esc</kbd><span>Back to Normal</span></div>
        <div class="binding"><kbd>Ctrl+Enter</kbd><span>Send</span></div>
      </div>

      <p class="close-hint">Press ? or click outside to close</p>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .help-panel {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 24px;
    max-width: 500px;
    max-height: 80vh;
    overflow-y: auto;
  }
  h2 { font-size: 16px; color: var(--text-primary); margin-bottom: 16px; }
  h3 { font-size: 12px; color: var(--accent); margin: 12px 0 6px; font-family: var(--font-mono); }
  .bindings {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 12px;
  }
  .binding {
    display: contents;
  }
  kbd {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    padding: 1px 6px;
    border-radius: 3px;
    color: var(--accent);
    text-align: center;
  }
  .binding span {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 24px;
  }
  .close-hint {
    margin-top: 16px;
    font-size: 11px;
    color: var(--text-dim);
    text-align: center;
  }
</style>
```

- [ ] **Step 4: Update `App.svelte` — integrate all overlays and account setup**

Add imports and account initialization logic:

```typescript
import AccountSetup from '$lib/components/AccountSetup.svelte';
import SearchBar from '$lib/components/SearchBar.svelte';
import HelpOverlay from '$lib/components/HelpOverlay.svelte';
import { accounts, activeAccount, activeFolder } from '$lib/stores/accounts';
import { syncFolder, loadMessages } from '$lib/stores/messages';

let hasAccounts = false;

onMount(async () => {
  setBindings(defaultBindings);
  window.addEventListener('keydown', handleKeyDown);

  // Load accounts
  try {
    const accts = await invoke<any[]>('list_accounts');
    accounts.set(accts);
    hasAccounts = accts.length > 0;
    if (hasAccounts) {
      activeAccount.set(accts[0]);
      await invoke('set_active_account', { id: accts[0].id });
      await syncFolder('INBOX');
    }
  } catch (e) {
    console.error('Failed to load accounts:', e);
  }

  return () => window.removeEventListener('keydown', handleKeyDown);
});

// Register folder navigation
registerHandler('goto-inbox', () => { activeFolder.set('INBOX'); loadMessages('INBOX'); });
registerHandler('goto-sent', () => { activeFolder.set('Sent'); loadMessages('Sent'); });
registerHandler('goto-drafts', () => { activeFolder.set('Drafts'); loadMessages('Drafts'); });
registerHandler('goto-archive', () => { activeFolder.set('Archive'); loadMessages('Archive'); });
```

Template:

```svelte
<div id="app">
  {#if !hasAccounts}
    <AccountSetup on:done={() => { hasAccounts = true; syncFolder('INBOX'); }} />
  {:else}
    <StatusBar />
    <SearchBar />
    <main class="content">
      <div class="message-list-pane">
        <MessageList />
      </div>
      <div class="reading-pane">
        {#if composing}
          <ComposeView replyMode={composeMode} on:close={() => composing = false} />
        {:else}
          <ReadingPane />
        {/if}
      </div>
    </main>
    <HintBar />
    <HelpOverlay />
  {/if}
</div>
```

- [ ] **Step 5: Build and verify**

```bash
cd frontend && npm run build 2>&1 | tail -5
```

- [ ] **Step 6: Commit**

```bash
git add frontend/src/
git commit -m "feat: add account setup, search bar, and help overlay"
```

---

### Task 15: Integration — Wire Actions + Background Sync + Notifications

Connect keybinding actions to Tauri commands, add background sync timer, and native notifications.

**Files:**
- Modify: `frontend/src/App.svelte`
- Modify: `src-tauri/src/main.rs` (if Tauri notification plugin needed)
- Modify: `src-tauri/tauri.conf.json` (enable notification permission)

- [ ] **Step 1: Wire remaining keybinding actions in `App.svelte`**

Add these handler registrations:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { selectedMessage, messages, selectedIndex, loadMessages } from '$lib/stores/messages';
import { get } from 'svelte/store';

registerHandler('archive', async () => {
  const msg = get(selectedMessage);
  if (!msg) return;
  await invoke('delete_message', { uid: msg.uid, folder: msg.folder });
  await loadMessages(get(activeFolder));
});

registerHandler('delete', async () => {
  const msg = get(selectedMessage);
  if (!msg) return;
  await invoke('delete_message', { uid: msg.uid, folder: msg.folder });
  await loadMessages(get(activeFolder));
});

registerHandler('star', async () => {
  const msg = get(selectedMessage);
  if (!msg) return;
  const newFlags = msg.flags.includes('Flagged')
    ? msg.flags.replace('Flagged', '').trim()
    : `${msg.flags} Flagged`.trim();
  await invoke('update_flags', { uid: msg.uid, folder: msg.folder, flags: newFlags });
  await loadMessages(get(activeFolder));
});

registerHandler('mark-unread', async () => {
  const msg = get(selectedMessage);
  if (!msg) return;
  const newFlags = msg.flags.replace('Seen', '').trim();
  await invoke('update_flags', { uid: msg.uid, folder: msg.folder, flags: newFlags });
  await loadMessages(get(activeFolder));
});

// Account switching
for (let i = 1; i <= 9; i++) {
  registerHandler(`switch-account-${i}`, async () => {
    const accts = get(accounts);
    if (i <= accts.length) {
      activeAccount.set(accts[i - 1]);
      await invoke('set_active_account', { id: accts[i - 1].id });
      activeFolder.set('INBOX');
      await syncFolder('INBOX');
    }
  });
}
```

- [ ] **Step 2: Add background sync interval**

In `onMount`:

```typescript
// Background sync every 5 minutes
const syncInterval = setInterval(async () => {
  if (hasAccounts) {
    try {
      await syncFolder(get(activeFolder));
    } catch (e) {
      console.error('Background sync failed:', e);
    }
  }
}, 5 * 60 * 1000);

return () => {
  window.removeEventListener('keydown', handleKeyDown);
  clearInterval(syncInterval);
};
```

- [ ] **Step 3: Add native notifications for new mail**

Install Tauri notification plugin:

```bash
cd src-tauri && cargo add tauri-plugin-notification
```

Update `src-tauri/src/main.rs`:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_notification::init())
    // ... rest of builder
```

In frontend, after sync:

```typescript
import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

async function notifyNewMail(count: number) {
  let permitted = await isPermissionGranted();
  if (!permitted) {
    const permission = await requestPermission();
    permitted = permission === 'granted';
  }
  if (permitted && count > 0) {
    sendNotification({ title: 'ScouterMail', body: `${count} new message${count > 1 ? 's' : ''}` });
  }
}
```

Install frontend package:

```bash
cd frontend && npm install @tauri-apps/plugin-notification
```

- [ ] **Step 4: Build full app and verify**

```bash
cd frontend && npm install && npm run build && cd ../src-tauri && cargo build 2>&1 | tail -10
```

Expected: full app builds successfully.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: wire keybinding actions, background sync, and native notifications"
```

---

### Task 16: Final Polish — Window Config + App Icon + First Launch

Configure Tauri window settings, dark title bar, and app metadata.

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Update `src-tauri/tauri.conf.json`**

Ensure these settings are present:

```json
{
  "app": {
    "windows": [
      {
        "title": "ScouterMail",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 500,
        "decorations": true,
        "transparent": false
      }
    ]
  },
  "bundle": {
    "identifier": "com.scoutermail.app",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 2: Verify full build**

```bash
cd src-tauri && cargo build --release 2>&1 | tail -5
```

- [ ] **Step 3: Launch and smoke test**

```bash
npm run tauri dev
```

Expected: App launches with dark theme, shows account setup on first run.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: configure window settings and app metadata"
```
