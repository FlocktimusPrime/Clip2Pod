<script lang="ts">
  import * as api from "$lib/api";
  import { ariaSort, sortIndicator, sortRows, toggleSort, type SortState } from "$lib/sort";
  import { app, toast } from "$lib/stores.svelte";
  import type { VoiceInfo } from "$lib/types";
  import Modal from "./Modal.svelte";

  type Col = "voice" | "gender" | "locale" | "category";

  let search = $state("");
  let selected = $state<string | null>(null);
  let previewing = $state(false);
  let sort = $state<SortState<Col>>(null);

  const pick = (v: VoiceInfo, key: Col) => (key === "voice" ? v.short_name : v[key]);

  const enabledSet = $derived(new Set(app.voices?.enabled ?? []));

  const filtered = $derived(
    (app.voices?.voices ?? []).filter((v) => {
      const q = search.trim().toLowerCase();
      if (!q) return true;
      return [v.short_name, v.gender, v.locale, v.country, v.category]
        .join(" ")
        .toLowerCase()
        .includes(q);
    }),
  );

  const sorted = $derived(sortRows(filtered, sort, pick));

  async function toggle(shortName: string, on: boolean) {
    // voices-changed event refreshes app.voices with authoritative counts
    await api.setVoiceEnabled(shortName, on);
  }

  async function preview() {
    if (!selected || previewing) return;
    previewing = true;
    try {
      const bytes = await api.previewVoice(selected);
      const url = URL.createObjectURL(new Blob([bytes], { type: "audio/mpeg" }));
      const audio = new Audio(url);
      audio.onended = () => URL.revokeObjectURL(url);
      await audio.play();
    } catch (e) {
      toast(`Preview failed: ${e}`, "error");
    } finally {
      previewing = false;
    }
  }
</script>

<Modal title="Manage voices" onclose={() => (app.dialog = null)}>
  <div class="toolbar">
    <input
      class="field"
      placeholder="Search name, locale, category…"
      bind:value={search}
    />
    <button class="btn" onclick={() => api.enableAllVoices()}>Enable all</button>
    <button class="btn" onclick={() => api.disableAllVoices()}>Disable all</button>
    <button class="btn" onclick={preview} disabled={!selected || previewing}>
      {previewing ? "Playing…" : "Preview voice"}
    </button>
  </div>

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th></th>
          {#each [["voice", "Voice"], ["gender", "Gender"], ["locale", "Locale"], ["category", "Category"]] as const as [key, label] (key)}
            <th aria-sort={ariaSort(sort, key)}>
              <button class="th-sort" onclick={() => (sort = toggleSort(sort, key))}>
                {label} <span class="arrow">{sortIndicator(sort, key)}</span>
              </button>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each sorted as v (v.short_name)}
          <tr
            class:selected={selected === v.short_name}
            onclick={() => (selected = v.short_name)}
          >
            <td>
              <input
                type="checkbox"
                checked={enabledSet.has(v.short_name)}
                onclick={(e) => e.stopPropagation()}
                onchange={(e) => toggle(v.short_name, (e.currentTarget as HTMLInputElement).checked)}
                aria-label={`Enable ${v.short_name}`}
              />
            </td>
            <td>{v.short_name}</td>
            <td>{v.gender}</td>
            <td>{v.locale}</td>
            <td>{v.category}</td>
          </tr>
        {:else}
          <tr><td colspan="5" class="empty">No voices match this search.</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</Modal>

<style>
  .toolbar {
    display: flex;
    gap: 8px;
    margin-bottom: 10px;
  }

  .toolbar .field {
    flex: 1;
  }

  .table-wrap {
    overflow: auto;
    max-height: 52vh;
    border: 1px solid var(--line-soft);
    border-radius: 4px;
  }

  tbody tr {
    cursor: pointer;
  }

  .th-sort {
    font: inherit;
    letter-spacing: inherit;
    text-transform: inherit;
    color: inherit;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  .arrow {
    color: var(--accent);
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
