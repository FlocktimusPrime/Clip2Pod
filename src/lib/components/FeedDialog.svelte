<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import * as api from "$lib/api";
  import * as rip from "$lib/rip_api";
  import { app, toast } from "$lib/stores.svelte";
  import type { FirewallHelp } from "$lib/types";
  import Modal from "./Modal.svelte";

  // The Feed button opens this from whichever tab you're on.
  const isRip = app.tab === "rip";
  const kind = isRip ? "ripped audio" : "narrated articles";

  let url = $state("");
  let qr = $state("");

  $effect(() => {
    (isRip ? rip.feedUrl() : api.feedUrl()).then(
      (u) => (url = u),
      (e) => toast(`Feed URL failed: ${e}`, "error"),
    );
  });

  // qrcode is ~40 kB and only reachable through this dialog — load it on open
  // instead of shipping it in the initial page chunk.
  $effect(() => {
    if (!url) return;
    let stale = false;
    (async () => {
      try {
        const { default: QRCode } = await import("qrcode");
        const dataUrl = await QRCode.toDataURL(url, { margin: 1, width: 220 });
        if (!stale) qr = dataUrl;
      } catch (e) {
        if (!stale) toast(`QR code failed: ${e}`, "error");
      }
    })();
    return () => {
      stale = true;
    };
  });

  let copied = $state(false);

  async function copy() {
    try {
      await writeText(url);
      // Inline confirmation, not a toast: a toast fires behind the open
      // <dialog> (top layer), where this feedback would be missed.
      copied = true;
      setTimeout(() => (copied = false), 1600);
    } catch (e) {
      toast(`Copy failed: ${e}`, "error");
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

<Modal title="Podcast feed — {kind}" onclose={() => (app.dialog = null)}>
  <p class="hint">
    Subscribe in your podcast app for {kind} — same Wi-Fi, and Clip2Pod must be
    running (the tray keeps it alive when the window is closed). The two feeds
    are separate; switch tabs for the other one.
  </p>
  <div class="url-row">
    <code>{url || "…"}</code>
    <button class="btn" onclick={copy} disabled={!url}>{copied ? "Copied" : "Copy"}</button>
  </div>
  {#if qr}
    <div class="qr-wrap">
      <img class="qr" src={qr} alt="QR code for feed URL" />
    </div>
  {/if}
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
