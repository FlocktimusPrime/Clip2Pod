<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import QRCode from "qrcode";
  import * as api from "$lib/api";
  import { app, toast } from "$lib/stores.svelte";
  import Modal from "./Modal.svelte";

  let url = $state("");
  let qr = $state("");

  $effect(() => {
    api.feedUrl().then(
      (u) => (url = u),
      (e) => toast(`Feed URL failed: ${e}`, "error"),
    );
  });

  $effect(() => {
    if (!url) return;
    QRCode.toDataURL(url, { margin: 1, width: 220 }).then(
      (d) => (qr = d),
      (e) => toast(`QR code failed: ${e}`, "error"),
    );
  });

  async function copy() {
    try {
      await writeText(url);
      toast("Feed URL copied");
    } catch (e) {
      toast(`Copy failed: ${e}`, "error");
    }
  }
</script>

<Modal title="Podcast feed" onclose={() => (app.dialog = null)}>
  <p class="hint">
    Subscribe in your podcast app — same Wi-Fi, and Clip2Pod must be running
    (the tray keeps it alive when the window is closed).
  </p>
  <div class="url-row">
    <code>{url || "…"}</code>
    <button class="btn" onclick={copy} disabled={!url}>Copy</button>
  </div>
  {#if qr}
    <div class="qr-wrap">
      <img class="qr" src={qr} alt="QR code for feed URL" />
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
</style>
