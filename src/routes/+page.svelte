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
    getText,
    setText,
    showJunkMatch,
  } from "$lib/editor";
  import { app, applyTheme, initApp, toast } from "$lib/stores.svelte";
  import type { AuthorGender, CapturedArticle, Extracted, Theme } from "$lib/types";
  import Header from "$lib/components/Header.svelte";
  import Slate from "$lib/components/Slate.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import VoicesDialog from "$lib/components/VoicesDialog.svelte";
  import JunkDialog from "$lib/components/JunkDialog.svelte";
  import QueueDialog from "$lib/components/QueueDialog.svelte";
  import LogDialog from "$lib/components/LogDialog.svelte";
  import FeedDialog from "$lib/components/FeedDialog.svelte";

  let editorHost: HTMLDivElement;
  let view: EditorView;

  const slate = $state({ title: "", author: "", filenameTitle: "" });

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
    initApp().catch((e) => toast(`Startup failed: ${e}`, "error"));
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

  function resetFind() {
    activeMatchLine = null;
    coach = null;
    if (view) clearJunkMatch(view);
  }

  function resetSlate() {
    slate.title = "";
    slate.author = "";
    slate.filenameTitle = "";
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
    resetSlate();
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
      slate.title = autofill.title;
      slate.author = autofill.author;
      slate.filenameTitle = autofill.filename_title;
    } else {
      // Extracted content: readability already separated the real title/author,
      // so line 1 of the body is prose. Keep the slate; only fill gaps in the
      // titles, and never guess the author from body lines.
      if (!slate.title) slate.title = autofill.title;
      if (!slate.filenameTitle) slate.filenameTitle = autofill.filename_title;
    }
    resetFind();
  }

  async function generate() {
    try {
      await api.enqueueGenerate(getText(view), slate.title, slate.author, slate.filenameTitle, sourceUrl);
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
    resetSlate();
    if (title) {
      slate.title = title;
      slate.filenameTitle = title;
    }
    if (author) slate.author = author;
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

  async function changeGender(g: AuthorGender) {
    if (app.config) app.config.author_gender = g;
    await api.setAuthorGender(g);
  }

  async function changeTheme(t: Theme) {
    if (app.config) app.config.theme = t;
    applyTheme(t);
    await api.setTheme(t);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (app.dialog) app.dialog = null;
      else if (coach) coach = null;
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
    if (shift) return;

    if (key === "f") return run(findJunkNext);
    if (key === "l") return run(cleanForTts);
    if (key === "d") return run(deleteLine);
    if (key === "j") return run(() => (app.dialog = "junk"));
    if (key === "m") return run(() => (app.dialog = "voices"));
    if (key === "q") return run(() => (app.dialog = "queue"));
    if (key === "enter") return run(generate);
  }
</script>

<svelte:window onkeydowncapture={onKeydown} />

<div class="desk">
  <Header
    lamp={app.lamp}
    theme={app.config?.theme ?? "dark"}
    ontheme={changeTheme}
    onfeed={() => (app.dialog = "feed")}
  />
  <Slate {slate} gender={app.config?.author_gender ?? "Unknown"} ongender={changeGender} />

  <div class="deck">
    <main class="script">
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
      onclean={cleanForTts}
      ongenerate={generate}
      onfetch={fetchArticle}
    />
  </div>
</div>

{#if app.dialog === "voices"}
  <VoicesDialog />
{:else if app.dialog === "junk"}
  <JunkDialog />
{:else if app.dialog === "queue"}
  <QueueDialog />
{:else if app.dialog === "log"}
  <LogDialog />
{:else if app.dialog === "feed"}
  <FeedDialog />
{/if}

<div class="toasts">
  {#each app.toasts as t (t.id)}
    <div class="toast" data-kind={t.kind}>{t.text}</div>
  {/each}
</div>

<style>
  .desk {
    display: flex;
    flex-direction: column;
    height: 100vh;
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
    border: 1px solid var(--amber-dim);
    border-radius: 6px;
    padding: 8px 14px;
    font-size: 12.5px;
    box-shadow: 0 8px 24px var(--shadow);
    white-space: nowrap;
  }

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
