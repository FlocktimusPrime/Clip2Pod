<script lang="ts">
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import { ask } from "@tauri-apps/plugin-dialog";
  import * as rip from "$lib/rip_api";
  import { app, refreshDoctor, refreshRipEpisodes, toast } from "$lib/stores.svelte";
  import type { RipJob } from "$lib/types";

  let url = $state("");
  let expanded = $state<string | null>(null);

  async function download() {
    const candidate = url.trim();
    if (!candidate) return;
    try {
      await rip.enqueue(candidate);
      url = "";
    } catch (e) {
      toast(`${e}`, "error");
    }
  }

  async function pasteFromClipboard() {
    try {
      const text = (await readText())?.trim() ?? "";
      if (!text.startsWith("http")) {
        toast("Clipboard has no link", "error");
        return;
      }
      url = text;
      await download();
    } catch (e) {
      toast(`Clipboard read failed: ${e}`, "error");
    }
  }

  async function removeEpisode(filename: string) {
    try {
      await rip.deleteEpisode(filename);
      await refreshRipEpisodes();
      toast(`Deleted ${filename}`);
    } catch (e) {
      toast(`Delete failed: ${e}`, "error");
    }
  }

  async function removeAll() {
    const n = app.ripEpisodes.length;
    if (n === 0) return;
    const yes = await ask(`Delete all ${n} episode file${n === 1 ? "" : "s"} from the folder?`, {
      title: "Delete all episodes",
      kind: "warning",
    });
    if (!yes) return;
    try {
      const deleted = await rip.deleteAllEpisodes();
      toast(`Deleted ${deleted} file${deleted === 1 ? "" : "s"}`);
    } catch (e) {
      toast(`${e}`, "error");
    }
    await refreshRipEpisodes();
  }

  function jobLabel(job: RipJob): string {
    return job.filename ?? job.url;
  }

  function fmtSize(bytes: number): string {
    if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  }

  function fmtDuration(ms: number | null): string {
    if (ms === null) return "—";
    const s = Math.round(ms / 1000);
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    return h > 0
      ? `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`
      : `${m}:${String(sec).padStart(2, "0")}`;
  }

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  const missingTools = $derived(
    app.doctor !== null && (!app.doctor.ytdlp_found || !app.doctor.ffmpeg_found),
  );

  const activeJobs = $derived(
    app.ripQueue.filter((j) => j.status === "Queued" || j.status === "Processing"),
  );
  const settledJobs = $derived(
    [...app.ripQueue.filter((j) => j.status !== "Queued" && j.status !== "Processing")].reverse(),
  );
</script>

<div class="desk">
  {#if missingTools && app.doctor}
    <div class="doctor" role="status">
      <span class="dot" aria-hidden="true"></span>
      <span class="doctor-text">
        {#if !app.doctor.ytdlp_found && !app.doctor.ffmpeg_found}
          Install yt-dlp and ffmpeg to rip audio.
        {:else if !app.doctor.ytdlp_found}
          Install yt-dlp to rip audio.
        {:else}
          Install ffmpeg to rip audio.
        {/if}
        Not on your PATH yet — narration still works without it.
      </span>
      {#if !app.doctor.ytdlp_found}
        <code>winget install yt-dlp.yt-dlp</code>
      {/if}
      {#if !app.doctor.ffmpeg_found}
        <code>winget install yt-dlp.FFmpeg</code>
      {/if}
      <button class="btn small" onclick={refreshDoctor}>Re-check</button>
    </div>
  {/if}

  <section class="band intake" aria-labelledby="rip-intake-h">
    <h2 class="label" id="rip-intake-h">Add episode</h2>
    <div class="row">
      <input
        class="field"
        aria-label="Video URL"
        placeholder="https://www.youtube.com/watch?v=…"
        bind:value={url}
        spellcheck="false"
        onkeydown={(e) => {
          if (e.key === "Enter") download();
        }}
      />
      <button class="btn" onclick={pasteFromClipboard}>Paste link</button>
      <button class="btn primary" onclick={download} disabled={!url.trim()}>Download</button>
    </div>
  </section>

  {#if activeJobs.length > 0 || settledJobs.length > 0}
    <section class="band queue" aria-labelledby="rip-queue-h">
      <div class="section-head">
        <h2 class="label" id="rip-queue-h">Queue</h2>
        {#if activeJobs.some((j) => j.status === "Queued")}
          <button class="btn" onclick={() => rip.clearPending()}>Clear pending</button>
        {/if}
      </div>
      <div class="job-list">
        {#each [...activeJobs, ...settledJobs] as job (job.id)}
          <div class="job">
            <div class="job-line">
              <span class="badge {job.status.toLowerCase()}">{job.status}</span>
              <span class="job-title" title={job.url}>{jobLabel(job)}</span>
              {#if job.status === "Processing" && job.progress.stage}
                <span class="stage">
                  {job.progress.stage}{job.progress.speed ? ` · ${job.progress.speed}` : ""}
                </span>
              {/if}
              {#if job.status === "Processing"}
                <button class="btn small" onclick={() => rip.stopJob(job.id)}>Stop</button>
              {/if}
              {#if job.status === "Failed"}
                <button
                  class="btn small"
                  aria-expanded={expanded === job.id}
                  onclick={() => (expanded = expanded === job.id ? null : job.id)}
                >
                  {expanded === job.id ? "Hide" : "Why?"}
                </button>
              {/if}
              {#if job.status === "Done" && job.detail}
                <span class="stage">{job.detail}</span>
              {/if}
            </div>
            {#if job.status === "Processing"}
              <div
                class="bar"
                role="progressbar"
                aria-label={jobLabel(job)}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={job.progress.percent ?? undefined}
              >
                <div
                  class="fill"
                  class:indeterminate={job.progress.percent === null}
                  style="transform: scaleX({(job.progress.percent ?? 100) / 100})"
                ></div>
              </div>
            {/if}
            {#if expanded === job.id && job.detail}
              <pre class="detail">{job.detail}</pre>
              <p class="fail-hint">
                An outdated yt-dlp is a common cause — run <code>yt-dlp -U</code>
                (or reinstall your build) and try again.
              </p>
            {/if}
          </div>
        {/each}
      </div>
    </section>
  {/if}

  <section class="band episodes" aria-labelledby="rip-episodes-h">
    <div class="section-head">
      <h2 class="label" id="rip-episodes-h">Episodes — {app.ripEpisodes.length}</h2>
      <div class="actions">
        <button class="btn" onclick={refreshRipEpisodes}>Refresh</button>
        <button class="btn" onclick={removeAll} disabled={app.ripEpisodes.length === 0}>
          Delete all
        </button>
      </div>
    </div>
    {#if app.ripEpisodes.length === 0}
      <p class="empty">No episodes yet — paste a link above to rip one.</p>
    {:else}
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Title</th>
              <th>Channel</th>
              <th>Length</th>
              <th>Size</th>
              <th>Added</th>
              <th><span class="vh">Actions</span></th>
            </tr>
          </thead>
          <tbody>
            {#each app.ripEpisodes as ep (ep.filename)}
              <tr>
                <td class="ep-title" title={ep.filename}>{ep.title}</td>
                <td>{ep.artist ?? "—"}</td>
                <td>{fmtDuration(ep.duration_ms)}</td>
                <td>{fmtSize(ep.size)}</td>
                <td>{fmtDate(ep.modified)}</td>
                <td class="right">
                  <button
                    class="btn small"
                    aria-label="Delete {ep.title}"
                    onclick={() => removeEpisode(ep.filename)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>
</div>

<style>
  /* RIP is a fixed desk, same as NARRATE: fills the content area, never scrolls
     as a whole — the queue and the episode table own their own scroll. */
  .desk {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .band {
    padding: 12px 16px;
    background: var(--panel);
    border-bottom: 1px solid var(--line);
  }

  /* Doctor strip: one row above the intake band, panel-raised so it reads as a
     transient notice rather than permanent chrome. */
  .doctor {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px 12px;
    padding: 9px 16px;
    background: var(--panel-raised);
    border-bottom: 1px solid var(--line);
    font-size: 12px;
  }

  .doctor .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--danger);
    flex-shrink: 0;
  }

  .doctor-text {
    color: var(--text);
  }

  .doctor code {
    font-family: var(--mono);
    font-size: 11.5px;
    background: var(--panel);
    border: 1px solid var(--line-soft);
    border-radius: 4px;
    padding: 1px 6px;
    user-select: all;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .section-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .intake .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 8px;
  }

  .intake .field {
    flex: 1;
    min-width: 16rem;
    font-family: var(--mono);
    font-size: 13px;
  }

  /* Bounded scroll region — jobs never push the episode table off the desk. */
  .queue {
    flex: 0 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .job-list {
    margin-top: 8px;
    overflow-y: auto;
    max-height: 168px;
  }

  .job {
    padding: 8px 0;
    border-top: 1px solid var(--line-soft);
  }

  .job:first-child {
    border-top: none;
  }

  .job-line {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .job-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }

  .stage {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }

  .badge {
    font-family: var(--mono);
    font-size: 9.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    padding: 2px 7px;
    border-radius: 4px;
    border: 1px solid var(--line);
    color: var(--muted);
  }

  .badge.processing {
    color: var(--accent-ink);
    background: var(--accent);
    border-color: var(--accent);
  }

  .badge.done {
    color: var(--ok);
    border-color: var(--ok);
  }

  .badge.failed {
    color: var(--danger);
    border-color: var(--danger);
  }

  .bar {
    margin-top: 7px;
    height: 6px;
    border-radius: 4px;
    background: var(--panel-raised);
    border: 1px solid var(--line-soft);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    width: 100%;
    background: var(--accent);
    transform-origin: left;
    transition: transform 300ms linear;
  }

  /* No percent yet: a barber-pole track, so it never reads as "complete". */
  .fill.indeterminate {
    background: repeating-linear-gradient(
      -45deg,
      var(--accent) 0 8px,
      var(--accent-dim) 8px 16px
    );
    animation: indeterminate-stripes 0.7s linear infinite;
  }

  @keyframes indeterminate-stripes {
    to {
      background-position: -22.63px 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .fill.indeterminate {
      animation: none;
    }
  }

  .detail {
    margin: 8px 0 0;
    padding: 8px 10px;
    font-family: var(--mono);
    font-size: 11px;
    line-height: 1.5;
    color: var(--muted);
    background: var(--bg);
    border: 1px solid var(--line-soft);
    border-radius: 4px;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 120px;
    overflow-y: auto;
  }

  .btn.small {
    padding: 3px 8px;
    font-size: 10px;
  }

  /* Fills the remaining desk height; the table body scrolls, its header sticks. */
  .episodes {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    border-bottom: none;
  }

  .table-wrap {
    flex: 1;
    overflow: auto;
    margin-top: 8px;
  }

  .ep-title {
    max-width: 0;
    width: 45%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  td.right {
    text-align: right;
  }

  .empty {
    color: var(--muted);
    font-size: 13px;
    margin: 8px 0 0;
  }

  .fail-hint {
    margin: 6px 0 0;
    font-size: 11.5px;
    color: var(--muted);
  }

  .fail-hint code {
    font-family: var(--mono);
  }
</style>
