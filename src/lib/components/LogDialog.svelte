<script lang="ts">
  import * as api from "$lib/api";
  import * as rip from "$lib/rip_api";
  import { app, toast, type TabName } from "$lib/stores.svelte";
  import Modal from "./Modal.svelte";
  import ModeTabs from "./ModeTabs.svelte";
  import NarrateLog from "./NarrateLog.svelte";
  import RipLog from "./RipLog.svelte";

  // Opens on the mode you came from; the tabs here never change app.tab.
  let mode = $state<TabName>(app.tab);

  const empty = $derived(mode === "rip" ? app.ripLog.length === 0 : app.log.length === 0);

  async function clear() {
    try {
      if (mode === "rip") {
        await rip.clearLog();
        app.ripLog = [];
      } else {
        await api.clearLog();
        app.log = [];
      }
    } catch (e) {
      toast(`Clear failed: ${e}`, "error");
    }
  }
</script>

<Modal title="Log" onclose={() => (app.dialog = null)}>
  <ModeTabs bind:value={mode} label="Which log" idPrefix="log" />

  <!-- Both panels stay mounted so each keeps its search and sort. -->
  <div role="tabpanel" id="log-panel-narrate" aria-labelledby="log-tab-narrate" hidden={mode !== "narrate"}>
    <NarrateLog />
  </div>
  <div role="tabpanel" id="log-panel-rip" aria-labelledby="log-tab-rip" hidden={mode !== "rip"}>
    <RipLog />
  </div>

  {#snippet footer()}
    <button class="btn" onclick={clear} disabled={empty}>
      Clear {mode === "rip" ? "Rip" : "Narrate"} log
    </button>
  {/snippet}
</Modal>
