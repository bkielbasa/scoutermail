# ScouterMail — Calendar Feature Design Spec

## Overview

Parse `.ics` calendar invites from emails, display them inline with accept/decline/tentative RSVP, store events locally, and provide a calendar view (agenda + month grid).

## Backend — ICS Parsing & Storage

### New module: `src-tauri/src/calendar/`

**`parser.rs`:**
- Parse `text/calendar` MIME parts and `.ics` attachments using the `ical` crate
- Extract VEVENT fields: summary, dtstart, dtend, location, description, organizer, attendees, uid, sequence, method (REQUEST/CANCEL/REPLY)
- Convert dtstart/dtend to unix epoch for sorting/querying
- Return a `CalendarEvent` struct

**`store.rs`:**
- Not a separate file — events table added to existing `store/db.rs`

**Integration with sync:**
- During IMAP sync (in `imap/sync.rs`), after parsing each email, check for `text/calendar` parts
- If found, parse the ICS and upsert into the `events` table
- CANCEL method should update event status to `cancelled`

### SQLite Schema Addition

Added to existing `store/db.rs` migration:

```sql
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
    attendees   TEXT,  -- JSON array of {email, name, partstat}
    status      TEXT NOT NULL DEFAULT 'needs-action',
    raw_ics     TEXT
);

CREATE INDEX IF NOT EXISTS idx_events_dtstart ON events(dtstart);
```

### New Tauri Commands

- `get_events()` — all events, ordered by dtstart
- `get_events_in_range(start: i64, end: i64)` — events within epoch range
- `respond_to_invite(event_uid: String, response: String)` — response is "accepted", "declined", or "tentative"
  - Builds an iCalendar REPLY with user's PARTSTAT
  - Sends via SMTP as `text/calendar` attachment to organizer
  - Updates event status in local DB

## Frontend — Invite Display

When viewing an email that has associated calendar event(s), show an **invite card** above the email body in the ReadingPane:

```
┌────────────────────────────────────────────┐
│ 📅 Team Standup                            │
│ Mar 25, 2026 · 10:00 AM – 10:30 AM        │
│ Location: Zoom (https://zoom.us/j/123)     │
│ Organizer: alice@example.com               │
│                                            │
│ [Accept]  [Tentative]  [Decline]           │
└────────────────────────────────────────────┘
```

- Buttons send RSVP and update status
- After responding, show status badge (✓ Accepted / ? Tentative / ✗ Declined)
- Card styled with dark theme, accent border

## Frontend — Calendar View

Opened via `:calendar` command. Replaces reading pane content.

### Agenda View (default)

Chronological list of upcoming events grouped by date:

```
Today — Mar 21
  10:00  Team Standup (30m)
  14:00  Project Review (1h)

Tomorrow — Mar 22
  09:00  1:1 with Alice (30m)

Mar 25
  11:00  Sprint Planning (1h)
```

### Month Grid View

Toggle with `m` key:

```
        March 2026
Mo  Tu  We  Th  Fr  Sa  Su
                        1
2   3   4   5   6   7   8
9   10  11  12  13  14  15
16  17  18  19  20 [21] 22
23  24  25  26  27  28  29
30  31
```

- Today highlighted with accent
- Days with events have a dot indicator
- Navigate months: `h` prev, `l` next
- Select day: `j`/`k` navigate days, `Enter` shows day's events
- `t` jumps to today

### Keybindings (inside calendar view)

| Key | Action |
|-----|--------|
| `m` | Toggle month/agenda view |
| `t` | Jump to today |
| `h` | Previous month (grid) / scroll up (agenda) |
| `l` | Next month (grid) / scroll down (agenda) |
| `j`/`k` | Navigate days (grid) / events (agenda) |
| `Enter` | Show day's events (grid) |
| `Escape` | Back to mail |

## RSVP Flow

1. User clicks Accept/Tentative/Decline on invite card
2. Frontend calls `respond_to_invite(event_uid, response)`
3. Backend:
   a. Loads event from DB, gets raw_ics and organizer
   b. Builds iCalendar REPLY:
      - METHOD: REPLY
      - ATTENDEE with PARTSTAT=ACCEPTED/DECLINED/TENTATIVE
      - Same UID and SEQUENCE as original
   c. Sends email to organizer with `Content-Type: text/calendar; method=REPLY`
   d. Updates event status in DB
4. Frontend updates invite card to show new status

## Rust Dependencies

- Add `ical = "0.11"` to Cargo.toml for ICS parsing
- ICS reply generation is manual string building (no extra crate needed)

## Files Changed

### New:
- `src-tauri/src/calendar/mod.rs`
- `src-tauri/src/calendar/parser.rs`
- `frontend/src/lib/components/InviteCard.svelte`
- `frontend/src/lib/components/CalendarView.svelte`
- `frontend/src/lib/components/AgendaView.svelte`
- `frontend/src/lib/components/MonthGrid.svelte`

### Modified:
- `src-tauri/src/store/db.rs` — add events table, CRUD methods
- `src-tauri/src/commands.rs` — add calendar commands
- `src-tauri/src/lib.rs` — register commands, add calendar module
- `src-tauri/src/imap/sync.rs` — detect and parse ICS during sync
- `src-tauri/src/parser/mime.rs` — expose calendar parts in ParsedEmail
- `src-tauri/Cargo.toml` — add ical crate
- `frontend/src/lib/components/ReadingPane.svelte` — show InviteCard
- `frontend/src/lib/keybindings/bindings.ts` — calendar keybindings
- `frontend/src/App.svelte` — wire :calendar command
