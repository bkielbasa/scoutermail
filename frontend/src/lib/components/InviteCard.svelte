<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface StoredEvent {
    event_uid: string;
    message_uid: number;
    folder: string;
    summary: string | null;
    dtstart: number;
    dtend: number | null;
    location: string | null;
    description: string | null;
    organizer: string | null;
    attendees: string | null;
    sequence: number;
    status: string;
    raw_ics: string | null;
  }

  export let events: StoredEvent[] = [];

  let responding: Record<string, boolean> = {};
  let localStatuses: Record<string, string> = {};

  function formatEventTime(dtstart: number, dtend: number | null): string {
    const start = new Date(dtstart * 1000);
    const dateStr = start.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
    const startTime = start.toLocaleTimeString(undefined, {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
    if (dtend) {
      const end = new Date(dtend * 1000);
      const endTime = end.toLocaleTimeString(undefined, {
        hour: 'numeric',
        minute: '2-digit',
        hour12: true,
      });
      return `${dateStr} \u00B7 ${startTime} \u2013 ${endTime}`;
    }
    return `${dateStr} \u00B7 ${startTime}`;
  }

  function formatOrganizer(org: string | null): string {
    if (!org) return 'Unknown';
    const match = org.match(/^(.+?)\s*<.+>$/);
    if (match) return match[1].replace(/^["']|["']$/g, '').trim();
    return org.replace(/^mailto:/i, '');
  }

  function effectiveStatus(event: StoredEvent): string {
    return localStatuses[event.event_uid] ?? event.status;
  }

  function hasResponded(event: StoredEvent): boolean {
    const s = effectiveStatus(event);
    return s === 'accepted' || s === 'declined' || s === 'tentative';
  }

  function statusLabel(event: StoredEvent): string {
    const s = effectiveStatus(event);
    if (s === 'accepted') return '\u2713 Accepted';
    if (s === 'tentative') return '? Tentative';
    if (s === 'declined') return '\u2717 Declined';
    return '';
  }

  function statusClass(event: StoredEvent): string {
    return effectiveStatus(event);
  }

  async function respond(eventUid: string, response: string): Promise<void> {
    responding = { ...responding, [eventUid]: true };
    try {
      await invoke('respond_to_invite', { eventUid, response });
      localStatuses = { ...localStatuses, [eventUid]: response };
    } catch (e) {
      console.error('Failed to respond to invite:', e);
    } finally {
      responding = { ...responding, [eventUid]: false };
    }
  }
</script>

{#each events as event (event.event_uid)}
  <div class="invite-card">
    <div class="invite-summary">{event.summary || '(No title)'}</div>
    <div class="invite-detail invite-time">{formatEventTime(event.dtstart, event.dtend)}</div>
    {#if event.location}
      <div class="invite-detail invite-location">{event.location}</div>
    {/if}
    <div class="invite-detail invite-organizer">Organizer: {formatOrganizer(event.organizer)}</div>

    {#if hasResponded(event)}
      <div class="invite-status badge-{statusClass(event)}">{statusLabel(event)}</div>
    {:else}
      <div class="invite-actions">
        <button
          class="invite-btn accept"
          disabled={responding[event.event_uid]}
          on:click={() => respond(event.event_uid, 'accepted')}
        >Accept</button>
        <button
          class="invite-btn tentative"
          disabled={responding[event.event_uid]}
          on:click={() => respond(event.event_uid, 'tentative')}
        >Tentative</button>
        <button
          class="invite-btn decline"
          disabled={responding[event.event_uid]}
          on:click={() => respond(event.event_uid, 'declined')}
        >Decline</button>
      </div>
    {/if}
  </div>
{/each}

<style>
  .invite-card {
    background: var(--bg-secondary);
    border-left: 3px solid var(--accent);
    border-radius: 4px;
    padding: 12px 14px;
    margin-bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .invite-summary {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .invite-detail {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .invite-time {
    color: var(--accent);
    font-weight: 500;
  }

  .invite-location {
    font-style: italic;
  }

  .invite-organizer {
    color: var(--text-dim);
  }

  .invite-actions {
    display: flex;
    gap: 8px;
    margin-top: 6px;
  }

  .invite-btn {
    padding: 5px 14px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }

  .invite-btn:hover:not(:disabled) {
    border-color: var(--accent);
  }

  .invite-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .invite-btn.accept:hover:not(:disabled) {
    background: rgba(34, 197, 94, 0.15);
    border-color: #22c55e;
    color: #22c55e;
  }

  .invite-btn.tentative:hover:not(:disabled) {
    background: rgba(234, 179, 8, 0.15);
    border-color: #eab308;
    color: #eab308;
  }

  .invite-btn.decline:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.15);
    border-color: #ef4444;
    color: #ef4444;
  }

  .invite-status {
    margin-top: 6px;
    font-size: 12px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 4px;
    display: inline-block;
    width: fit-content;
  }

  .badge-accepted {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  .badge-tentative {
    background: rgba(234, 179, 8, 0.15);
    color: #eab308;
  }

  .badge-declined {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }
</style>
