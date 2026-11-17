# Auto Notes button (replaces Redo)

## Goal

Add a bottom action-row button that fills every empty cell with all **correct** pencil notes in one tap, and reuse the slot currently occupied by the Redo button.

## Decisions (confirmed with user)

- **Action row becomes: Undo / Auto Notes / Notes / Hint.** Redo no longer has a button and its `Ctrl+Y` shortcut is removed entirely. `Game::redo`, `can_redo`, `Move::reapply` and the redo stack stay in the core (still tested, still used by undo bookkeeping); only the UI entry points are removed.
- Auto Notes is **one-shot and idempotent**: each tap recomputes notes from scratch and *replaces* each empty cell's notes. If nothing changes, no history entry is recorded (so repeats do not pollute undo).
- "Correct" means **solution-based**: a note is set iff the digit matches the known solution for that cell. This ignores wrong entries already on the board (calm mode); those wrong entries are still flagged by the existing conflict highlight.
- Filled (non-empty) cells end with zero notes.
- One tap = **one undo step** (undoing restores the exact previous note masks).
- No new settings; no extra cost (hints, mistakes) is charged.
- Keyboard shortcut for Auto Notes: **`A`** only when no text field has focus — see "Keyboard" below for how to avoid clashing with movement keys. *(Recommended default in this plan; if the user prefers no shortcut, skip the keybinding task.)*

## Task list

### 1. Core: `Game::fill_auto_notes` (`src/game.rs`)

1. Add a new `Move` variant next to `Notes`:

   ```rust
   FillNotes {
       /// Previous note masks for every cell that changes (index, mask).
       previous: Vec<(usize, usize, u16)>,
   }
   ```

2. Implement:

   ```rust
   /// Replace every cell's notes with the solution-based candidates.
   /// Empty cells get all digits that are legal per the solution; filled or
   /// given cells get no notes. Returns `true` when anything changed.
   /// Recorded as a single undo step.
   pub fn fill_auto_notes(&mut self) -> bool
   ```

   Logic:
   - Build the target mask per cell: if `cells[r][c] != EMPTY` → `0`; else for `digit in 1..=9`, set bit when the digit does not already appear as a value in row/col/box **of the solution** (`self.solution`), i.e. run the existing `is_safe`/`peers` helper against `self.solution`.
   - Diff against `self.notes`; if identical, return `false` without touching history/redo.
   - Otherwise push `Move::FillNotes { previous }` (only changed cells), `self.redo.clear()`, apply the new masks, return `true`.
   - Edge case: a wrong user value already in the board must **not** influence the candidates (solution-based only). A cell with a wrong value therefore still ends with zero notes (it is "filled"), which matches the user's expectation that the digit is flagged as an error instead.
   - Empty-board case (no notes at all and a fully-given puzzle) → returns `false`.

3. Extend `revert` and `reapply`:
   - `Move::FillNotes { previous }`: `revert` writes `previous` masks back; `reapply` recomputes is not possible from `previous`, so store **both** `previous` and `next` masks (recommended) and have `reapply` write `next`. This keeps redo behaviour correct for saves/undo/redo added later without recomputation.
   - Update the `Move` equality/`Debug` derives automatically (no change needed).

4. Save/load: the encoded format is unchanged because redo/history are not serialized (verify: `encode`/`decode` do not persist stacks — confirmed in `src/game.rs:402-490`). No format bump required.

5. Unit tests in `src/game.rs`:
   - `auto_notes_fill_solution_candidates`: fresh game → `fill_auto_notes()` returns true; every empty cell's mask equals expected solution-based candidates; givens/filled cells have mask 0.
   - `auto_notes_is_idempotent`: second call returns `false` and `can_undo()` count unchanged (call `undo()` twice to prove only one step was added).
   - `auto_notes_undo_restores_previous_notes`: place a manual note, call fill, undo → manual note restored exactly.
   - `auto_notes_ignores_wrong_values`: place a wrong digit in calm mode, fill notes, assert notes around it are still solution-based.
   - `auto_notes_redo_reapplies` (only if `reapply` is kept): fill, undo, redo → same auto masks.

### 2. Icon (`src/icons.rs`, `assets/icons/`)

1. Add a Material Symbols SVG (suggested glyph: `note_add` or `playlist_add_check`) to `assets/icons/<name>.svg` — download from <https://fonts.google.com/icons> and follow the existing white-fill style of the other SVGs.
2. Regenerate masks with `bash scripts/generate-icons.sh` → `assets/icons/rgba/<name>.a.gray` (48×48 8-bit alpha).
3. Add `AutoNotes => "<name>"` to `icon_set!` in `src/icons.rs` and to the test list in `every_icon_mask_is_correct_size_and_not_blank`.
4. Add the new icon file name to the Material Symbols file list in `THIRD_PARTY_NOTICES.md`.

### 3. UI: action row (`src/ui.rs`)

1. In `draw_number_pad` (around `src/ui.rs:770-855`) replace the Redo button block with an Auto Notes button:

   ```rust
   if icon_action_button(..., Icon::AutoNotes, palette.pad_text, !self.game.is_complete()).clicked() {
       self.auto_notes();
   }
   ```

   Enabled whenever the game is not complete (`is_complete()`); a no-op tap is harmless because the core returns `false`.
   - Hover text: `"Auto notes"` (and optionally `"Auto notes (A)"` if the shortcut task is done).
2. Update the comment at `src/ui.rs:22` and `src/ui.rs:770` (`undo / auto-notes / notes / hint`).
3. Add helper on `SudokuApp` next to `hint()`:

   ```rust
   fn auto_notes(&mut self) {
       if self.game.fill_auto_notes() {
           self.start_clock();
           self.after_move();
       }
   }
   ```

   `start_clock()` matches other edits so the timer starts on the first action; no flash/hint changes are needed (`after_move` already handles save + win check).
   - Left-handed mirroring already applies to the whole action row, so no layout change is needed.

### 4. Keyboard (`src/ui.rs`, ~`src/ui.rs:1211-1238`)

- Remove the `egui::Key::Y if modifiers.command` redo arm.
- Add auto notes. **Conflict warning:** `A` is already bound to "move selection left" (`egui::Key::ArrowLeft | egui::Key::A`). Do not silently steal it. Options for the implementer, pick one:
  - Recommended: bind `egui::Key::A if modifiers.command` (Ctrl/Cmd+A, consistent with common "select all" semantics and not conflicting with WASD movement).
  - Alternative: leave keyboard out of scope and rely on the button only.
- Keep `Ctrl+Z` undo untouched.
- Update the keyboard hint list in `Feature_Plan.md` §2.6 if it enumerates keys (currently lists `N, H, Z` and Ctrl+Y — drop Ctrl+Y, add Ctrl+A).

### 5. Docs (`Feature_Plan.md`)

- §2.6: replace the "Add Redo (… plus Ctrl+Y)" line with a note that the redo *button* was replaced by Auto Notes, while the redo stack still exists internally for future use (or a short "Auto Notes" bullet under §2.5 Notes improvements).
- Add under §2.5: `[x] Auto Notes button fills all correct pencil notes in one undoable step.`

## Risks / notes

- Users who relied on the Redo button lose that affordance; undo/redo internals remain so it can be restored from a menu later if desired.
- `fill_auto_notes` must never consult `self.cells` for candidate legality, only `self.solution` (decision above). Double-check by test.
- If `reapply` for `FillNotes` is dropped (write `next` only via stored masks), keep the stored `next` masks rather than recomputing, otherwise redo after a manual note edit could diverge. Recommend storing both `previous` and `next`.

## Validation

- `cargo test` — all existing tests (including the redo-stack tests, which stay green since the core APIs are untouched) plus the new `auto_notes_*` tests.
- `cargo clippy --all-targets` clean.
- Manual desktop (`cargo run`): tap Auto Notes on an empty board → notes appear; tap again → nothing added to undo; undo once → board note state returns exactly to before; place a wrong digit in calm mode then Auto Notes → notes around it still reflect the solution; left-handed mode mirrors the row correctly; Ctrl+Z still undoes.
- Android: `./android/build-rust.sh --release` → `./gradlew :app:assembleDebug` → install, rotate device, background/foreground (autosave keeps the new notes because notes are part of the save format).
