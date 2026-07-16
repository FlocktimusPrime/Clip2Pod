<script lang="ts">
  import { app } from "$lib/stores.svelte";
</script>

<div class="toasts">
  {#each app.toasts as t (t.id)}
    <div class="toast" data-kind={t.kind}>{t.text}</div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 70;
  }

  .toast {
    background: var(--panel-raised);
    border: 1px solid var(--line);
    border-radius: 4px;
    padding: 9px 14px 9px 28px;
    font-size: 12.5px;
    box-shadow: 0 8px 24px var(--shadow);
    max-width: 340px;
    overflow-wrap: anywhere;
    position: relative;
  }

  .toast::before {
    content: "";
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
  }

  .toast[data-kind="error"]::before {
    background: var(--onair);
  }
</style>
