<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import * as api from "$lib/api";
  import * as rip from "$lib/rip_api";
  import { app, toast, type TabName } from "$lib/stores.svelte";
  import type { FirewallHelp } from "$lib/types";
  import Modal from "./Modal.svelte";
  import ModeTabs from "./ModeTabs.svelte";

  // Both feeds in one dialog, one per tab so only one QR is ever on screen.
  // Opens on the mode you came from; the tabs here never change app.tab.
  let mode = $state<TabName>(app.tab);

  const feeds: { id: TabName; title: string; kind: string; load: () => Promise<string> }[] = [
    { id: "narrate", title: "Clip2Pod Narrated", kind: "Narrated articles", load: api.feedUrl },
    { id: "rip", title: "Clip2Pod Ripped", kind: "Ripped audio", load: rip.feedUrl },
  ];

  let urls = $state<Record<TabName, string>>({ narrate: "", rip: "" });
  let qrs = $state<Record<TabName, string>>({ narrate: "", rip: "" });

  function loadUrls() {
    for (const f of feeds) {
      f.load().then(
        (u) => (urls[f.id] = u),
        (e) => toast(`Feed URL failed: ${e}`, "error"),
      );
    }
  }

  $effect(loadUrls);

  // qrcode is ~40 kB and only reachable through this dialog — load it on open
  // instead of shipping it in the initial page chunk.
  $effect(() => {
    const pending = feeds.filter((f) => urls[f.id]).map((f) => [f.id, urls[f.id]] as const);
    if (!pending.length) return;
    let stale = false;
    (async () => {
      try {
        const { default: QRCode } = await import("qrcode");
        for (const [id, u] of pending) {
          const dataUrl = await QRCode.toDataURL(u, { margin: 1, width: 220 });
          if (!stale) qrs[id] = dataUrl;
        }
      } catch (e) {
        if (!stale) toast(`QR code failed: ${e}`, "error");
      }
    })();
    return () => {
      stale = true;
    };
  });

  let copied = $state<TabName | null>(null);

  async function copy(id: TabName) {
    try {
      await writeText(urls[id]);
      // Inline confirmation, not a toast: a toast fires behind the open
      // <dialog> (top layer), where this feedback would be missed.
      copied = id;
      setTimeout(() => (copied = null), 1600);
    } catch (e) {
      toast(`Copy failed: ${e}`, "error");
    }
  }

  // Same inline confirm flow as QueuePanel's "Delete episodes": focus lands on
  // the safe choice (Keep) when it opens, back on the trigger when it closes.
  let confirmReset = $state(false);
  let resetting = $state(false);
  let keepBtn = $state<HTMLButtonElement>();
  let resetBtn = $state<HTMLButtonElement>();
  let confirmWasOpen = false;
  $effect(() => {
    if (confirmReset) {
      confirmWasOpen = true;
      keepBtn?.focus();
    } else if (confirmWasOpen) {
      confirmWasOpen = false;
      resetBtn?.focus();
    }
  });

  async function reset() {
    resetting = true;
    try {
      await api.resetFeedUrl();
      urls = { narrate: "", rip: "" };
      qrs = { narrate: "", rip: "" };
      loadUrls();
    } catch (e) {
      toast(`Reset failed: ${e}`, "error");
    } finally {
      resetting = false;
      confirmReset = false;
    }
  }

  let help = $state<FirewallHelp | null>(null);
  let showHelp = $state(false);
  let cmdCopied = $state(false);

  async function toggleHelp() {
    showHelp = !showHelp;
    if (showHelp && !help) {
      try {
        help = await api.firewallHelp();
      } catch (e) {
        toast(`Firewall help failed: ${e}`, "error");
        showHelp = false;
      }
    }
  }

  async function copyCmd() {
    if (!help) return;
    try {
      await writeText(help.command);
      cmdCopied = true;
      setTimeout(() => (cmdCopied = false), 1600);
    } catch (e) {
      toast(`Copy failed: ${e}`, "error");
    }
  }
</script>

<Modal title="Podcast feeds" onclose={() => (app.dialog = null)}>
  <p class="hint">
    Two separate feeds — subscribe to the ones you want. Same Wi-Fi, and Clip2Pod
    must be running (the tray keeps it alive).
  </p>
  <ModeTabs bind:value={mode} label="Which feed" idPrefix="feed" />
  {#each feeds as f (f.id)}
    <div role="tabpanel" id="feed-panel-{f.id}" aria-labelledby="feed-tab-{f.id}" hidden={mode !== f.id}>
      <p class="hint">
        {f.kind}. Shows up as <strong>{f.title}</strong> in your
        podcast app.
      </p>
      <div class="url-row">
        <code>{urls[f.id] || "…"}</code>
        <button class="btn" onclick={() => copy(f.id)} disabled={!urls[f.id]}>
          {copied === f.id ? "Copied" : "Copy"}
        </button>
      </div>
      {#if qrs[f.id]}
        <div class="qr-wrap">
          <img class="qr" src={qrs[f.id]} alt="QR code for the {f.title} feed" />
        </div>
      {/if}
    </div>
  {/each}
  <div class="reset-row">
    {#if !confirmReset}
      <button class="btn" bind:this={resetBtn} onclick={() => (confirmReset = true)}>
        Reset feed URL
      </button>
    {:else}
      <span class="confirm" role="alert">
        New URL for both feeds — every subscribed device stops getting episodes until you
        re-subscribe it.
      </span>
      <div class="reset-actions">
        <button class="btn" onclick={reset} disabled={resetting}>
          {resetting ? "Resetting…" : "Reset"}
        </button>
        <button class="btn" bind:this={keepBtn} onclick={() => (confirmReset = false)} disabled={resetting}>
          Keep
        </button>
      </div>
    {/if}
  </div>
  <p class="hint firewall-hint">
    Phone can't reach the feed? It may be blocked by your firewall.
    <button class="btn" onclick={toggleHelp} aria-expanded={showHelp}>
      {showHelp ? "Hide firewall help" : "Firewall help"}
    </button>
  </p>
  {#if showHelp && help}
    <div class="fw-help">
      <p class="hint">{help.shell_hint}</p>
      <div class="url-row">
        <code>{help.command}</code>
        <button class="btn" onclick={copyCmd}>{cmdCopied ? "Copied" : "Copy"}</button>
      </div>
      {#if help.tips.length}
        <ul class="fw-tips">
          {#each help.tips as tip}
            <li>{tip}</li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</Modal>

<style>
  .hint {
    font-size: 12.5px;
    color: var(--muted);
    margin: 0 0 12px;
  }

  .url-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  code {
    flex: 1;
    font-family: var(--mono);
    font-size: 13px;
    padding: 8px 10px;
    background: var(--panel-raised);
    border: 1px solid var(--line);
    border-radius: 4px;
    user-select: all;
    overflow-wrap: anywhere;
  }

  .qr-wrap {
    display: flex;
    justify-content: center;
    margin-top: 14px;
  }

  .qr {
    width: 220px;
    height: 220px;
    padding: 8px;
    background: #fff;
    border: 1px solid var(--line);
    border-radius: 4px;
  }

  .reset-row {
    margin-top: 14px;
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .reset-actions {
    display: flex;
    gap: 10px;
  }

  .confirm {
    flex: 1 1 260px;
    font-size: 12.5px;
    color: var(--danger);
  }

  .firewall-hint {
    margin-top: 14px;
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .fw-help {
    margin-top: 10px;
  }

  .fw-help code {
    white-space: pre-wrap;
  }

  .fw-tips {
    margin: 12px 0 0;
    padding-left: 18px;
    font-size: 12.5px;
    color: var(--muted);
  }

  .fw-tips li + li {
    margin-top: 6px;
  }
</style>
