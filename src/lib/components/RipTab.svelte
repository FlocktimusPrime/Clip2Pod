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

<svelte:window
  onkeydown={(e) => {
    if (app.tab === "rip" && e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "l") {
      e.preventDefault();
      app.dialog = "rip-log";
    }
  }}
/>

<main>
  {#if missingTools && app.doctor}
    <div class="notice" role="status">
      <p class="notice-head">
        {#if !app.doctor.ytdlp_found && !app.doctor.ffmpeg_found}
          yt-dlp and ffmpeg aren't on your PATH.
        {:else if !app.doctor.ytdlp_found}
          yt-dlp isn't on your PATH.
        {:else}
          ffmpeg isn't on your PATH.
        {/if}
        RIP needs both to work.
      </p>
      <ul>
        {#if !app.doctor.ytdlp_found}
          <li><code>winget install yt-dlp.yt-dlp</code> &nbsp;— or&nbsp; <code>pipx install yt-dlp</code></li>
        {/if}
        {#if !app.doctor.ffmpeg_found}
          <li><code>winget install yt-dlp.FFmpeg</code> &nbsp;(the build yt-dlp expects)</li>
        {/if}
      </ul>
      <p class="notice-foot">
        Restart Clip2Pod after installing.
        <button class="btn small" onclick={refreshDoctor}>Re-check</button>
      </p>
    </div>
  {/if}

  <section class="intake" aria-labelledby="rip-intake-h">
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
    <section class="queue" aria-labelledby="rip-queue-h">
      <div class="section-head">
        <h2 class="label" id="rip-queue-h">Queue</h2>
        {#if activeJobs.some((j) => j.status === "Queued")}
          <button class="btn" onclick={() => rip.clearPending()}>Clear pending</button>
        {/if}
      </div>
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
    </section>
  {/if}

  <section class="episodes" aria-labelledby="rip-episodes-h">
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
                  <button class="btn small" onclick={() => removeEpisode(ep.filename)}>
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
</main>

<style>
  main {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 14px 18px;
    overflow-y: auto;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  section {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 12px 14px;
  }

  .section-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 10px;
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

  .job {
    padding: 8px 0;
    border-top: 1px solid var(--line-soft);
  }

  .job:first-of-type {
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
    border-radius: 3px;
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
    border-radius: 3px;
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
    max-height: 180px;
    overflow-y: auto;
  }

  .btn.small {
    padding: 3px 8px;
    font-size: 10px;
  }

  .episodes {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 120px;
  }

  .table-wrap {
    overflow: auto;
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
    margin: 6px 0;
  }

  .notice {
    border: 1px solid var(--danger);
    border-radius: 8px;
    padding: 12px 14px;
    background: color-mix(in srgb, var(--danger) 10%, transparent);
  }

  .notice-head {
    margin: 0;
    font-size: 13px;
    color: var(--text);
  }

  .notice ul {
    margin: 8px 0;
    padding-left: 18px;
    font-size: 12.5px;
  }

  .notice li {
    margin: 4px 0;
  }

  .notice code {
    font-family: var(--mono);
    font-size: 12px;
    background: var(--panel-raised);
    border: 1px solid var(--line-soft);
    border-radius: 3px;
    padding: 1px 5px;
    user-select: all;
  }

  .notice-foot {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: var(--muted);
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
