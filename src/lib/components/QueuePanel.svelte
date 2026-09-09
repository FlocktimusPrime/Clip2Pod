<script lang="ts">
  import * as api from "$lib/api";
  import { app, toast } from "$lib/stores.svelte";

  const processing = $derived(app.queue.find((j) => j.status === "Processing"));
  const waiting = $derived(app.queue.filter((j) => j.status === "Queued"));
  const failed = $derived(app.queue.filter((j) => j.status === "Failed"));

  // Queue is in-memory (empty after a restart); fall back to the persisted log.
  const lastDone = $derived(
    app.queue.findLast((j) => j.status === "Done")?.title ??
      app.log.find((e) => e.status === "Done")?.title ??
      null,
  );

  const waitingTitles = $derived(waiting.map((j) => j.title).join(", "));
  // Nothing to report at all — not even a past success to name.
  const quiet = $derived(!processing && waiting.length === 0 && failed.length === 0 && !lastDone);

  const frac = $derived(
    app.renderProgress && app.renderProgress.total
      ? Math.min(1, app.renderProgress.done / app.renderProgress.total)
      : 0,
  );
  const pct = $derived(frac ? ` (${Math.round(frac * 100)}%)` : "");

  let cancelling = $state(false);
  // Cancel lands at the next chunk boundary; reset the label once the job clears.
  $effect(() => {
    if (!processing) cancelling = false;
  });

  function cancel() {
    cancelling = true;
    api.cancelCurrent();
  }

  /** Episode count pending delete confirmation; null = no confirm active. */
  let confirmCount = $state<number | null>(null);
  let deleting = $state(false);

  // Keep keyboard focus with the confirm flow: land on the safe choice (Keep)
  // when it opens, return to the trigger when it closes rather than dropping
  // to <body>.
  let keepBtn = $state<HTMLButtonElement>();
  let deleteBtn = $state<HTMLButtonElement>();
  let confirmWasOpen = false;
  $effect(() => {
    if (confirmCount !== null) {
      confirmWasOpen = true;
      keepBtn?.focus();
    } else if (confirmWasOpen) {
      confirmWasOpen = false;
      deleteBtn?.focus();
    }
  });

  async function askDeleteEpisodes() {
    try {
      const count = await api.episodeCount();
      if (count === 0) {
        toast("No episodes in output folder");
        return;
      }
      confirmCount = count;
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function confirmDeleteEpisodes() {
    deleting = true;
    try {
      const deleted = await api.deleteAllEpisodes();
      toast(`Deleted ${deleted} episode${deleted === 1 ? "" : "s"}`);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      deleting = false;
      confirmCount = null;
    }
  }
</script>

<footer class="queue-bar" aria-label="Queue">
  {#if processing}
    <div class="rail" style="--p: {frac}" aria-hidden="true"></div>
  {/if}
  <span class="label">Queue</span>

  <div class="status">
    {#if quiet}
      <span class="item muted no-dot">Idle</span>
    {/if}

    {#if processing}
      <span class="item proc" title={processing.title}>
        Rendering{pct} — {processing.title}
      </span>
    {/if}

    {#if waiting.length}
      <span class="item muted" title={waitingTitles}>
        {waiting.length} waiting — {waitingTitles}
      </span>
    {/if}

    {#each failed as job (job.id)}
      <span class="item fail" title={`${job.title} — ${job.detail}`}>
        {job.title} — {job.detail}
      </span>
    {/each}

    {#if lastDone}
      <span class="item done" title={lastDone}>Last — {lastDone}</span>
    {/if}
  </div>

  <div class="actions">
    {#if processing}
      <button class="btn" onclick={cancel} disabled={cancelling}>
        {cancelling ? "Cancelling…" : "Cancel"}
      </button>
    {/if}
    {#if waiting.length}
      <button class="btn" onclick={() => api.clearPending()}>Clear pending ({waiting.length})</button>
    {/if}
    {#if confirmCount === null}
      <button class="btn" bind:this={deleteBtn} onclick={askDeleteEpisodes}>Delete episodes</button>
    {:else}
      <span class="confirm" role="alert">
        Delete {confirmCount} episode{confirmCount === 1 ? "" : "s"}?
      </span>
      <button class="btn" onclick={confirmDeleteEpisodes} disabled={deleting}>
        {deleting ? "Deleting…" : "Delete"}
      </button>
      <button class="btn" bind:this={keepBtn} onclick={() => (confirmCount = null)} disabled={deleting}>
        Keep
      </button>
    {/if}
  </div>
</footer>

<style>
  .queue-bar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 16px;
    background: var(--panel);
    border-top: 1px solid var(--line);
    font-family: var(--mono);
    font-size: 11px;
  }

  /* Render progress reads as the top edge "charging" left-to-right, easing
     toward each chunk's mark so discrete jumps become continuous. */
  .rail {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
    transform: scaleX(var(--p, 0));
    transform-origin: left;
    transition: transform 400ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .queue-bar > .label {
    flex-shrink: 0;
  }

  .status {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px 18px;
    /* failures push the strip taller; cap it and scroll rather than shove
       the editor around, but give a couple of wrapped errors room first. */
    max-height: 92px;
    overflow-y: auto;
  }

  .item {
    position: relative;
    padding-left: 14px;
    color: var(--text);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* One marker vocabulary across the app: the status-lamp bulb, the toast
     dot, and here — a small disc tinted to the item's own state color,
     hanging in a fixed gutter so the text truncates or wraps independently. */
  .item::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0.45em;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  .item.no-dot {
    padding-left: 0;
  }

  .item.no-dot::before {
    display: none;
  }

  .item.muted {
    color: var(--muted);
  }

  .item.proc {
    color: var(--accent);
  }

  /* The rendering marker breathes in time with the header's ON AIR lamp —
     one "the desk is live" signal, echoed in the status bar. */
  .item.proc::before {
    animation: footer-live 1.4s ease-in-out infinite;
  }

  @keyframes footer-live {
    50% {
      opacity: 0.35;
    }
  }

  .item.fail {
    color: var(--danger);
    /* the error text is the point of a failure row — never truncate it */
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .item.done {
    color: var(--ok);
  }

  .actions {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .actions .btn {
    padding: 5px 10px;
  }

  .confirm {
    color: var(--danger);
  }

  @media (prefers-reduced-motion: reduce) {
    /* progress still fills, it just steps per chunk instead of easing */
    .rail {
      transition: none;
    }
    /* drop the breathing marker; colour alone still marks "rendering" */
    .item.proc::before {
      animation: none;
    }
  }
</style>
