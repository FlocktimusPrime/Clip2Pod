<script lang="ts">
  import * as api from "$lib/api";
  import { app } from "$lib/stores.svelte";
  import Modal from "./Modal.svelte";

  async function answer(launch: boolean) {
    // Native <dialog> routes Esc through onclose → answer(false), so this also
    // runs on dismiss. Always close, even if the backend call fails, so the
    // prompt doesn't wedge open.
    try {
      if (app.config) app.config.launch_at_startup = launch;
      await api.setLaunchAtStartup(launch);
    } finally {
      app.dialog = null;
    }
  }
</script>

<Modal title="Launch at startup?" onclose={() => answer(false)}>
  <p>Open Clip2Pod automatically when you sign in to Windows?</p>
  <p class="hint">You can change this anytime in Settings.</p>

  {#snippet footer()}
    <button class="btn" onclick={() => answer(false)}>No</button>
    <button class="btn primary" onclick={() => answer(true)}>Yes</button>
  {/snippet}
</Modal>

<style>
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 0;
  }
</style>
