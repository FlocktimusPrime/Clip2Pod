<script lang="ts">
  import * as api from "$lib/api";
  import { ariaSort, sortIndicator, sortRows, toggleSort, type SortState } from "$lib/sort";
  import { app } from "$lib/stores.svelte";
  import type { Job } from "$lib/types";
  import Modal from "./Modal.svelte";

  type Col = "status" | "title" | "file" | "voice" | "created" | "started" | "finished";

  let sort = $state<SortState<Col>>(null);

  const pick = (j: Job, key: Col) =>
    key === "file" ? j.filename : key === "voice" ? j.voice.short_name : j[key];

  const sorted = $derived(sortRows(app.queue, sort, pick));

  const pendingCount = $derived(app.queue.filter((j) => j.status === "Queued").length);

  const fmt = (iso: string | null) =>
    iso ? new Date(iso).toLocaleTimeString(undefined, { hour12: false }) : "—";
</script>

<Modal title="Generation queue" onclose={() => (app.dialog = null)}>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          {#each [["status", "Status"], ["title", "Title"], ["file", "File"], ["voice", "Voice"], ["created", "Queued"], ["started", "Started"], ["finished", "Finished"]] as const as [key, label] (key)}
            <th aria-sort={ariaSort(sort, key)}>
              <button class="th-sort" onclick={() => (sort = toggleSort(sort, key))}>
                {label} <span class="arrow">{sortIndicator(sort, key)}</span>
              </button>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each sorted as job (job.id)}
          <tr>
            <td><span class="status" data-status={job.status}>{job.status}</span></td>
            <td class="clip" title={job.title}>{job.title}</td>
            <td class="clip mono" title={job.filename}>{job.filename}</td>
            <td class="mono">{job.voice.short_name}</td>
            <td class="mono">{fmt(job.created)}</td>
            <td class="mono">{fmt(job.started)}</td>
            <td class="mono">{fmt(job.finished)}</td>
          </tr>
        {:else}
          <tr><td colspan="7" class="empty">Queue is empty. Generate MP3 adds a job here.</td></tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#snippet footer()}
    <button class="btn" onclick={() => api.clearPending()} disabled={pendingCount === 0}>
      Clear pending ({pendingCount})
    </button>
  {/snippet}
</Modal>

<style>
  .table-wrap {
    overflow: auto;
    max-height: 56vh;
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
    max-width: 160px;
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

  .status[data-status="Processing"] {
    color: var(--accent);
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
