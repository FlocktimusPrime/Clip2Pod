<script lang="ts">
  import * as api from "$lib/api";
  import { ariaSort, sortIndicator, sortRows, toggleSort, type SortState } from "$lib/sort";
  import { app, toast } from "$lib/stores.svelte";
  import type { LogEntry } from "$lib/types";
  import Modal from "./Modal.svelte";

  type Col = "time" | "status" | "title" | "voice" | "file" | "detail";

  let search = $state("");
  let menu = $state<{ x: number; y: number; voice: string } | null>(null);
  let sort = $state<SortState<Col>>(null);

  const pick = (e: LogEntry, key: Col) =>
    key === "time" ? e.timestamp : key === "file" ? e.filename : e[key];

  const filtered = $derived(
    app.log.filter((e) => e.title.toLowerCase().includes(search.trim().toLowerCase())),
  );

  const sorted = $derived(sortRows(filtered, sort, pick));

  const fmt = (iso: string) =>
    new Date(iso).toLocaleString(undefined, { hour12: false });

  function openMenu(e: MouseEvent, voice: string) {
    e.preventDefault();
    menu = { x: e.clientX, y: e.clientY, voice };
  }

  async function disableVoice() {
    if (!menu) return;
    await api.setVoiceEnabled(menu.voice, false);
    toast(`${menu.voice} removed from rotation`);
    menu = null;
  }

  async function clear() {
    await api.clearLog();
    app.log = [];
  }
</script>

<svelte:window onclick={() => (menu = null)} />

<Modal title="Generation log" onclose={() => (app.dialog = null)}>
  <div class="toolbar">
    <input class="field" placeholder="Search by title…" bind:value={search} />
  </div>

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          {#each [["time", "Time"], ["status", "Status"], ["title", "Title"], ["voice", "Voice"], ["file", "File"], ["detail", "Detail"]] as const as [key, label] (key)}
            <th aria-sort={ariaSort(sort, key)}>
              <button class="th-sort" onclick={() => (sort = toggleSort(sort, key))}>
                {label} <span class="arrow">{sortIndicator(sort, key)}</span>
              </button>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each sorted as entry, i (i)}
          <tr oncontextmenu={(e) => openMenu(e, entry.voice)}>
            <td class="mono">{fmt(entry.timestamp)}</td>
            <td><span class="status" data-status={entry.status}>{entry.status}</span></td>
            <td class="clip" title={entry.title}>{entry.title}</td>
            <td class="mono">{entry.voice}</td>
            <td class="clip mono" title={entry.filename}>{entry.filename}</td>
            <td class="clip" title={entry.detail}>{entry.detail}</td>
          </tr>
        {:else}
          <tr><td colspan="6" class="empty">No log entries yet. Generated jobs land here.</td></tr>
        {/each}
      </tbody>
    </table>
  </div>

  <p class="hint">Right-click an entry to disable the voice that narrated it.</p>

  {#snippet footer()}
    <button class="btn" onclick={clear} disabled={app.log.length === 0}>Clear log</button>
  {/snippet}
</Modal>

{#if menu}
  <div class="ctx" style="left: {menu.x}px; top: {menu.y}px" role="menu">
    <button class="btn" role="menuitem" onclick={disableVoice}>
      Disable {menu.voice}
    </button>
  </div>
{/if}

<style>
  .toolbar {
    margin-bottom: 10px;
  }

  .table-wrap {
    overflow: auto;
    max-height: 50vh;
    border: 1px solid var(--line-soft);
    border-radius: 4px;
  }

  .th-sort {
    font: inherit;
    letter-spacing: inherit;
    text-transform: inherit;
    color: inherit;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  .arrow {
    color: var(--accent);
  }

  .clip {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mono {
    font-family: var(--mono);
    font-size: 11.5px;
  }

  .status {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .status[data-status="Queued"] {
    color: var(--accent);
  }

  .status[data-status="Done"] {
    color: var(--ok);
  }

  .status[data-status="Failed"] {
    color: var(--danger);
  }

  .status[data-status="Cancelled"] {
    color: var(--muted);
  }

  .empty {
    color: var(--muted);
    text-align: center;
    padding: 18px;
  }

  .hint {
    font-size: 11px;
    color: var(--muted);
    margin: 8px 0 0;
  }

  .ctx {
    position: fixed;
    z-index: 60;
    box-shadow: 0 8px 24px var(--shadow);
  }
</style>
