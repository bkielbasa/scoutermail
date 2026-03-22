<p align="center">
  <h1 align="center">ScouterMail</h1>
  <p align="center">
    A lightweight, vim-driven desktop email client for macOS.
    <br />
    Built with Rust, Tauri, and Svelte.
  </p>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-000000?style=flat-square&logo=apple" />
  <img src="https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/tauri-v2-24C8D8?style=flat-square" />
  <img src="https://img.shields.io/badge/svelte-5-FF3E00?style=flat-square&logo=svelte" />
  <img src="https://img.shields.io/badge/tests-40%20passing-brightgreen?style=flat-square" />
</p>

---

ScouterMail is an email client that treats your keyboard like a first-class citizen. If you've ever wished your email worked like Vim, this is it. No chrome, no clutter, no mouse required. Everything is a keystroke or a command away.

```
 Work:INBOX(3)                    NORMAL                ? help
+-----------------+---------------------------------------------+
| Alice Smith  10:32 | Re: Project update                      |
| > Plus a new Go... | Alice -> me, Bob - Mar 20, 2026 at 10:32|
|                    |                                          |
| Bob Jones     9:15 | Hey, just wanted to follow up on the     |
| > Can we push...   | project timeline. I think we should      |
|                    | push the deadline by a week...            |
| GitHub       Yest. |                                          |
| > [repo] PR #42   | > Bob Jones - Mar 18                     |
|                    | > Sounds good, let's discuss.             |
+-----------------+---------------------------------------------+
 j/k navigate - enter open - r reply - a archive - / search - ? help
```

## Why ScouterMail?

- **~15MB binary**. No Electron. Uses the native macOS WebKit webview via Tauri.
- **Vim keybindings everywhere**. `j/k` to navigate, `gg`/`G` to jump, `v` for visual select, `:` for commands, `/` to search.
- **Instant full-text search**. Tantivy indexes every message locally. `from:alice subject:project` just works.
- **Offline-first**. All mail synced to local SQLite. Search and read without a connection.
- **Rules engine with superpowers**. Auto-label, auto-move, fire webhooks, run shell commands, or call AI APIs on incoming mail.
- **Dark theme by default**. Because your eyes matter.

## Features

### Core Email
- IMAP sync with IDLE push support
- SMTP sending with STARTTLS and implicit TLS (port 465/587)
- RFC 5256 threading (References + In-Reply-To, subject fallback)
- Full MIME parsing: multipart, attachments, inline images
- HTML email rendering in sandboxed iframe with CSP protection
- Plain text toggle (`h` key)
- Reply-To header handling

### Vim-Style Interface

Every interaction is keyboard-driven:

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate messages |
| `J` / `K` | Navigate threads |
| `gg` / `G` | Jump to top / bottom |
| `Enter` | Open message |
| `r` / `R` | Reply / Reply all |
| `f` | Forward |
| `c` | Compose |
| `a` | Archive |
| `d` | Delete |
| `s` | Star |
| `u` | Mark unread |
| `!` | Mark as spam |
| `v` | Visual mode (bulk select) |
| `/` | Search |
| `?` | Help |
| `:` | Command mode |
| `Tab` | Toggle pane focus |
| `+` / `-` / `=` | Resize reading font |
| `[` / `]` | Previous / Next page |
| `1`-`9` | Switch accounts |

### Command Mode

Type `:` to enter command mode. Tab auto-completes.

| Command | Description |
|---------|-------------|
| `:move <folder>` | Move message to folder |
| `:label <name>` | Add label to message |
| `:unlabel <name>` | Remove label |
| `:labeled <name>` | Show messages with label |
| `:filter unread\|starred\|all` | Quick filter current view |
| `:contacts` | Open contacts list |
| `:calendar` | Open calendar (agenda + month grid) |
| `:folders` | Toggle folder sidebar |
| `:drafts` | Show saved drafts |
| `:unified` | Unified inbox (all accounts) |
| `:signature` | Edit email signature |
| `:template save\|list\|delete\|<name>` | Manage canned responses |
| `:rules` | Open rules editor |
| `:snooze <minutes>` | Snooze message |
| `:spam` | Mark as spam |
| `:print` | Print email |
| `:backup` | Backup database |
| `:set <key> <value>` | Change setting |

### Calendar

- Parses `.ics` calendar invites from emails automatically
- Accept / Tentative / Decline with RSVP sent to organizer
- Agenda view (chronological list) and Month grid view
- Toggle views with `m`, navigate with `h`/`l`, jump to today with `t`

### Compose

- Markdown formatting toolbar (Bold, Italic, Link, List, Quote, Code)
- Keyboard shortcuts: `Ctrl+B` bold, `Ctrl+I` italic, `Ctrl+K` link
- Auto-complete contacts in To/Cc/Bcc fields
- Email signature auto-appended
- Draft auto-save every 30 seconds
- Schedule send: click "Schedule" or compose now, send later
- Templates: save and insert canned responses

### Rules Engine

Automate your inbox with conditions and actions:

**Conditions** match on from, to, subject, body using contains, equals, regex, or negations.

**Built-in actions:**
- Add/remove labels
- Move to folder
- Mark read/unread, star, archive, delete
- Auto-reply with a template

**Extended actions:**
- **Webhook** &mdash; POST message data to any URL
- **Shell** &mdash; run commands with `$MAIL_FROM`, `$MAIL_SUBJECT`, `$MAIL_BODY` env vars
- **AI Prompt** &mdash; send to an LLM API for auto-labeling, summarization, or smart replies

Example: *"If from contains `invoice` → add label `finance` → webhook POST to accounting API"*

### Multi-Account

- Support for multiple IMAP/SMTP accounts
- Quick switch with `1`-`9` number keys
- Unified inbox (`:unified`) merges all accounts
- Per-account color coding
- Provider auto-fill for Gmail, Outlook, Yahoo

### Search

Powered by [Tantivy](https://github.com/quickwit-oss/tantivy) full-text search:

```
/meeting notes                    # search everywhere
/from:alice                       # search by sender
/subject:invoice from:finance     # combine fields
```

### Organization

- **Labels** &mdash; tag messages, filter by label
- **Quick filters** &mdash; `:filter unread`, `:filter starred`
- **Snooze** &mdash; hide messages and resurface later
- **Folder list** &mdash; browse all IMAP folders with unread counts
- **Pagination** &mdash; 50 messages per page, `[`/`]` to navigate

### Performance

- **Lazy body loading** &mdash; message list loads headers only, body fetched on demand
- **Background flag sync** &mdash; flag changes sync to IMAP without blocking UI
- **IMAP retry** &mdash; exponential backoff on connection failures
- **Pagination** &mdash; handles large folders without loading everything into memory

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri v2](https://v2.tauri.app) (system WebKit webview, ~15MB binary) |
| Backend | Rust |
| Frontend | Svelte 5, TypeScript, Vite |
| Local storage | SQLite via rusqlite |
| Full-text search | Tantivy |
| Email protocols | async-imap, lettre (SMTP) |
| Email parsing | mailparse |
| Calendar | ical crate |
| Notifications | tauri-plugin-notification |

## Getting Started

### Prerequisites

- macOS 12+
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+

### Install and Run

```bash
git clone https://github.com/yourusername/scoutermail.git
cd scoutermail
npm install
cd frontend && npm install && cd ..
npm run dev
```

The app will compile Rust dependencies on first run (~5 minutes). Subsequent launches are fast.

### Add Your Email Account

1. On first launch, you'll see a quick keybinding tutorial. Press any key to continue.
2. Choose your provider (Gmail, Outlook, Yahoo) or enter custom IMAP/SMTP settings.
3. For Gmail: use an [App Password](https://myaccount.google.com/apppasswords), not your regular password.
4. Click "Test Connection" to verify, then "Save Account".
5. Your inbox will sync automatically.

### Build for Release

```bash
npm run build
```

This produces a `.dmg` installer in `src-tauri/target/release/bundle/dmg/`.

## Project Structure

```
src-tauri/                     Rust backend
  src/
    lib.rs                     App entry point, plugin registration
    commands.rs                Tauri IPC command handlers
    store/
      db.rs                    SQLite schema, migrations, CRUD
      search.rs                Tantivy full-text search index
    imap/
      client.rs                IMAP connection, auth, folders
      sync.rs                  Sync engine (headers-first, background fetch)
      idle.rs                  IMAP IDLE push listener
    smtp/
      client.rs                SMTP sending with TLS
    parser/
      mime.rs                  MIME parsing, attachments, inline images
      threading.rs             RFC 5256 threading engine
    accounts/
      manager.rs               Multi-account config, provider defaults
      keychain.rs              Credential storage
    calendar/
      parser.rs                ICS parsing, RSVP reply builder
    rules/
      engine.rs                Rule conditions, actions, webhook/shell execution

frontend/                      Svelte 5 frontend
  src/
    App.svelte                 Root layout, keybinding wiring
    app.css                    Global dark theme
    lib/
      stores/                  Svelte stores (messages, accounts, UI state)
      keybindings/             Vim keymap engine, binding definitions
      components/
        StatusBar.svelte       Top: account, folder, mode indicator
        HintBar.svelte         Bottom: contextual shortcuts, command input
        MessageList.svelte     Left pane: message list with pagination
        ReadingPane.svelte     Right pane: email view with threads
        ComposeView.svelte     Compose with toolbar, templates, scheduling
        AccountSetup.svelte    First-run account configuration
        SearchBar.svelte       Search input overlay
        HelpOverlay.svelte     Keybindings + commands reference
        CalendarView.svelte    Agenda + month grid calendar
        ContactsList.svelte    Contact directory
        FolderList.svelte      Folder sidebar with unread counts
        InviteCard.svelte      Calendar invite accept/decline
        AttachmentList.svelte  Attachment chips with download
        RuleEditor.svelte      Rules automation editor
        DraftsList.svelte      Resume saved drafts
        SignatureEditor.svelte Email signature editor
        Onboarding.svelte      First-run tutorial
        AddressInput.svelte    Contact autocomplete input
        ToastContainer.svelte  Error/success notifications
```

## Security

- **Content Security Policy** restricts what HTML emails can do &mdash; no external resources, no scripts, no tracking pixels
- **Sandboxed iframe** renders HTML email content in isolation
- **External links** intercepted at the Tauri webview level and opened in your default browser
- **Credentials** stored in a local encrypted file in the app data directory
- **No telemetry**. No analytics. Your email stays on your machine.

## Keyboard Philosophy

ScouterMail is built on the belief that the fastest interface is one you never have to look at. Every feature is reachable from the keyboard. The mouse works too, but it's never required.

The keybinding system supports:
- **Modal editing** &mdash; NORMAL, INSERT, VISUAL, COMMAND modes
- **Multi-key sequences** &mdash; `gi` (go to inbox), `gs` (go to sent), `gg` (jump to top)
- **Command mode** with Tab auto-complete
- **Remappable bindings** (config file support planned)

## Contributing

ScouterMail is in active development. Contributions welcome.

```bash
# Run in development
npm run dev

# Run tests
cd src-tauri && cargo test

# Build release
npm run build
```

## License

MIT
