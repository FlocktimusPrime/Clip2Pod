<script lang="ts">
  import { app } from "$lib/stores.svelte";
</script>

<div class="toasts" role="status" aria-live="polite" aria-atomic="false">
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
    animation: toast-in 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateX(12px);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    /* still fade in so the message registers — just don't travel */
    .toast {
      animation-name: toast-fade;
    }
  }

  @keyframes toast-fade {
    from {
      opacity: 0;
    }
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
    background: var(--danger);
  }
</style>
