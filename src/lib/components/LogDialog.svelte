<script lang="ts">
  import * as api from "$lib/api";
  import { ariaSort, sortIndicator, sortRows, toggleSort, type SortState } from "$lib/sort";
  import { app, toast } from "$lib/stores.svelte";
  import type { LogEntry } from "$lib/types";
  import Modal from "./Modal.svelte";

  type Col = "time" | "status" | "title" | "voice" | "file" | "detail";

  let search = $state("");
  let menu = $state<{ x: number; y: number; voice: string } | null>(null);
  let menuEl = $state<HTMLDivElement>();
  let sort = $state<SortState<Col>>(null);

  // The menu is a popover only so it clears the top-layer <dialog> it opens
  // inside. Dismissal is handled here (not popover="auto") so Escape closes
  // just the menu, not the dialog behind it.
  $effect(() => {
    if (!menu || !menuEl) return;
    try {
      menuEl.showPopover();
    } catch {
      /* already shown */
    }
    menuEl.querySelector("button")?.focus();

    const onPointer = (e: PointerEvent) => {
      if (!menuEl?.contains(e.target as Node)) menu = null;
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopImmediatePropagation();
        menu = null;
      }
    };
    // next tick so the opening right-click doesn't immediately dismiss it
    const t = setTimeout(() => window.addEventListener("pointerdown", onPointer), 0);
    window.addEventListener("keydown", onKey, true);
    return () => {
      clearTimeout(t);
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey, true);
    };
  });

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

  let removed = $state(false);

  async function disableVoice() {
    if (!menu) return;
    try {
      await api.setVoiceEnabled(menu.voice, false);
      // Confirm in the menu, not a toast — a toast fires behind the dialog.
      removed = true;
      setTimeout(() => {
        menu = null;
        removed = false;
      }, 900);
    } catch (e) {
      toast(String(e), "error");
      menu = null;
    }
  }

  async function clear() {
    await api.clearLog();
    app.log = [];
  }
</script>

<Modal title="Generation log" onclose={() => (app.dialog = null)}>
  <div class="toolbar">
    <input
      class="field"
      aria-label="Search log by title"
      placeholder="Search by title…"
      bind:value={search}
    />
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
  <div
    class="ctx"
    bind:this={menuEl}
    popover="manual"
    style="left: min({menu.x}px, calc(100vw - 220px)); top: min({menu.y}px, calc(100vh - 60px))"
    role="menu"
  >
    <button class="btn" role="menuitem" onclick={disableVoice} disabled={removed}>
      {removed ? "Removed from rotation" : `Disable ${menu.voice}`}
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
    inset: auto;
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
    overflow: visible;
    box-shadow: 0 8px 24px var(--shadow);
  }
</style>
