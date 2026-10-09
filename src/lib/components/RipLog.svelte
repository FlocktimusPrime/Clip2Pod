<script lang="ts">
  import { ariaSort, sortIndicator, sortRows, toggleSort, type SortState } from "$lib/sort";
  import { app } from "$lib/stores.svelte";
  import type { RipLogEntry } from "$lib/types";

  // Rip tab of the Log dialog. Stays mounted while the Narrate tab shows, so
  // its search and sort survive switching back.

  type Col = "time" | "status" | "title" | "detail";

  let search = $state("");
  let sort = $state<SortState<Col>>(null);

  const pick = (e: RipLogEntry, key: Col) => (key === "time" ? e.timestamp : e[key]);

  const filtered = $derived(
    app.ripLog.filter((e) => e.title.toLowerCase().includes(search.trim().toLowerCase())),
  );
  const sorted = $derived(sortRows(filtered, sort, pick));

  const fmt = (iso: string) => new Date(iso).toLocaleString(undefined, { hour12: false });
</script>

<div class="toolbar">
  <input
    class="field"
    aria-label="Search Rip log by title"
    placeholder="Search by title…"
    bind:value={search}
  />
</div>

<div class="table-wrap">
  <table>
    <thead>
      <tr>
        {#each [["time", "Time"], ["status", "Status"], ["title", "Title"], ["detail", "Detail"]] as const as [key, label] (key)}
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
        <tr>
          <td class="mono">{fmt(entry.timestamp)}</td>
          <td><span class="status" data-status={entry.status}>{entry.status}</span></td>
          <td class="clip" title={entry.title}>{entry.title}</td>
          <td class="clip" title={entry.detail}>{entry.detail}</td>
        </tr>
      {:else}
        <tr><td colspan="4" class="empty">No log entries yet. Finished rips land here.</td></tr>
      {/each}
    </tbody>
  </table>
</div>

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
    max-width: 220px;
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
</style>
