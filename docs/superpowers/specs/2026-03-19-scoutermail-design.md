# ScouterMail — Desktop Email Client Design Spec

## Overview

ScouterMail is a lightweight, vim-driven desktop email client for macOS. It prioritizes productivity and minimalism — no chrome, no clutter, everything driven by keyboard. Think Outlook's threading and structure meets Mutt's keyboard efficiency in a modern dark UI.

**Target user:** Personal use, 1-3 IMAP accounts, moderate volume.

**This is a greenfield build.** The repository previously contained a Go/Wails prototype which is being replaced entirely with a Rust/Tauri/Svelte stack. All existing Go code should be discarded.

## Tech Stack

- **Backend:** Rust (via Tauri)
- **Frontend:** Svelte
- **Desktop framework:** Tauri (system webview, ~5-10MB binary)
- **Local storage:** SQLite (one database per account)
- **Full-text search:** Tantivy
- **Email protocols:** IMAP (sync/read), SMTP (send)
- **Future extensibility:** Gmail API, Microsoft Graph API as optional provider backends

## Architecture

```
src-tauri/                — Rust backend
  src/
    imap/                 — IMAP sync engine (async-imap)
    smtp/                 — SMTP sending
    parser/               — Email parsing (mailparse)
    store/                — SQLite storage + Tantivy full-text search index
    accounts/             — Multi-account management & credentials
    commands.rs           — Tauri IPC command handlers
frontend/                 — Svelte frontend
  src/
    lib/
      components/         — UI components
      keybindings/        — Vim keymap engine
      stores/             — Svelte stores (messages, accounts, UI state)
    App.svelte
```

**Data flow:** IMAP sync → mailparse → SQLite + Tantivy index → Tauri IPC → Svelte stores → UI render

## Layout — Two-Pane, No Sidebar

```
┌─────────────────────────────────────────────────────────┐
│ Work:Inbox(3)                    NORMAL            ? help│  ← status bar
├──────────────────┬──────────────────────────────────────┤
│ ▸ Alice Smith  3 │ Re: Project update                   │
│   Re: Project... │ Alice → me, Bob · 10:32 AM           │
│                  │                                       │
│   Bob Jones      │ Hey, just wanted to follow up on the  │
│   Meeting tom... │ project timeline. I think we should   │
│                  │ push the deadline by a week...         │
│   GitHub         │                                       │
│   [repo] PR #42  │ ▼ Bob Jones · Mar 18                  │
│                  │ │ Sounds good, let's discuss.          │
│   Newsletter     │                                       │
│   Weekly digest  │                                       │
├──────────────────┴──────────────────────────────────────┤
│ j/k navigate · enter open · r reply · a archive · / search│  ← hint bar
└─────────────────────────────────────────────────────────┘
```

- **No persistent sidebar.** Folder/account switching via `g` commands and `:` commands.
- **Status bar (top):** current account:folder, mode indicator (NORMAL/INSERT/VISUAL/COMMAND), help shortcut.
- **Message list (~30-35% width, resizable):** sender, subject preview, thread count, timestamp. Selected message highlighted with accent border.
- **Reading pane:** email header (sender, recipients, date), body, threaded replies as collapsible chain (newest at bottom).
- **Hint bar (bottom):** contextual keybinding hints, doubles as command input in COMMAND mode.

## Vim Keybinding System

All bindings are remappable via config file.

### NORMAL mode (default)

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate message list down/up |
| `J` / `K` | Next/prev thread |
| `enter` | Open message in reading pane |
| `r` | Reply |
| `R` | Reply all |
| `f` | Forward |
| `c` | Compose new message |
| `a` | Archive |
| `d` | Delete |
| `s` | Star/flag |
| `u` | Mark unread |
| `/` | Open search bar |
| `gi` | Go to Inbox |
| `gs` | Go to Sent |
| `gd` | Go to Drafts |
| `ga` | Go to Archive |
| `1`-`9` | Switch account by number |
| `tab` | Toggle focus between list and reading pane |
| `h` | Toggle HTML/plain text in reading pane |
| `H` | Show full raw headers |
| `v` | Enter VISUAL mode |
| `?` | Show all keybindings |

### COMMAND mode (`:` prefix)

| Command | Action |
|---------|--------|
| `:move <folder>` | Move message to folder |
| `:label <name>` | Add label |
| `:account <name>` | Switch account by name |
| `:set <option>` | Change settings |

### INSERT mode (composing)

- Normal text editing
- `Esc` — back to NORMAL
- `Ctrl+Enter` — send message

### VISUAL mode (`v` from NORMAL)

- `j` / `k` — extend selection
- `a` — archive selected, `d` — delete selected
- `Esc` — cancel selection

## Email Core

### Threading

- RFC 5256 threading via `References` and `In-Reply-To` headers.
- Fallback: subject-based grouping when headers are missing.
- Thread view: collapsible conversation chain, newest message at bottom.
- Thread count badge shown in message list.

### Parsing

- Full MIME parsing: multipart bodies, attachments, inline images.
- HTML email rendered in a sandboxed iframe with all external resource loading blocked (images, scripts, stylesheets). No per-message or global opt-in for v1 — external resources are always blocked.
- Inline images (`cid:` references) are resolved from MIME parts and injected as `data:` URIs into the HTML before rendering.
- Plain text preferred when available; toggle via `h` key.
- Full raw headers viewable via `H` key.

### IMAP Sync

- Background sync on configurable interval (default: 5 minutes).
- IMAP IDLE push for real-time updates when server supports it.
- Initial sync strategy: headers first (fast list population), then bodies on demand + background fetch.
- Offline queue: out of scope for v1. Actions require an active connection; if offline, the action fails with a clear error. Offline queuing is a future enhancement.

### SMTP

- Standard SMTP with STARTTLS/SSL.
- Drafts auto-saved locally every 30 seconds.
- Sent messages synced back to IMAP Sent folder.

## Local Storage

### SQLite (one database per account)

**Core tables:**
- `messages` — uid, message_id, folder, subject, from, to, cc, date, body_text, body_html, flags, thread_id
- `threads` — thread_id, subject, last_date, message_count
- `folders` — name, uidvalidity, uidnext, last_sync
- `contacts` — email, name, last_seen, frequency
- `labels` — label_id, name, color
- `message_labels` — message_uid, label_id
- `attachments` — message_uid, filename, content_type, size, path

### Tantivy Full-Text Search Index

- Indexed fields: subject, from, to, body_text.
- Updated incrementally on each sync.
- Search accessible via `/` in NORMAL mode.
- Supports query syntax: `from:alice subject:project` etc.

## UI Design

### Dark Theme

- Background: `#0d0d14`
- Borders: `#1a1a2e`
- Accent: `#7c3aed` (muted purple) — selection, unread, active states
- Text: white (primary), gray (secondary), dim (metadata)
- Message list font: monospace (JetBrains Mono / system)
- Reading pane font: proportional (system default)

### Compose View

- Opens as full-pane overlay replacing reading pane.
- Minimal fields: To, Subject, Body.
- Cc/Bcc expand on demand (`Ctrl+B` for Bcc, `Ctrl+C` for Cc).
- Markdown compose with preview toggle. Sends as multipart/alternative (plain text + HTML generated from markdown).

### Notifications

- Native macOS notifications for new mail.
- Unread count in dock badge.
- Subtle in-app indicator in status bar.

## Account Management

### Account Setup Flow

On first launch (no accounts configured), show a minimal setup screen:

1. **Server type selection:** IMAP/SMTP (v1 only). Buttons for common providers (Gmail, Outlook, Yahoo) that auto-fill server/port defaults.
2. **Credentials form:** Email address, password (or app-specific password), IMAP server, IMAP port, SMTP server, SMTP port, TLS toggle. Common providers auto-fill everything except email/password.
3. **Test connection** button — validates IMAP and SMTP before saving.
4. **Save** — stores credentials in macOS Keychain, creates account SQLite database, starts initial sync.

OAuth2 is **out of scope for v1**. Gmail/Outlook users must use app-specific passwords. OAuth2 will be added alongside the provider-specific API backends.

The same form is accessible via `:account add` command or a settings view (`:settings`).

### Runtime

- Credentials stored in macOS Keychain.
- Each account has its own SQLite database and sync state.
- Switch between accounts via `1`-`9` number keys or `:account <name>`.

## Future Extensibility (Out of Scope for v1)

- Gmail API / Microsoft Graph API provider backends
- Unified inbox (all accounts in one view)
- Calendar integration
- Contact management / address book
- PGP/GPG encryption
- Custom themes
