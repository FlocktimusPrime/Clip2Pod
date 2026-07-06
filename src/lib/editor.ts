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
    caretColor: "var(--amber)",
    padding: "10px 0",
  },
  ".cm-line": { padding: "0 12px" },
  ".cm-scroller": { overflow: "auto" },
  "&.cm-focused": { outline: "none" },
  "&.cm-focused .cm-cursor": { borderLeftColor: "var(--amber)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
    backgroundColor: "color-mix(in srgb, var(--amber) 22%, transparent) !important",
  },
  ".cm-placeholder": { color: "var(--muted)" },
  ".junk-hit": {
    backgroundColor: "color-mix(in srgb, var(--onair) 30%, transparent)",
    outline: "1px solid var(--onair)",
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

export function setText(view: EditorView, text: string) {
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: text },
    effects: setJunkHighlight.of(null),
  });
}

export const deleteCurrentLine = (view: EditorView) => deleteLine(view);

/**
 * Select the matched line and highlight the phrase inside it. The byte
 * offsets from Rust don't translate to UTF-16, so the phrase is re-located
 * with a case-insensitive search in JS.
 */
export function showJunkMatch(view: EditorView, lineIdx: number, phrase: string) {
  if (lineIdx >= view.state.doc.lines) return;
  const line = view.state.doc.line(lineIdx + 1);
  const at = line.text.toLowerCase().indexOf(phrase.toLowerCase());
  const effects: StateEffect<unknown>[] = [
    setJunkHighlight.of(at >= 0 ? { from: line.from + at, to: line.from + at + phrase.length } : null),
    EditorView.scrollIntoView(line.from, { y: "center" }),
  ];
  view.dispatch({ selection: { anchor: line.from, head: line.to }, effects });
  view.focus();
}

export function clearJunkMatch(view: EditorView) {
  view.dispatch({ effects: setJunkHighlight.of(null) });
}
