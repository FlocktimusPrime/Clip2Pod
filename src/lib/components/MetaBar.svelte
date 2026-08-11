<script lang="ts">
  import type { AuthorGender } from "$lib/types";

  // meta is a $state proxy owned by the page; mutating its fields here
  // is reactive without bind:.
  let {
    meta,
    gender,
    ongender,
  }: {
    meta: { title: string; author: string; filenameTitle: string };
    gender: AuthorGender;
    ongender: (g: AuthorGender) => void;
  } = $props();

  const genders: AuthorGender[] = ["Unknown", "Male", "Female"];
</script>

<div class="meta-bar">
  <label class="cell wide">
    <span class="label">Title</span>
    <input class="field" bind:value={meta.title} placeholder="ID3 title tag" />
  </label>
  <label class="cell">
    <span class="label">Author</span>
    <input class="field" bind:value={meta.author} placeholder="ID3 artist tag" />
  </label>
  <label class="cell">
    <span class="label">Filename title</span>
    <input class="field" bind:value={meta.filenameTitle} placeholder="output filename" />
  </label>
  <div class="cell">
    <span class="label" id="gender-label">Author gender</span>
    <div class="seg" role="group" aria-labelledby="gender-label">
      {#each genders as g (g)}
        <button class="seg-cell" class:active={gender === g} onclick={() => ongender(g)}>
          {g === "Unknown" ? "UNK" : g === "Male" ? "M" : "F"}
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  .meta-bar {
    display: grid;
    grid-template-columns: 2fr 1.2fr 1.2fr auto;
    gap: 10px;
    padding: 12px 16px;
    background: var(--panel);
    border-bottom: 1px solid var(--line-soft);
  }

  .cell {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .seg {
    display: flex;
    border: 1px solid var(--line);
    border-radius: 4px;
    overflow: hidden;
    height: 31px;
  }

  .seg-cell {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.1em;
    padding: 0 12px;
    background: var(--panel-raised);
    color: var(--muted);
    border: none;
    cursor: pointer;
  }

  .seg-cell.active {
    background: var(--accent);
    color: var(--accent-ink);
  }

  @media (max-width: 900px) {
    .meta-bar {
      grid-template-columns: 1fr 1fr;
    }
  }
</style>
