// CodeMirror 6 setup for the script editor: line-based selection for the junk
// finder, a mark decoration for the matched phrase, and a theme driven by the
// production-desk CSS variables so it follows the day/night rocker for free.

import { defaultKeymap, deleteLine, history, historyKeymap } from "@codemirror/commands";
import { EditorState, StateEffect, StateField } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  keymap,
  placeholder,
} from "@codemirror/view";

const setJunkHighlight = StateEffect.define<{ from: number; to: number } | null>();

const junkHighlight = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(setJunkHighlight)) {
        deco = e.value
          ? Decoration.set([Decoration.mark({ class: "junk-hit" }).range(e.value.from, e.value.to)])
          : Decoration.none;
      }
    }
    // Any edit invalidates the highlight; the next Find Junk recomputes it.
    if (tr.docChanged) deco = Decoration.none;
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

const deskTheme = EditorView.theme({
  "&": {
    height: "100%",
    fontSize: "13.5px",
    backgroundColor: "var(--bg)",
    color: "var(--text)",
  },
  ".cm-content": {
    fontFamily: "var(--sans)",
    caretColor: "var(--accent)",
    padding: "10px 0",
  },
  ".cm-line": { padding: "0 12px" },
  ".cm-scroller": { overflow: "auto" },
  "&.cm-focused": { outline: "none" },
  "&.cm-focused .cm-cursor": { borderLeftColor: "var(--accent)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
    backgroundColor: "color-mix(in srgb, var(--accent) 22%, transparent) !important",
  },
  ".cm-placeholder": { color: "var(--muted)" },
  ".junk-hit": {
    backgroundColor: "color-mix(in srgb, var(--danger) 30%, transparent)",
    outline: "1px solid var(--danger)",
    borderRadius: "2px",
  },
});

export function createEditor(parent: HTMLElement): EditorView {
  return new EditorView({
    parent,
    state: EditorState.create({
      extensions: [
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        EditorView.lineWrapping,
        junkHighlight,
        deskTheme,
        placeholder("Paste an article here (Ctrl+Shift+V grabs the clipboard)…"),
      ],
    }),
  });
}

export const getText = (view: EditorView) => view.state.doc.toString();

/** Currently selected text (empty string when the selection is a caret). */
export const getSelection = (view: EditorView) =>
  view.state.sliceDoc(view.state.selection.main.from, view.state.selection.main.to);

export function setText(view: EditorView, text: string) {
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: text },
    effects: setJunkHighlight.of(null),
  });
}

export const deleteCurrentLine = (view: EditorView) => deleteLine(view);

/** Char range of `phrase` within `text` (case-insensitive). A `*` in the
 *  phrase matches any run of characters. Null when it isn't found. */
function locatePhrase(text: string, phrase: string): { from: number; to: number } | null {
  const lower = text.toLowerCase();
  if (!phrase.includes("*")) {
    const at = lower.indexOf(phrase.toLowerCase());
    return at >= 0 ? { from: at, to: at + phrase.length } : null;
  }
  const pattern = phrase
    .toLowerCase()
    .split("*")
    .filter(Boolean)
    .map((s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    .join(".*?");
  const m = pattern ? new RegExp(pattern).exec(lower) : null;
  return m ? { from: m.index, to: m.index + m[0].length } : null;
}

/**
 * Select the matched line and highlight the phrase inside it. The offsets
 * from Rust don't translate to UTF-16, so the phrase is re-located in JS.
 */
export function showJunkMatch(view: EditorView, lineIdx: number, phrase: string) {
  if (lineIdx >= view.state.doc.lines) return;
  const line = view.state.doc.line(lineIdx + 1);
  const hit = locatePhrase(line.text, phrase);
  const effects: StateEffect<unknown>[] = [
    setJunkHighlight.of(hit ? { from: line.from + hit.from, to: line.from + hit.to } : null),
    EditorView.scrollIntoView(line.from, { y: "center" }),
  ];
  view.dispatch({ selection: { anchor: line.from, head: line.to }, effects });
  view.focus();
}

export function clearJunkMatch(view: EditorView) {
  view.dispatch({ effects: setJunkHighlight.of(null) });
}
