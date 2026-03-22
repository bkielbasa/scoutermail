<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Rule {
    rule_id: number | null;
    name: string;
    enabled: boolean;
    conditions: string;
    actions: string;
    created_at: number;
  }

  interface Condition {
    field: string;
    op: string;
    value: string;
  }

  interface RuleAction {
    type: string;
    value?: string;
    url?: string;
    method?: string;
    command?: string;
    prompt?: string;
    action?: string;
  }

  const CONDITION_FIELDS = ['from', 'to', 'subject', 'body', 'has_attachment', 'has_calendar'];
  const CONDITION_OPS = ['contains', 'not_contains', 'equals', 'not_equals', 'regex'];
  const ACTION_TYPES = [
    'add_label', 'remove_label', 'move_to_folder',
    'mark_read', 'mark_unread', 'star', 'archive', 'delete', 'mark_spam',
    'auto_reply', 'webhook', 'shell', 'ai_prompt',
  ];
  const NO_VALUE_ACTIONS = ['mark_read', 'mark_unread', 'star', 'archive', 'delete', 'mark_spam'];
  const NO_VALUE_FIELDS = ['has_attachment', 'has_calendar'];

  let rules: Rule[] = [];
  let loading = true;
  let editing = false;
  let editRule: Rule | null = null;
  let editConditions: Condition[] = [];
  let editActions: RuleAction[] = [];
  let editName = '';

  onMount(async () => {
    await loadRules();
  });

  async function loadRules() {
    loading = true;
    try {
      rules = await invoke<Rule[]>('get_rules');
    } catch (e) {
      console.error('Failed to load rules:', e);
      rules = [];
    } finally {
      loading = false;
    }
  }

  function startNew() {
    editRule = null;
    editName = '';
    editConditions = [{ field: 'from', op: 'contains', value: '' }];
    editActions = [{ type: 'add_label', value: '' }];
    editing = true;
  }

  function startEdit(rule: Rule) {
    editRule = rule;
    editName = rule.name;
    try {
      editConditions = JSON.parse(rule.conditions);
      if (!editConditions.length) editConditions = [{ field: 'from', op: 'contains', value: '' }];
    } catch {
      editConditions = [{ field: 'from', op: 'contains', value: '' }];
    }
    try {
      editActions = JSON.parse(rule.actions);
      if (!editActions.length) editActions = [{ type: 'add_label', value: '' }];
    } catch {
      editActions = [{ type: 'add_label', value: '' }];
    }
    editing = true;
  }

  function cancelEdit() {
    editing = false;
    editRule = null;
  }

  function addCondition() {
    editConditions = [...editConditions, { field: 'from', op: 'contains', value: '' }];
  }

  function removeCondition(index: number) {
    editConditions = editConditions.filter((_, i) => i !== index);
  }

  function addAction() {
    editActions = [...editActions, { type: 'add_label', value: '' }];
  }

  function removeAction(index: number) {
    editActions = editActions.filter((_, i) => i !== index);
  }

  async function saveRule() {
    const rule: Rule = {
      rule_id: editRule?.rule_id ?? null,
      name: editName,
      enabled: editRule?.enabled ?? true,
      conditions: JSON.stringify(editConditions),
      actions: JSON.stringify(editActions),
      created_at: editRule?.created_at ?? Math.floor(Date.now() / 1000),
    };
    try {
      await invoke('save_rule', { rule });
      editing = false;
      editRule = null;
      await loadRules();
    } catch (e) {
      console.error('Failed to save rule:', e);
    }
  }

  async function deleteRule(ruleId: number) {
    try {
      await invoke('delete_rule', { ruleId });
      await loadRules();
    } catch (e) {
      console.error('Failed to delete rule:', e);
    }
  }

  async function toggleRule(ruleId: number, enabled: boolean) {
    try {
      await invoke('toggle_rule', { ruleId, enabled });
      await loadRules();
    } catch (e) {
      console.error('Failed to toggle rule:', e);
    }
  }

  function summarizeConditions(conditionsJson: string): string {
    try {
      const conds: Condition[] = JSON.parse(conditionsJson);
      return conds.map((c) => `${c.field} ${c.op} ${c.value || ''}`).join(', ');
    } catch {
      return '';
    }
  }

  function summarizeActions(actionsJson: string): string {
    try {
      const acts: RuleAction[] = JSON.parse(actionsJson);
      return acts.map((a) => a.type + (a.value ? `: ${a.value}` : '')).join(', ');
    } catch {
      return '';
    }
  }
</script>

<div class="rules-view">
  {#if editing}
    <div class="rules-header">
      <h2 class="rules-title">{editRule ? 'Edit Rule' : 'New Rule'}</h2>
    </div>
    <div class="rule-form">
      <label class="form-label">
        Name
        <input class="form-input" type="text" bind:value={editName} placeholder="Rule name" />
      </label>

      <div class="form-section">
        <div class="form-section-title">Conditions (all must match)</div>
        {#each editConditions as cond, i}
          <div class="form-row">
            <select class="form-select" bind:value={cond.field}>
              {#each CONDITION_FIELDS as f}
                <option value={f}>{f}</option>
              {/each}
            </select>
            <select class="form-select" bind:value={cond.op}>
              {#each CONDITION_OPS as op}
                <option value={op}>{op}</option>
              {/each}
            </select>
            {#if !NO_VALUE_FIELDS.includes(cond.field)}
              <input class="form-input flex-grow" type="text" bind:value={cond.value} placeholder="Value" />
            {/if}
            <button class="btn-icon" on:click={() => removeCondition(i)} title="Remove condition">&times;</button>
          </div>
        {/each}
        <button class="btn-link" on:click={addCondition}>+ Add condition</button>
      </div>

      <div class="form-section">
        <div class="form-section-title">Actions (execute in order)</div>
        {#each editActions as act, i}
          <div class="form-row">
            <select class="form-select" bind:value={act.type}>
              {#each ACTION_TYPES as t}
                <option value={t}>{t}</option>
              {/each}
            </select>

            {#if ['add_label', 'remove_label', 'move_to_folder', 'auto_reply'].includes(act.type)}
              <input class="form-input flex-grow" type="text" bind:value={act.value} placeholder="Value" />
            {:else if act.type === 'webhook'}
              <input class="form-input flex-grow" type="text" bind:value={act.url} placeholder="URL" />
              <select class="form-select form-select-sm" bind:value={act.method}>
                <option value="GET">GET</option>
                <option value="POST">POST</option>
              </select>
            {:else if act.type === 'shell'}
              <input class="form-input flex-grow" type="text" bind:value={act.command} placeholder="Command" />
            {:else if act.type === 'ai_prompt'}
              <textarea class="form-textarea flex-grow" bind:value={act.prompt} placeholder="Prompt" rows="2"></textarea>
              <select class="form-select form-select-sm" bind:value={act.action}>
                <option value="add_label">add_label</option>
                <option value="auto_reply">auto_reply</option>
              </select>
            {/if}

            <button class="btn-icon" on:click={() => removeAction(i)} title="Remove action">&times;</button>
          </div>
        {/each}
        <button class="btn-link" on:click={addAction}>+ Add action</button>
      </div>

      <div class="form-buttons">
        <button class="btn-primary" on:click={saveRule}>Save</button>
        <button class="btn-secondary" on:click={cancelEdit}>Cancel</button>
      </div>
    </div>
  {:else}
    <div class="rules-header">
      <h2 class="rules-title">Rules</h2>
      <button class="btn-primary" on:click={startNew}>New Rule</button>
    </div>

    {#if loading}
      <p class="rules-empty">Loading...</p>
    {:else if rules.length === 0}
      <p class="rules-empty">No rules configured</p>
    {:else}
      <div class="rules-list">
        {#each rules as rule}
          <div class="rule-item" class:disabled={!rule.enabled}>
            <label class="toggle-wrap">
              <input
                type="checkbox"
                checked={rule.enabled}
                on:change={() => rule.rule_id !== null && toggleRule(rule.rule_id, !rule.enabled)}
              />
              <span class="toggle-slider"></span>
            </label>
            <div class="rule-info">
              <div class="rule-name">{rule.name}</div>
              <div class="rule-summary">
                {summarizeConditions(rule.conditions)} &rarr; {summarizeActions(rule.actions)}
              </div>
            </div>
            <div class="rule-actions">
              <button class="btn-icon" on:click={() => startEdit(rule)} title="Edit">&#9998;</button>
              <button class="btn-icon btn-danger" on:click={() => rule.rule_id !== null && deleteRule(rule.rule_id)} title="Delete">&times;</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .rules-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 16px 20px;
    overflow: hidden;
  }

  .rules-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .rules-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .rules-empty {
    color: var(--text-dim);
    text-align: center;
    padding: 24px;
  }

  .rules-list {
    flex: 1;
    overflow-y: auto;
  }

  .rule-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 4px;
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .rule-item:hover {
    background: var(--bg-secondary);
  }

  .rule-item.disabled {
    opacity: 0.5;
  }

  .rule-info {
    flex: 1;
    min-width: 0;
  }

  .rule-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .rule-summary {
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .rule-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  /* Toggle switch */
  .toggle-wrap {
    position: relative;
    display: inline-block;
    width: 34px;
    height: 18px;
    flex-shrink: 0;
    cursor: pointer;
  }

  .toggle-wrap input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }

  .toggle-slider {
    position: absolute;
    inset: 0;
    background: var(--bg-tertiary);
    border-radius: 9px;
    transition: background 0.2s;
  }

  .toggle-slider::before {
    content: '';
    position: absolute;
    width: 14px;
    height: 14px;
    left: 2px;
    top: 2px;
    background: var(--text-dim);
    border-radius: 50%;
    transition: transform 0.2s, background 0.2s;
  }

  .toggle-wrap input:checked + .toggle-slider {
    background: var(--accent);
  }

  .toggle-wrap input:checked + .toggle-slider::before {
    transform: translateX(16px);
    background: var(--bg-primary);
  }

  /* Form styles */
  .rule-form {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .form-input {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 12px;
    outline: none;
  }

  .form-input:focus {
    border-color: var(--accent);
  }

  .form-textarea {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 12px;
    outline: none;
    resize: vertical;
    font-family: inherit;
  }

  .form-textarea:focus {
    border-color: var(--accent);
  }

  .form-select {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 6px 8px;
    border-radius: 4px;
    font-size: 12px;
    outline: none;
    cursor: pointer;
  }

  .form-select:focus {
    border-color: var(--accent);
  }

  .form-select-sm {
    width: 90px;
    flex-shrink: 0;
  }

  .form-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-section-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .form-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .flex-grow {
    flex: 1;
    min-width: 0;
  }

  .form-buttons {
    display: flex;
    gap: 8px;
    padding-top: 8px;
  }

  .btn-primary {
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    padding: 6px 16px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    padding: 6px 16px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .btn-secondary:hover {
    background: var(--bg-tertiary);
  }

  .btn-icon {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 16px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    line-height: 1;
  }

  .btn-icon:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  .btn-danger:hover {
    color: #e06c75;
  }

  .btn-link {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 12px;
    cursor: pointer;
    padding: 4px 0;
    text-align: left;
  }

  .btn-link:hover {
    text-decoration: underline;
  }
</style>
