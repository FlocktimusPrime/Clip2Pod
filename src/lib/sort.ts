// Column-sort state shared by the voices / queue / log tables.
// Cycle per header click: natural order → ascending → descending → natural.

export type SortState<K extends string> = { key: K; dir: 1 | -1 } | null;

export function toggleSort<K extends string>(state: SortState<K>, key: K): SortState<K> {
  if (state?.key !== key) return { key, dir: 1 };
  return state.dir === 1 ? { key, dir: -1 } : null;
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}T/;

/** Numbers numerically, ISO timestamps chronologically, rest as text. */
export function cmp(a: unknown, b: unknown): number {
  if (a == null || b == null) return a == null ? (b == null ? 0 : -1) : 1;
  if (typeof a === "number" && typeof b === "number") return a - b;
  const sa = String(a);
  const sb = String(b);
  if (ISO_DATE.test(sa) && ISO_DATE.test(sb)) {
    return new Date(sa).getTime() - new Date(sb).getTime();
  }
  return sa.localeCompare(sb, undefined, { sensitivity: "base" });
}

export function sortRows<T, K extends string>(
  rows: T[],
  state: SortState<K>,
  pick: (row: T, key: K) => unknown,
): T[] {
  if (!state) return rows;
  return [...rows].sort((x, y) => state.dir * cmp(pick(x, state.key), pick(y, state.key)));
}

export function sortIndicator<K extends string>(state: SortState<K>, key: K): string {
  if (state?.key !== key) return "";
  return state.dir === 1 ? "▲" : "▼";
}

export function ariaSort<K extends string>(
  state: SortState<K>,
  key: K,
): "ascending" | "descending" | "none" {
  if (state?.key !== key) return "none";
  return state.dir === 1 ? "ascending" : "descending";
}
