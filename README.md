<p align="center">
  <h1 align="center">ScouterMail</h1>
  <p align="center">
    A lightweight, vim-driven desktop email client for macOS, Windows, and Linux.
    <br />
    Built with Rust, Tauri, and Svelte.
  </p>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-000000?style=flat-square" />
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

- **~15MB binary**. No Electron. Uses the native system webview via Tauri.
- **Vim keybindings everywhere**. `j/k` to navigate, `gg`/`G` to jump, `v` for visual select, `:` for commands, `/` to search.
- **Instant full-text search**. Tantivy indexes every message locally. `from:alice subject:project` just works.
- **Offline-first**. All mail synced to local SQLite. Search and read without a connection.
- **Rules engine with superpowers**. Auto-label, auto-move, fire webhooks, run shell commands, or call AI APIs on incoming mail.
- **Cross-platform**. macOS, Windows, and Linux from a single codebase.
- **Dark theme by default**. Because your eyes matter.

## Download

Pre-built binaries are available on the [Releases](https://github.com/bkielbasa/scoutermail/releases) page:

| Platform | Format |
|----------|--------|
| macOS (Apple Silicon) | `.dmg` |
| macOS (Intel) | `.dmg` |
| Linux | `.deb` / `.AppImage` |
| Windows | `.exe` (NSIS installer) |

## Features

### Core Email
- IMAP sync with IDLE push support and automatic retry with exponential backoff
- SMTP sending with STARTTLS (port 587) and implicit TLS (port 465)
- RFC 5256 threading (References + In-Reply-To, subject fallback)
- Full MIME parsing: multipart, attachments, inline images
- HTML email rendering in sandboxed iframe with CSP protection
- Plain text toggle (`h` key)
- Reply-To header handling
- List-Unsubscribe support &mdash; one-click unsubscribe button for newsletters
- Attachment viewing and download to ~/Downloads
- Message snippets in list view for quick scanning

### Vim-Style Interface

Every interaction is keyboard-driven:

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate messages |
| `J` / `K` | Navigate threads |
| `gg` / `G` | Jump to top / bottom |
| `Enter` | Open message (marks as read) |
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
| `?` | Help (tabbed: keybindings + commands) |
| `:` | Command mode (Tab auto-completes) |
| `Tab` | Toggle pane focus |
| `+` / `-` / `=` | Increase / Decrease / Reset reading font size |
| `[` / `]` | Previous / Next page |
| `1`-`9` | Switch accounts |

### Visual Mode

Press `v` to enter visual mode for bulk operations:

| Key | Action |
|-----|--------|
| `j` / `k` | Extend selection down / up |
| `a` | Archive all selected |
| `d` | Delete all selected |
| `Esc` | Cancel selection |

### Command Mode

Type `:` to enter command mode. Tab auto-completes. Suggestions shown in the hint bar.

| Command | Description |
|---------|-------------|
| `:move <folder>` | Move message to IMAP folder |
| `:label <name>` | Add label to message |
| `:unlabel <name>` | Remove label |
| `:labeled <name>` | Show messages with label |
| `:filter unread\|starred\|all` | Quick filter current view |
| `:contacts` | Open contacts list |
| `:calendar` | Open calendar (agenda + month grid) |
| `:folders` | Toggle folder sidebar with unread counts |
| `:drafts` | Show and resume saved drafts |
| `:unified` | Unified inbox (all accounts combined) |
| `:signature` | Edit email signature |
| `:template save\|list\|delete\|<name>` | Manage canned responses |
| `:rules` | Open rules automation editor |
| `:snooze <minutes>` | Snooze message for N minutes |
| `:spam` | Mark as spam (move to Junk) |
| `:print` | Print current email |
| `:backup` | Backup database to timestamped file |
| `:set <key> <value>` | Change setting |

### Calendar

- Parses `.ics` calendar invites from emails automatically
- Accept / Tentative / Decline with RSVP reply sent to organizer via SMTP
- **Agenda view** &mdash; chronological list of upcoming events grouped by date
- **Month grid view** &mdash; traditional calendar with event dot indicators
- Toggle views with `m`, navigate months with `h`/`l`, jump to today with `t`

### Compose

- **Formatting toolbar** &mdash; Bold, Italic, Link, List, Quote, Code
- **Keyboard shortcuts** &mdash; `Ctrl+B` bold, `Ctrl+I` italic, `Ctrl+K` link
- **Contact autocomplete** &mdash; To/Cc/Bcc fields suggest contacts as you type, sorted by frequency
- **Email signature** &mdash; auto-appended on compose/reply/forward, editable via `:signature`
- **Draft auto-save** &mdash; saves every 30 seconds, resume via `:drafts` (`gd`)
- **Schedule send** &mdash; compose now, send later (ISO 8601 datetime or `+N` minutes)
- **Templates** &mdash; save and insert canned responses via `:template`

### Rules Engine

Automate your inbox with conditions and actions. Open the visual editor with `:rules`.

**Conditions** match on from, to, subject, body using contains, equals, regex, or negations. Also supports has_attachment and has_calendar.

**Built-in actions:**
- Add/remove labels
- Move to folder
- Mark read/unread, star, archive, delete
- Auto-reply with a template

**Extended actions:**
- **Webhook** &mdash; POST/GET message data as JSON to any URL
- **Shell** &mdash; run commands with `$MAIL_FROM`, `$MAIL_SUBJECT`, `$MAIL_BODY` env vars
- **AI Prompt** &mdash; send to an LLM API for auto-labeling, summarization, or smart replies

Rules execute automatically on every sync against new messages.

Example: *"If from contains `invoice` → add label `finance` → webhook POST to accounting API"*

### Multi-Account

- Support for multiple IMAP/SMTP accounts
- Quick switch with `1`-`9` number keys
- Unified inbox (`:unified`) merges all accounts sorted by date
- Per-account color coding (9-color palette auto-assigned)
- Provider auto-fill for Gmail, Outlook, Yahoo

### Search

Powered by [Tantivy](https://github.com/quickwit-oss/tantivy) full-text search:

```
/meeting notes                    # search everywhere
/from:alice                       # search by sender
/subject:invoice from:finance     # combine fields
```

### Organization

- **Labels** &mdash; tag messages with `:label`, filter by label with `:labeled`
- **Quick filters** &mdash; `:filter unread`, `:filter starred`, `:filter all`
- **Snooze** &mdash; hide messages and resurface later (`:snooze 60` for 1 hour)
- **Folder list** &mdash; browse all IMAP folders with unread counts (`:folders`)
- **Pagination** &mdash; 50 messages per page, `[`/`]` to navigate
- **Unsubscribe** &mdash; one-click unsubscribe button for newsletter emails

### Notifications

- **Native OS notifications** for new mail (single message shows sender + subject, multiple shows count)
- **Error toasts** &mdash; bottom-right notifications for failed operations
- **Loading indicator** &mdash; "syncing..." in status bar during sync

### Performance

- **Lazy body loading** &mdash; message list loads headers only, body fetched on demand
- **Background flag sync** &mdash; mark-as-read and star sync to IMAP without blocking UI
- **IMAP retry** &mdash; exponential backoff (3 retries) on connection failures
- **Pagination** &mdash; handles large folders without loading everything into memory
- **Database backup** &mdash; `:backup` creates timestamped SQLite backup

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri v2](https://v2.tauri.app) (~15MB binary, native system webview) |
| Backend | Rust |
| Frontend | Svelte 5, TypeScript, Vite |
| Local storage | SQLite via rusqlite |
| Full-text search | Tantivy |
| Email protocols | async-imap, lettre (SMTP) |
| Email parsing | mailparse |
| Calendar | ical crate |
| HTTP client | reqwest (for webhook actions) |
| Regex | regex crate (for rule conditions) |
| Notifications | tauri-plugin-notification |
| External links | tauri-plugin-shell |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- **Linux only:** `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libssl-dev`

### Install and Run

```bash
git clone https://github.com/bkielbasa/scoutermail.git
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

Produces platform-specific installers in `src-tauri/target/release/bundle/`:
- **macOS:** `.dmg` in `bundle/dmg/`
- **Linux:** `.deb` in `bundle/deb/`, `.AppImage` in `bundle/appimage/`
- **Windows:** `.exe` in `bundle/nsis/`

## Project Structure

```
src-tauri/                     Rust backend
  src/
    lib.rs                     App entry, plugin registration, navigation handler
    commands.rs                Tauri IPC command handlers (~30 commands)
    store/
      db.rs                    SQLite schema, migrations, CRUD operations
      search.rs                Tantivy full-text search index
    imap/
      client.rs                IMAP connection with retry, auth, folder ops
      sync.rs                  Sync engine (headers-first, background fetch)
      idle.rs                  IMAP IDLE push listener
    smtp/
      client.rs                SMTP sending (STARTTLS + implicit TLS)
    parser/
      mime.rs                  MIME parsing, attachments, inline images, unsubscribe
      threading.rs             RFC 5256 threading engine
    accounts/
      manager.rs               Multi-account config, provider defaults
      keychain.rs              Credential storage (file-based)
    calendar/
      parser.rs                ICS parsing, RSVP reply builder
    rules/
      engine.rs                Rule conditions, actions, webhook/shell/AI execution

frontend/                      Svelte 5 frontend
  src/
    App.svelte                 Root layout, keybinding wiring, view routing
    app.css                    Global dark theme (CSS custom properties)
    lib/
      stores/
        messages.ts            Message list, selection, pagination, notifications
        accounts.ts            Accounts, folders, unread counts, colors
        ui.ts                  Mode, focus, search, font size, filters
        toast.ts               Error/success toast notifications
      keybindings/
        engine.ts              Vim keymap engine with Tab auto-complete
        bindings.ts            Default binding definitions
      components/
        StatusBar.svelte       Top: account, folder, mode, unread count, loading
        HintBar.svelte         Bottom: contextual shortcuts, command suggestions
        MessageList.svelte     Left pane: messages with pagination and snippets
        ReadingPane.svelte     Right pane: email view with threads, unsubscribe
        ComposeView.svelte     Compose: toolbar, templates, scheduling, drafts
        AccountSetup.svelte    First-run account configuration
        SearchBar.svelte       Search input overlay
        HelpOverlay.svelte     Tabbed: keybindings + commands + current state
        CalendarView.svelte    Agenda + month grid calendar
        ContactsList.svelte    Contact directory with frequency sorting
        FolderList.svelte      Folder sidebar with unread counts
        InviteCard.svelte      Calendar invite accept/decline/tentative
        AttachmentList.svelte  Attachment chips with download
        RuleEditor.svelte      Visual rules editor (conditions + actions)
        DraftsList.svelte      Resume saved drafts
        SignatureEditor.svelte Email signature editor
        Onboarding.svelte      First-run keybinding tutorial
        AddressInput.svelte    Contact autocomplete input
        ToastContainer.svelte  Error/success/info notifications
```

## Security

- **Content Security Policy** &mdash; strict CSP blocks external resources, scripts, and tracking pixels from HTML emails
- **Sandboxed iframe** &mdash; HTML email content rendered in isolation with `allow-same-origin` only
- **External links** &mdash; intercepted at the Tauri webview level and opened in your default browser
- **Credentials** &mdash; stored in a local file in the app data directory
- **No telemetry** &mdash; no analytics, no tracking, no phone-home. Your email stays on your machine.

## Keyboard Philosophy

ScouterMail is built on the belief that the fastest interface is one you never have to look at. Every feature is reachable from the keyboard. The mouse works too, but it's never required.

The keybinding system supports:
- **Modal editing** &mdash; NORMAL, INSERT, VISUAL, COMMAND modes
- **Multi-key sequences** &mdash; `gi` (go to inbox), `gs` (go to sent), `gg` (jump to top)
- **Command mode** &mdash; Tab auto-complete with suggestions in the hint bar
- **Contextual hints** &mdash; hint bar shows relevant shortcuts for the current mode

## CI/CD

GitHub Actions workflows included:
- **CI** &mdash; runs on every push/PR: tests + build check on macOS, Linux, Windows
- **Release** &mdash; triggered by `git tag v*`: builds all platforms, creates GitHub Release with artifacts

To create a release:
```bash
git tag v0.2.0
git push origin v0.2.0
```

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
