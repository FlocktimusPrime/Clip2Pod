<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import type { EditorView } from "@codemirror/view";
  import { onMount } from "svelte";
  import * as api from "$lib/api";
  import {
    clearJunkMatch,
    createEditor,
    deleteCurrentLine,
    getSelection,
    getText,
    setText,
    showJunkMatch,
  } from "$lib/editor";
  import { app, toast } from "$lib/stores.svelte";
  import type { AuthorGender, CapturedArticle, Extracted } from "$lib/types";
  import MetaBar from "./MetaBar.svelte";
  import Sidebar from "./Sidebar.svelte";
  import QueuePanel from "./QueuePanel.svelte";

  let editorHost: HTMLDivElement;
  let view: EditorView;

  const meta = $state({ title: "", author: "", filenameTitle: "" });

  /** Recognized-authors gender for the current `meta.author`, or null if
   *  unrecognized. Drives MetaBar's indicator and the pre-fill on load/edit. */
  let savedGenderForAuthor = $state<AuthorGender | null>(null);
  let authorLookupTimer: ReturnType<typeof setTimeout> | undefined;

  /** Origin URL of the current script when it came from fetch/capture. */
  let sourceUrl = $state<string | null>(null);

  /** Line of the currently flagged junk match, if a review is active. */
  let activeMatchLine: number | null = null;
  /** Where the current review pass began; decides Wrap vs No-junk at the end. */
  let reviewStart = 0;

  type Coach = { text: string; actions: { label: string; run: () => void }[] };
  let coach = $state<Coach | null>(null);

  onMount(() => {
    view = createEditor(editorHost);
    // Browser extension posts a page to the capture listener.
    const unCapture = listen<CapturedArticle>("article-captured", (e) =>
      applyArticle(e.payload, e.payload.url),
    );
    // Global hotkey / tray: window is shown by Rust, then we paste.
    const unIntake = listen("intake-clipboard", () => pasteClipboard());
    return () => {
      view.destroy();
      unCapture.then((f) => f());
      unIntake.then((f) => f());
    };
  });

  // The tab lives in a display:none pane when RIP is showing; CodeMirror needs a
  // remeasure when it comes back or the gutter/scroll geometry is stale.
  $effect(() => {
    if (app.tab === "narrate" && view) view.requestMeasure();
  });

  // Recognize the author on load and live as the name field is edited
  // (debounced so every keystroke doesn't round-trip). A match pre-fills the
  // gender picker; no match leaves whatever gender is already selected alone.
  $effect(() => {
    const name = meta.author;
    clearTimeout(authorLookupTimer);
    if (!name.trim()) {
      savedGenderForAuthor = null;
      return;
    }
    authorLookupTimer = setTimeout(async () => {
      const found = await api.lookupAuthorGender(name);
      savedGenderForAuthor = found;
      if (found) await setGender(found);
    }, 300);
  });

  function resetFind() {
    activeMatchLine = null;
    coach = null;
    if (view) clearJunkMatch(view);
  }

  function resetMeta() {
    meta.title = "";
    meta.author = "";
    meta.filenameTitle = "";
  }

  async function pasteClipboard() {
    let text: string | null = null;
    try {
      text = await readText();
    } catch {
      // clipboard without text content throws on Linux; treat as empty
    }
    if (!text || !text.trim()) {
      toast("Clipboard has no text — editor left untouched", "error");
      return;
    }
    setText(view, text);
    sourceUrl = null;
    resetMeta();
    resetFind();
  }

  const cursorLine = () =>
    view.state.doc.lineAt(view.state.selection.main.head).number - 1;

  async function searchJunk(from: number) {
    const match = await api.findJunk(getText(view), from);
    if (match) {
      showJunkMatch(view, match.line_idx, match.phrase);
      activeMatchLine = match.line_idx;
      coach = {
        text: `Junk: “${match.phrase}”`,
        actions: [
          { label: "Delete line", run: deleteFlagged },
          { label: "Next", run: () => searchJunk(match.line_idx + 1) },
          { label: "Stop", run: resetFind },
        ],
      };
      return;
    }
    activeMatchLine = null;
    clearJunkMatch(view);
    // Coach buttons get torn down between states; park focus in the editor
    // so keyboard users can Tab straight to the new coach actions.
    view.focus();
    const clean = {
      label: "Clean for TTS",
      run: () => {
        coach = null;
        cleanForTts();
      },
    };
    const dismiss = { label: "Dismiss", run: () => (coach = null) };
    coach =
      reviewStart > 0
        ? {
            text: "End of script.",
            actions: [
              {
                label: "Wrap to top",
                run: () => {
                  reviewStart = 0;
                  searchJunk(0);
                },
              },
              clean,
              dismiss,
            ],
          }
        : { text: "No junk found.", actions: [clean, dismiss] };
  }

  function findJunkNext() {
    coach = null;
    // Continue past an active match, otherwise start wherever the cursor is —
    // manual edits move the cursor, so the search never goes stale.
    const from = activeMatchLine !== null ? activeMatchLine + 1 : cursorLine();
    reviewStart = from;
    searchJunk(from);
  }

  /** Add the current editor selection to the persisted junk phrase list. */
  async function addSelectionAsJunk() {
    const raw = getSelection(view).trim();
    if (!raw) return toast("Select text in the editor first", "error");
    if (raw.includes("\n")) return toast("Junk phrases match one line — select less", "error");
    if (raw.length > 80) return toast("That selection is too long for a junk phrase", "error");
    const phrase = raw.toLowerCase();
    const phrases = await api.getJunkPhrases();
    if (phrases.includes(phrase)) return toast(`Already a junk phrase: “${phrase}”`);
    await api.setJunkPhrases([...phrases, phrase]);
    toast(`Junk phrase added: “${phrase}”`);
  }

  /** First line whose trimmed, lowercased text is exactly `phrase`. */
  function findLineIndex(phrase: string): number {
    return getText(view)
      .split("\n")
      .findIndex((l) => l.trim().toLowerCase() === phrase);
  }

  /** Walk scanned junk suggestions through the coach bar, one at a time. */
  async function suggestJunk() {
    resetFind();
    const list = await api.suggestJunk(getText(view));
    if (!list.length) {
      coach = { text: "No junk suggestions found.", actions: suggestEndActions() };
      return;
    }
    walkSuggestion(list, 0);
  }

  /** Shown when a Suggest Junk pass ends: hand off to Find Junk, or dismiss. */
  function suggestEndActions() {
    return [
      { label: "Find junk", run: findJunkNext },
      { label: "Dismiss", run: () => (coach = null) },
    ];
  }

  function walkSuggestion(list: string[], i: number) {
    if (i >= list.length) {
      clearJunkMatch(view);
      coach = { text: "No more suggestions.", actions: suggestEndActions() };
      return;
    }
    const phrase = list[i];
    const lineIdx = findLineIndex(phrase);
    if (lineIdx >= 0) showJunkMatch(view, lineIdx, phrase);
    const next = () => walkSuggestion(list, i + 1);
    coach = {
      text: `Possible junk: “${phrase}”`,
      actions: [
        {
          label: "Add & delete line",
          run: async () => {
            const phrases = await api.getJunkPhrases();
            if (!phrases.includes(phrase)) await api.setJunkPhrases([...phrases, phrase]);
            const li = findLineIndex(phrase);
            if (li >= 0) {
              showJunkMatch(view, li, phrase);
              deleteCurrentLine(view);
            }
            next();
          },
        },
        { label: "Skip", run: next },
        { label: "Stop", run: resetFind },
      ],
    };
  }

  /** Delete the flagged line, then resume the review at the same index. */
  function deleteFlagged() {
    if (activeMatchLine === null) return;
    const line = activeMatchLine;
    deleteCurrentLine(view);
    activeMatchLine = null;
    searchJunk(line);
  }

  async function cleanForTts() {
    const { cleaned, autofill } = await api.cleanText(getText(view));
    setText(view, cleaned);
    if (sourceUrl === null) {
      // Pasted text: "Title / By Author / body…" so the positional autofill applies.
      meta.title = autofill.title;
      meta.author = autofill.author;
      meta.filenameTitle = autofill.filename_title;
    } else {
      // Extracted content: readability already separated the real title/author,
      // so line 1 of the body is prose. Keep the existing fields; only fill
      // gaps in the titles, and never guess the author from body lines.
      if (!meta.title) meta.title = autofill.title;
      if (!meta.filenameTitle) meta.filenameTitle = autofill.filename_title;
    }
    resetFind();
  }

  async function generate() {
    try {
      await api.enqueueGenerate(getText(view), meta.title, meta.author, meta.filenameTitle, sourceUrl);
      toast("Queued for narration");
    } catch (e) {
      toast(String(e), "error");
    }
  }

  function deleteLine() {
    deleteCurrentLine(view);
    // a manual delete invalidates any flagged match; next find starts at cursor
    activeMatchLine = null;
  }

  function applyArticle({ title, author, text }: Extracted, url: string | null) {
    setText(view, text);
    sourceUrl = url;
    resetMeta();
    if (title) {
      meta.title = title;
      meta.filenameTitle = title;
    }
    if (author) meta.author = author;
    resetFind();
    toast("Article extracted — review, then clean");
  }

  async function fetchArticle(url: string) {
    try {
      applyArticle(await api.extractUrl(url), url);
    } catch (e) {
      toast(String(e), "error");
    }
  }

  /** Set the active gender without touching the recognized-authors table —
   *  used for the automatic pre-fill from a lookup match. */
  async function setGender(g: AuthorGender) {
    if (app.config) app.config.author_gender = g;
    await api.setAuthorGender(g);
  }

  /** The user manually picked a gender for this article: apply it, and
   *  remember it for this author going forward (see AuthorGender rules in
   *  clip2pod-core/src/authors.rs). */
  async function changeGender(g: AuthorGender) {
    await setGender(g);
    const name = meta.author.trim();
    if (name) savedGenderForAuthor = await api.upsertAuthorGender(name, g);
  }

  function onKeydown(e: KeyboardEvent) {
    if (app.tab !== "narrate") return;
    if (e.key === "Escape") {
      // An open dialog is a native <dialog>; it closes itself on Esc and its
      // onclose clears app.dialog. Only the coach bar needs handling here.
      if (!app.dialog && coach) coach = null;
      return;
    }
    if (!e.ctrlKey || e.altKey) return;

    const shift = e.shiftKey;
    const key = e.key.toLowerCase();
    const run = (fn: () => void) => {
      e.preventDefault();
      e.stopPropagation();
      fn();
    };

    if (key === "v" && shift) return run(pasteClipboard);
    if (key === "l" && shift) return run(() => (app.dialog = "log"));
    if (key === "k" && shift) return run(suggestJunk);
    if (shift) return;

    if (key === "f") return run(findJunkNext);
    if (key === "k") return run(addSelectionAsJunk);
    if (key === "l") return run(cleanForTts);
    if (key === "d") return run(deleteLine);
    if (key === "j") return run(() => (app.dialog = "junk"));
    if (key === "m") return run(() => (app.dialog = "voices"));
    if (key === "g") return run(() => (app.dialog = "authors"));
    // Ctrl+, is the conventional Settings/Preferences shortcut.
    if (key === ",") return run(() => (app.dialog = "settings"));
    if (key === "enter") return run(generate);
  }
</script>

<svelte:window onkeydowncapture={onKeydown} />

<div class="desk">
  <MetaBar
    {meta}
    gender={app.config?.author_gender ?? "Unknown"}
    recognized={savedGenderForAuthor !== null &&
      savedGenderForAuthor === (app.config?.author_gender ?? "Unknown")}
    ongender={changeGender}
  />

  <div class="deck">
    <main class="script" aria-label="Script editor">
      <div class="editor" bind:this={editorHost}></div>

      {#if coach}
        <div class="coach" role="status">
          <span>{coach.text}</span>
          {#each coach.actions as action (action.label)}
            <button class="btn" onclick={action.run}>{action.label}</button>
          {/each}
        </div>
      {/if}
    </main>

    <Sidebar
      onpaste={pasteClipboard}
      onfind={findJunkNext}
      onaddjunk={addSelectionAsJunk}
      onsuggestjunk={suggestJunk}
      onclean={cleanForTts}
      ongenerate={generate}
      onfetch={fetchArticle}
    />
  </div>

  <QueuePanel />
</div>

<style>
  .desk {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .deck {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .script {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
  }

  .editor {
    flex: 1;
    min-width: 0;
    /* CodeMirror's .cm-scroller owns scrolling; hidden avoids a double bar */
    overflow: hidden;
  }

  .coach {
    position: absolute;
    bottom: 14px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--panel-raised);
    border: 1px solid var(--accent-dim);
    border-radius: 6px;
    padding: 8px 14px;
    font-size: 12.5px;
    box-shadow: 0 8px 24px var(--shadow);
    white-space: nowrap;
    animation: coach-in 160ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes coach-in {
    from {
      opacity: 0;
      transform: translate(-50%, 4px);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .coach {
      animation-name: coach-fade;
    }
  }

  @keyframes coach-fade {
    from {
      opacity: 0;
    }
  }
</style>
