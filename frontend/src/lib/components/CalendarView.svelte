<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
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

  interface DayGroup {
    dateKey: string;
    label: string;
    events: StoredEvent[];
  }

  let viewMode: 'agenda' | 'month' = 'agenda';
  let events: StoredEvent[] = [];
  let loading = true;

  // Month view state
  const now = new Date();
  let currentYear = now.getFullYear();
  let currentMonth = now.getMonth();
  let selectedDay: number | null = null;
  let monthEvents: StoredEvent[] = [];

  const WEEKDAYS = ['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su'];

  function formatTime(epoch: number): string {
    return new Date(epoch * 1000).toLocaleTimeString(undefined, {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  }

  function formatDuration(dtstart: number, dtend: number | null): string {
    if (!dtend) return '';
    const mins = Math.round((dtend - dtstart) / 60);
    if (mins < 60) return `${mins}m`;
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return m > 0 ? `${h}h ${m}m` : `${h}h`;
  }

  function groupByDate(evts: StoredEvent[]): DayGroup[] {
    const sorted = [...evts].sort((a, b) => a.dtstart - b.dtstart);
    const groups: Map<string, DayGroup> = new Map();

    for (const evt of sorted) {
      const d = new Date(evt.dtstart * 1000);
      const dateKey = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
      if (!groups.has(dateKey)) {
        const label = d.toLocaleDateString(undefined, {
          weekday: 'long',
          month: 'long',
          day: 'numeric',
          year: 'numeric',
        });
        groups.set(dateKey, { dateKey, label, events: [] });
      }
      groups.get(dateKey)!.events.push(evt);
    }

    return Array.from(groups.values());
  }

  function getMonthDays(): (number | null)[] {
    const firstDay = new Date(currentYear, currentMonth, 1);
    const daysInMonth = new Date(currentYear, currentMonth + 1, 0).getDate();
    // Monday = 0, Sunday = 6
    let startWeekday = firstDay.getDay() - 1;
    if (startWeekday < 0) startWeekday = 6;

    const cells: (number | null)[] = [];
    for (let i = 0; i < startWeekday; i++) cells.push(null);
    for (let d = 1; d <= daysInMonth; d++) cells.push(d);
    // Pad to fill the last row
    while (cells.length % 7 !== 0) cells.push(null);
    return cells;
  }

  function isToday(day: number): boolean {
    const t = new Date();
    return day === t.getDate() && currentMonth === t.getMonth() && currentYear === t.getFullYear();
  }

  function dayHasEvents(day: number): boolean {
    return monthEvents.some((e) => {
      const d = new Date(e.dtstart * 1000);
      return d.getDate() === day && d.getMonth() === currentMonth && d.getFullYear() === currentYear;
    });
  }

  function eventsForDay(day: number): StoredEvent[] {
    return monthEvents
      .filter((e) => {
        const d = new Date(e.dtstart * 1000);
        return d.getDate() === day && d.getMonth() === currentMonth && d.getFullYear() === currentYear;
      })
      .sort((a, b) => a.dtstart - b.dtstart);
  }

  function monthLabel(): string {
    return new Date(currentYear, currentMonth, 1).toLocaleDateString(undefined, {
      month: 'long',
      year: 'numeric',
    });
  }

  function prevMonth(): void {
    if (currentMonth === 0) {
      currentMonth = 11;
      currentYear--;
    } else {
      currentMonth--;
    }
    selectedDay = null;
    loadMonthEvents();
  }

  function nextMonth(): void {
    if (currentMonth === 11) {
      currentMonth = 0;
      currentYear++;
    } else {
      currentMonth++;
    }
    selectedDay = null;
    loadMonthEvents();
  }

  async function loadAllEvents(): Promise<void> {
    try {
      events = await invoke<StoredEvent[]>('get_events');
    } catch (e) {
      console.error('Failed to load events:', e);
      events = [];
    }
  }

  async function loadMonthEvents(): Promise<void> {
    try {
      const start = Math.floor(new Date(currentYear, currentMonth, 1).getTime() / 1000);
      const end = Math.floor(new Date(currentYear, currentMonth + 1, 0, 23, 59, 59).getTime() / 1000);
      monthEvents = await invoke<StoredEvent[]>('get_events_in_range', { start, end });
    } catch (e) {
      console.error('Failed to load month events:', e);
      monthEvents = [];
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    // Only handle keys when no input/textarea is focused
    const tag = (event.target as HTMLElement)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

    if (event.key === 'm') {
      event.stopPropagation();
      event.preventDefault();
      if (viewMode === 'agenda') {
        viewMode = 'month';
        loadMonthEvents();
      } else {
        viewMode = 'agenda';
      }
    } else if (viewMode === 'month') {
      if (event.key === 'h') {
        event.stopPropagation();
        event.preventDefault();
        prevMonth();
      } else if (event.key === 'l') {
        event.stopPropagation();
        event.preventDefault();
        nextMonth();
      } else if (event.key === 't') {
        event.stopPropagation();
        event.preventDefault();
        currentYear = now.getFullYear();
        currentMonth = now.getMonth();
        selectedDay = null;
        loadMonthEvents();
      }
    }
  }

  onMount(async () => {
    window.addEventListener('keydown', handleKeydown, true);
    loading = true;
    await loadAllEvents();
    loading = false;
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown, true);
  });

  $: dayGroups = groupByDate(events);
  $: monthDays = getMonthDays();
  $: selectedDayEvents = selectedDay ? eventsForDay(selectedDay) : [];
</script>

<div class="calendar-view">
  <div class="calendar-header">
    <h2 class="calendar-title">Calendar</h2>
    <div class="view-toggle">
      <button
        class="toggle-btn"
        class:active={viewMode === 'agenda'}
        on:click={() => { viewMode = 'agenda'; }}
      >Agenda</button>
      <button
        class="toggle-btn"
        class:active={viewMode === 'month'}
        on:click={() => { viewMode = 'month'; loadMonthEvents(); }}
      >Month</button>
    </div>
    <span class="hint-text">m: toggle view{viewMode === 'month' ? ' | h/l: prev/next month | t: today' : ''}</span>
  </div>

  {#if loading}
    <p class="calendar-empty">Loading events...</p>
  {:else if viewMode === 'agenda'}
    {#if dayGroups.length === 0}
      <p class="calendar-empty">No upcoming events</p>
    {:else}
      <div class="agenda-list">
        {#each dayGroups as group (group.dateKey)}
          <div class="day-group">
            <div class="day-header">{group.label}</div>
            {#each group.events as event (event.event_uid)}
              <div class="agenda-event">
                <div class="event-time">{formatTime(event.dtstart)}</div>
                <div class="event-info">
                  <div class="event-summary">{event.summary || '(No title)'}</div>
                  <div class="event-meta">
                    {#if event.dtend}
                      <span class="event-duration">{formatDuration(event.dtstart, event.dtend)}</span>
                    {/if}
                    {#if event.location}
                      <span class="event-location">{event.location}</span>
                    {/if}
                  </div>
                </div>
                {#if event.status !== 'needs-action'}
                  <div class="event-status status-{event.status}">{event.status}</div>
                {/if}
              </div>
            {/each}
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    <div class="month-view">
      <div class="month-nav">
        <button class="nav-btn" on:click={prevMonth}>&lsaquo;</button>
        <span class="month-label">{monthLabel()}</span>
        <button class="nav-btn" on:click={nextMonth}>&rsaquo;</button>
      </div>

      <div class="month-grid">
        {#each WEEKDAYS as wd}
          <div class="weekday-header">{wd}</div>
        {/each}
        {#each monthDays as day}
          {#if day === null}
            <div class="day-cell empty"></div>
          {:else}
            <button
              class="day-cell"
              class:today={isToday(day)}
              class:selected={selectedDay === day}
              class:has-events={dayHasEvents(day)}
              on:click={() => { selectedDay = selectedDay === day ? null : day; }}
            >
              <span class="day-number">{day}</span>
              {#if dayHasEvents(day)}
                <span class="event-dot"></span>
              {/if}
            </button>
          {/if}
        {/each}
      </div>

      {#if selectedDay !== null}
        <div class="day-events">
          <div class="day-events-header">
            {new Date(currentYear, currentMonth, selectedDay).toLocaleDateString(undefined, {
              weekday: 'long',
              month: 'long',
              day: 'numeric',
            })}
          </div>
          {#if selectedDayEvents.length === 0}
            <p class="calendar-empty">No events this day</p>
          {:else}
            {#each selectedDayEvents as event (event.event_uid)}
              <div class="agenda-event">
                <div class="event-time">{formatTime(event.dtstart)}</div>
                <div class="event-info">
                  <div class="event-summary">{event.summary || '(No title)'}</div>
                  <div class="event-meta">
                    {#if event.dtend}
                      <span class="event-duration">{formatDuration(event.dtstart, event.dtend)}</span>
                    {/if}
                    {#if event.location}
                      <span class="event-location">{event.location}</span>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .calendar-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 16px 20px;
    overflow: hidden;
  }

  .calendar-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .calendar-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .view-toggle {
    display: flex;
    gap: 2px;
    background: var(--bg-tertiary);
    border-radius: 4px;
    padding: 2px;
  }

  .toggle-btn {
    padding: 4px 12px;
    border: none;
    border-radius: 3px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
  }

  .toggle-btn.active {
    background: var(--accent);
    color: white;
  }

  .hint-text {
    font-size: 11px;
    color: var(--text-dim);
    margin-left: auto;
    font-family: var(--font-mono);
  }

  .calendar-empty {
    color: var(--text-dim);
    text-align: center;
    padding: 24px;
  }

  /* Agenda */
  .agenda-list {
    flex: 1;
    overflow-y: auto;
  }

  .day-group {
    margin-bottom: 16px;
  }

  .day-header {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
    padding: 4px 0;
    margin-bottom: 4px;
    border-bottom: 1px solid var(--border);
  }

  .agenda-event {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 6px;
    border-radius: 4px;
  }

  .agenda-event:hover {
    background: var(--bg-secondary);
  }

  .event-time {
    font-size: 12px;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    width: 80px;
    flex-shrink: 0;
    padding-top: 1px;
  }

  .event-info {
    flex: 1;
    min-width: 0;
  }

  .event-summary {
    font-size: 13px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .event-meta {
    display: flex;
    gap: 10px;
    font-size: 11px;
    color: var(--text-dim);
    margin-top: 2px;
  }

  .event-location::before {
    content: '\1F4CD ';
  }

  .event-status {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    flex-shrink: 0;
    text-transform: capitalize;
  }

  .status-accepted {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  .status-tentative {
    background: rgba(234, 179, 8, 0.15);
    color: #eab308;
  }

  .status-declined {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }

  /* Month view */
  .month-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .month-nav {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    margin-bottom: 12px;
  }

  .nav-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    width: 28px;
    height: 28px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: inherit;
  }

  .nav-btn:hover {
    border-color: var(--accent);
  }

  .month-label {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    min-width: 160px;
    text-align: center;
  }

  .month-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }

  .weekday-header {
    text-align: center;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
    padding: 6px 0;
  }

  .day-cell {
    aspect-ratio: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    background: var(--bg-secondary);
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    color: var(--text-primary);
    font-size: 13px;
    position: relative;
  }

  .day-cell.empty {
    background: transparent;
    cursor: default;
  }

  .day-cell:not(.empty):hover {
    border-color: var(--border);
    background: var(--bg-tertiary);
  }

  .day-cell.today {
    border-color: var(--accent);
    background: var(--accent-dim);
  }

  .day-cell.today .day-number {
    color: var(--accent);
    font-weight: 700;
  }

  .day-cell.selected {
    border-color: var(--accent);
    background: var(--bg-tertiary);
  }

  .event-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
  }

  .day-events {
    margin-top: 12px;
    border-top: 1px solid var(--border);
    padding-top: 8px;
  }

  .day-events-header {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }
</style>
