<script lang="ts">
  import { onMount } from "svelte";
  import { app, initApp, toast } from "$lib/stores.svelte";
  import type { Lamp } from "$lib/types";
  import Header from "$lib/components/Header.svelte";
  import NarrateTab from "$lib/components/NarrateTab.svelte";
  import RipTab from "$lib/components/RipTab.svelte";
  import VoicesDialog from "$lib/components/VoicesDialog.svelte";
  import JunkDialog from "$lib/components/JunkDialog.svelte";
  import LogDialog from "$lib/components/LogDialog.svelte";
  import FeedDialog from "$lib/components/FeedDialog.svelte";
  import SettingsDialog from "$lib/components/SettingsDialog.svelte";
  import StartupPromptDialog from "$lib/components/StartupPromptDialog.svelte";
  import Toasts from "$lib/components/Toasts.svelte";

  onMount(() => {
    initApp().catch((e) => toast(`Startup failed: ${e}`, "error"));
  });

  // One lamp for two workers: severity from whichever is busier, label names
  // what's actually happening.
  const combined = $derived.by((): { lamp: Lamp; label: string } => {
    const n = app.lamp;
    const r = app.ripLamp;
    const nq = n.state === "Queued" ? n.queued : 0;
    const rq = r.state === "Queued" ? r.queued : 0;
    const queued = nq + rq;

    let lamp: Lamp;
    if (n.state === "OnAir" || r.state === "OnAir") lamp = { state: "OnAir" };
    else if (queued > 0) lamp = { state: "Queued", queued };
    else lamp = { state: "Idle" };

    let label: string;
    if (n.state === "OnAir" && r.state === "OnAir") label = "ON AIR";
    else if (n.state === "OnAir") label = "RENDERING";
    else if (r.state === "OnAir") label = "RIPPING";
    else if (queued > 0) label = `QUEUED ${queued}`;
    else label = "IDLE";

    return { lamp, label };
  });
</script>

<div class="shell">
  <Header
    tab={app.tab}
    ontab={(t) => (app.tab = t)}
    lamp={combined.lamp}
    label={combined.label}
    onfeed={() => (app.dialog = "feed")}
    onlog={() => (app.dialog = app.tab === "rip" ? "rip-log" : "log")}
    onsettings={() => (app.dialog = "settings")}
  />

  <div class="content">
    <div class="pane" hidden={app.tab !== "narrate"}><NarrateTab /></div>
    <div class="pane" hidden={app.tab !== "rip"}><RipTab /></div>
  </div>
</div>

{#if app.dialog === "voices"}
  <VoicesDialog />
{:else if app.dialog === "junk"}
  <JunkDialog />
{:else if app.dialog === "log" || app.dialog === "rip-log"}
  <LogDialog />
{:else if app.dialog === "feed"}
  <FeedDialog />
{:else if app.dialog === "settings"}
  <SettingsDialog />
{:else if app.dialog === "startup-prompt"}
  <StartupPromptDialog />
{/if}

<Toasts />

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .content {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .pane {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
</style>
