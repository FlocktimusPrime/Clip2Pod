<script lang="ts">
  import * as api from "$lib/api";
  import { app, toast } from "$lib/stores.svelte";
  import type { AuthorEntry, AuthorGender } from "$lib/types";
  import Modal from "./Modal.svelte";

  let authors = $state<AuthorEntry[]>([]);
  let search = $state("");
  let selected = $state<Set<string>>(new Set());

  $effect(() => {
    api.listAuthors().then((list) => (authors = list));
  });

  const filtered = $derived(
    authors.filter((a) => a.name.toLowerCase().includes(search.trim().toLowerCase())),
  );

  const genders: AuthorGender[] = ["Unknown", "Male", "Female"];

  async function refresh() {
    authors = await api.listAuthors();
  }

  async function setGender(name: string, gender: AuthorGender) {
    await api.upsertAuthorGender(name, gender);
    await refresh();
  }

  async function rename(oldName: string, e: Event) {
    const newName = (e.currentTarget as HTMLInputElement).value.trim();
    if (!newName || newName === oldName) {
      (e.currentTarget as HTMLInputElement).value = oldName;
      return;
    }
    await api.renameAuthor(oldName, newName);
    await refresh();
    toast(`Renamed to "${newName}"`);
  }

  async function remove(name: string) {
    await api.deleteAuthor(name);
    selected.delete(name);
    await refresh();
  }

  function toggleSelect(name: string) {
    if (selected.has(name)) selected.delete(name);
    else selected.add(name);
    selected = new Set(selected);
  }

  const mergePair = $derived([...selected]);

  async function merge(primary: string, other: string) {
    await api.mergeAuthors(primary, other);
    selected = new Set();
    await refresh();
    toast(`Merged into "${primary}"`);
  }
</script>

<Modal title="Manage Author Genders" onclose={() => (app.dialog = null)}>
  <p class="hint">
    Remembered from picking a gender in the editor. Pick a new gender here to
    correct one; select two rows to merge byline variants of the same person.
  </p>

  <div class="toolbar">
    <input
      class="field"
      aria-label="Search authors"
      placeholder="Search author name…"
      bind:value={search}
    />
  </div>

  {#if mergePair.length === 2}
    <div class="merge-bar" role="group" aria-label="Merge selected authors">
      <span>Merge into:</span>
      <button class="btn" onclick={() => merge(mergePair[0], mergePair[1])}>
        “{mergePair[0]}”
      </button>
      <button class="btn" onclick={() => merge(mergePair[1], mergePair[0])}>
        “{mergePair[1]}”
      </button>
    </div>
  {/if}

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th aria-label="Select for merge"></th>
          <th>Author</th>
          <th>Gender</th>
          <th aria-label="Delete"></th>
        </tr>
      </thead>
      <tbody>
        {#each filtered as a (a.name)}
          <tr>
            <td>
              <input
                type="checkbox"
                checked={selected.has(a.name)}
                onchange={() => toggleSelect(a.name)}
                aria-label={`Select ${a.name} to merge`}
              />
            </td>
            <td>
              <input
                class="field name"
                value={a.name}
                onchange={(e) => rename(a.name, e)}
                aria-label={`Rename ${a.name}`}
              />
            </td>
            <td>
              <div class="seg" role="group" aria-label={`Gender for ${a.name}`}>
                {#each genders as g (g)}
                  <button
                    class="seg-cell"
                    class:active={a.gender === g}
                    aria-pressed={a.gender === g}
                    onclick={() => setGender(a.name, g)}
                  >
                    {g === "Unknown" ? "UNK" : g === "Male" ? "M" : "F"}
                  </button>
                {/each}
              </div>
            </td>
            <td>
              <button class="btn" onclick={() => remove(a.name)} aria-label={`Delete ${a.name}`}>
                Delete
              </button>
            </td>
          </tr>
        {:else}
          <tr><td colspan="4" class="empty">No recognized authors yet.</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</Modal>

<style>
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 0 0 10px;
  }

  .toolbar {
    margin-bottom: 10px;
  }

  .toolbar .field {
    width: 100%;
  }

  .merge-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    font-size: 12.5px;
  }

  .table-wrap {
    overflow: auto;
    max-height: 52vh;
    border: 1px solid var(--line-soft);
    border-radius: 4px;
  }

  .name {
    width: 100%;
    border: none;
    background: none;
  }

  .seg {
    display: flex;
    border: 1px solid var(--line);
    border-radius: 4px;
    overflow: hidden;
    width: fit-content;
  }

  .seg-cell {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.1em;
    padding: 4px 10px;
    background: var(--panel-raised);
    color: var(--muted);
    border: none;
    cursor: pointer;
  }

  .seg-cell.active {
    background: var(--accent);
    color: var(--accent-ink);
  }

  input[type="checkbox"] {
    accent-color: var(--accent);
  }

  .empty {
    color: var(--muted);
    text-align: center;
    padding: 18px;
  }
</style>
