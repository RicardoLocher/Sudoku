# Sudoku — Feature & UI Plan

Status: playable on Android (arm64). Core = `lib.rs` (generator/solver/game).
UI = `src/ui.rs` (egui, shared desktop + Android). This doc lists the next
wave: a nicer UI and quality-of-life features.

Priorities: **P0** = core polish that makes it feel finished, **P1** = high-value
QoL, **P2** = nice-to-have / later.

---

## 1. UI overhaul

### 1.1 Layout & responsiveness (P0)
- [ ] Rework layout into three zones that scale on phones: **top bar** (timer,
      difficulty, menu), **board** (max square, vertically centered), **bottom
      pad** (number pad + action buttons).
- [ ] Make the number pad full-width with large touch targets (min ~48dp),
      evenly spaced — current 40px buttons are small on phones.
- [ ] Respect safe areas / notches / gesture bar (Android insets).
- [ ] Handle both portrait and landscape (optional lock currently portrait).
- [ ] Handle screen rotation without losing the game (state is in `SudokuApp`,
      should survive; verify on device).

### 1.2 Visual style (P0)
- [ ] Centralized theme/palette struct instead of scattered `Color32` consts,
      so light/dark can swap cleanly.
- [ ] Light theme + **dark theme** (follow system, or toggle in settings).
- [ ] Rounded corners, subtle shadows, cleaner separators (3×3 boxes).
- [ ] Consistent typography scale; tabular/even digits for the board.
- [ ] App icon (currently none — uses default) + launch splash.
- [ ] Animated selection / placement feedback (short fade or pop).

### 1.3 Board rendering (P1)
- [ ] Distinguish **row/column/box** highlights with a slightly stronger box
      highlight than row/col (currently uniform).
- [ ] Colorblind-safe conflict (error) color + an icon/shape cue, not color only.
- [ ] Highlight all identical digits when one is selected (mostly done), tune colors.
- [ ] Dim/disable interaction glow on givens; make givens visually distinct.
- [ ] Diagonal "flash" when a digit completes its 9th placement.

### 1.4 Number pad (P1)
- [ ] Show **remaining count** under each digit (e.g. "9 → 2 left"), grey out at 0
      (extends the existing `is_digit_complete` behavior with a counter).
- [ ] Long-press a digit in the pad = switch to notes for that digit.
- [ ] Clear visual state for **Notes mode on/off** (toggle button, colored active).
- [ ] Layout pad as 3×3 or single row depending on width.

---

## 2. Quality-of-life features (gameplay)

### 2.1 Timer & pause (P0)
- [ ] Per-game timer (start on first move, not on puzzle load).
- [ ] Pause button → overlay that hides the board and stops the clock.
- [ ] Persist elapsed time so resuming continues correctly.
- [ ] Auto-pause when the app goes to background (`App::logic` / focus events).

### 2.2 Save / resume (P0)
- [ ] Autosave the in-progress `Game` (grid, notes, history, seed, difficulty,
      elapsed time) so closing the app resumes exactly where you left off.
- [ ] Add (de)serialization to the core: manual compact encoding to keep the lib
      dependency-free, or a `serde` feature flag.
- [ ] Storage location: app-specific dir (Android internal storage via
      `android-activity` path APIs). Load on startup.
- [ ] "Resume game" vs "New game" prompt on launch.

### 2.3 Mistakes & correctness feedback (P0/P1)
- [ ] Option A (calm): only mark **conflicts** (same digit in row/col/box),
      as today.
- [ ] Option B (strict): flag digits that differ from the known solution.
- [ ] Make it a **setting** ("Check against solution" on/off).
- [ ] Optional mistake counter with a limit (or endless mode, no limit).

### 2.4 Hints (P1)
- [ ] "Hint" button reveals the correct digit for the selected (or a random
      empty) cell.
- [ ] Limited hints (e.g. 3 per game) or a score penalty; show remaining hints.
- [ ] Hint should fill the value and clear that cell's notes; register as an undo step.

### 2.5 Notes improvements (P1)
- [ ] **Auto-remove notes**: when a digit is placed, optionally remove it from
      notes in the same row/col/box ("Clean notes" toggle).
- [ ] Long-press cell to toggle a single note without entering notes mode.
- [ ] Undo for notes already exists — verify grouping feels right.

### 2.6 Undo/redo & input (P0/P1)
- [ ] Add **Redo** (currently only Undo); wire into a small toolbar.
- [ ] Undo should revert a whole placement including auto-note cleanup.
- [ ] Keyboard input on desktop (digits 1–9, arrows, backspace); keep mobile
      first-class.
- [ ] Tap an already-filled selected cell again to erase (some apps do this) —
      decide and document the interaction rules.

### 2.7 New game & difficulty (P0)
- [ ] Difficulty picker as a clean menu/dialog (not inline labels).
- [ ] Friendly labels + "Daily puzzle" (date-seeded, same for everyone that day).
- [ ] Confirm before discarding an in-progress game.
- [ ] Generate on a background thread + loading spinner for Expert
      (measure carve time on phone first).

### 2.8 Win / progress UX (P1)
- [ ] Win screen: time, difficulty, mistakes/hints used, "New game" / "Replay".
- [ ] Celebrate completed 3×3 boxes / full board subtly.
- [ ] Progress indicator (% filled) optional.

---

## 3. Settings screen (P1)
- [ ] Theme (system/light/dark).
- [ ] Check-against-solution (strict vs calm).
- [ ] Auto-remove notes toggle.
- [ ] Highlight same digit / highlight row-col-box toggles.
- [ ] Left-handed number pad option.
- [ ] Persist settings alongside the save file.

---

## 4. Core / engine improvements (P1/P2)
- [ ] **Better difficulty rating**: current tiers only vary clue count. Rate by
      solving techniques (singles → locked candidates → pointing pairs → …) and
      map to Easy/Medium/Hard/Expert.
- [ ] Generation performance pass + a thread with loading UI.
- [ ] Property tests: many seeds × tiers → always uniquely solvable; givens
      consistent with solution (some exist already in `generator.rs`/`game.rs`).
- [ ] Puzzle symmetry option (rotational symmetry when carving) for aesthetics.
- [ ] Optional seeded "Daily challenge".

---

## 5. Platform / release (P2)
- [ ] Release build: signing config, `./gradlew :app:assembleRelease`.
- [ ] Versioning + Play Store / F-Droid listing (free, no ads, no IAP).
- [ ] Crash reporting / logging (`adb logcat` sanity on device).
- [ ] Accessibility: scalable fonts, content descriptions, screen-reader basics.
- [ ] Localization strings externalized (start with EN, structure for i18n).

---

## Suggested implementation order
1. **Theme + layout + number pad sizing** (P0 §1.1, §1.2, §1.4) — biggest visible win.
2. **Timer + pause** and **autosave/resume** (§2.1, §2.2) — makes it feel like a real app.
3. **Settings scaffold** (§3) so new toggles have a home.
4. **Hints + notes improvements + redo** (§2.4, §2.5, §2.6).
5. **Win screen + difficulty menu/daily** (§2.7, §2.8).
6. **Difficulty rating + generation perf** (§4).

## Testing checklist (per feature)
- `cargo test` stays green; add unit tests for any new core logic (notes cleanup,
  hint application, save/load round-trip).
- `cargo clippy` clean.
- Manual: desktop (`cargo run`) then Android (`./android/build-rust.sh --release`
  → `./gradlew :app:assembleDebug` → `adb install -r ...`).
- Rotate device, background/foreground, kill+relaunch (save/resume), airplane mode.
