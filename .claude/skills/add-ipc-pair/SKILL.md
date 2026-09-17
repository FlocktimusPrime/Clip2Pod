---
name: add-ipc-pair
description: Use when adding a new Tauri command, backend event, or TS wrapper to Clip2Pod — picks the correct narrate (bare) vs rip (rip_/rip:) naming convention and shows every file that needs a matching change.
---

# add-ipc-pair

Clip2Pod has two independent modes sharing one shell. Which mode a new
command/event belongs to decides its naming and which files it touches:

| | Narrate | Rip |
|---|---|---|
| Command file | `src-tauri/src/commands.rs` | `src-tauri/src/rip_commands.rs` |
| Command name | bare, e.g. `clean_text` | `rip_`-prefixed, e.g. `rip_enqueue` |
| Worker/event file | `src-tauri/src/worker.rs` | `src-tauri/src/rip_worker.rs` |
| Event name | bare, e.g. `"voices-changed"`, `"lamp"` | `"rip:"`-prefixed, e.g. `"rip:lamp"` |
| TS wrapper | `src/lib/api.ts` | `src/lib/rip_api.ts` (import as `* as rip`) |
| TS types | unprefixed, in `src/lib/types.ts` | `Rip`-prefixed, e.g. `RipJob` |
| Handler registration | `src-tauri/src/lib.rs`, the `tauri::generate_handler![...]` list (shared by both) | same list |

## Steps

1. Add the `#[tauri::command]` function to the matching command file, named
   per the table above.
2. Register it in the `generate_handler![...]` list in `src-tauri/src/lib.rs`
   as `commands::your_fn` or `rip_commands::your_fn`.
3. If it emits an event, do it from the matching worker file with
   `app.emit(...)`, using the bare or `"rip:"`-prefixed name.
4. Add the TS wrapper to the matching file (`api.ts` or `rip_api.ts`),
   following the existing `invoke<ReturnType>("command_name", { args })` style.
   Add any new payload type to `src/lib/types.ts`, prefixed `Rip` if it's a rip
   type.

## Example (narrate, from the existing code)

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub fn clean_text(text: String) -> CleanResult { /* ... */ }
```
```rust
// src-tauri/src/lib.rs — inside generate_handler![...]
commands::clean_text,
```
```ts
// src/lib/api.ts
export const cleanText = (text: string) => invoke<CleanResult>("clean_text", { text });
```

The rip equivalent is the same shape with the `rip_` prefix throughout:
`rip_commands::rip_enqueue`, `rip.enqueue`, `invoke<string>("rip_enqueue", ...)`.

Don't mirror a change into the other mode just because it's the "matching"
file — narrate and rip queues are intentionally independent. Only add to both
when the feature genuinely belongs to both modes.
