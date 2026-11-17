# Sudoku — Feature & UI Plan

Status: playable on Android (arm64). Core = `lib.rs` (generator/solver/game).
UI = `src/ui.rs` (egui, shared desktop + Android). This doc lists the next
wave: a nicer UI and quality-of-life features.

Priorities: **P0** = core polish that makes it feel finished, **P1** = high-value
QoL, **P2** = nice-to-have / later.

---

## 1. UI overhaul

### 1.1 Layout & responsiveness (P0)
- [x] Rework layout into three zones that scale on phones: **top bar** (timer,
      difficulty, menu), **board** (max square, vertically centered), **bottom
      pad** (number pad + action buttons).
- [x] Make the number pad full-width with large touch targets (min ~48dp),
      evenly spaced — current 40px buttons are small on phones.
- [x] Respect safe areas / notches / gesture bar (Android insets).
- [x] Handle both portrait and landscape (`screenOrientation=fullSensor`;
      pad switches 3×3 → single row by width).
- [x] Handle screen rotation without losing the game (state is in `SudokuApp`,
      survives; and is autosaved so even a process restart resumes).

### 1.2 Visual style (P0)
- [x] Centralized theme/palette struct instead of scattered `Color32` consts,
      so light/dark can swap cleanly (`src/theme.rs`).
- [x] Light theme + **dark theme** (follow system, or toggle in settings).
- [x] Rounded corners, subtle shadows, cleaner separators (3×3 boxes).
- [x] Consistent typography scale; tabular/even digits for the board.
- [x] App icon (adaptive vector) + launch splash.
- [x] Animated selection / placement feedback (pulsing selection ring, digit
      completion diagonal flash).

### 1.3 Board rendering (P1)
- [x] Distinguish **row/column/box** highlights with a slightly stronger box
      highlight than row/col.
- [x] Colorblind-safe conflict (error) color + an icon/shape cue, not color only
      (triangle badge in the cell corner).
- [x] Highlight all identical digits when one is selected, tuned colors.
- [x] Dim/disable interaction glow on givens; givens use a distinct color.
- [x] Diagonal "flash" when a digit completes its 9th placement.

### 1.4 Number pad (P1)
- [x] Show **remaining count** under each digit, grey out at 0.
- [x] Long-press a digit in the pad = switch to notes for that digit.
- [x] Clear visual state for **Notes mode on/off** (colored active toggle).
- [x] Layout pad as 3×3/single row depending on width.

---

## 2. Quality-of-life features (gameplay)

### 2.1 Timer & pause (P0)
- [x] Per-game timer (starts on first move, not on puzzle load).
- [x] Pause button → overlay that hides the board and stops the clock.
- [x] Persist elapsed time so resuming continues correctly.
- [x] Auto-pause when the app goes to background (`App::logic` / focus events).

### 2.2 Save / resume (P0)
- [x] Autosave the in-progress `Game` (grid, notes, history, seed, difficulty,
      elapsed time) so closing the app resumes exactly where you left off.
- [x] Add (de)serialization to the core: compact manual encoding, no deps.
- [x] Storage location: app-specific dir (Android internal storage via
      `android-activity`); loaded on startup.
- [ ] "Resume game" vs "New game" prompt on launch (currently always resumes;
      new game is one tap away in the menu).

### 2.3 Mistakes & correctness feedback (P0/P1)
- [x] Option A (calm): only mark **conflicts** (duplicates in row/col/box).
- [x] Option B (strict): flag digits that differ from the known solution.
- [x] This is a **setting** ("Check against solution" on/off).
- [x] Mistake counter (shown on the win screen; no hard limit = endless mode).

### 2.4 Hints (P1)
- [x] "Hint" button reveals the correct digit for the selected (or first empty)
      cell.
- [x] Limited hints (3 per game); remaining hints shown on the button.
- [x] Hint fills the value and clears that cell's notes; is one undo step.

### 2.5 Notes improvements (P1)
- [x] **Auto-remove notes**: when a digit is placed, remove it from notes in the
      same row/col/box ("Clean notes" toggle).
- [x] Long-press a digit in the pad to toggle notes for that digit.
- [x] Undo groups a placement with its auto-note cleanup as one step.
- [x] **Auto notes** button (bottom action row, replacing the Redo button):
      fills every empty cell with its pencil candidates for the current board,
      leaving out the cell's solution digit so nothing is given away. One tap =
      one undo step; tapping again is a no-op.

### 2.6 Undo/redo & input (P0/P1)
- [x] Redo stack in the core (`Game::redo`, used by tests); its button was later
      replaced by **Auto notes**, so redo is no longer reachable from the UI.
- [x] Undo reverts a whole placement including auto-note cleanup.
- [x] Keyboard input on desktop (digits 1–9, arrows/WASD, backspace, N, H, Z,
      Ctrl+A for auto notes); mobile stays first-class.
- [x] Tap an already-filled selected cell's digit again to erase (documented in
      `Game::play`).

### 2.7 New game & difficulty (P0)
- [x] Difficulty picker as a clean menu (not inline labels).
- [x] Friendly labels + "Daily puzzle" (date-seeded, same for everyone that day).
- [x] Confirm before discarding an in-progress game.
- [ ] Generate on a background thread + loading spinner for Expert (generation
      is fast enough at current clue counts; revisit if carving slows down).

### 2.8 Win / progress UX (P1)
- [x] Win screen: time, difficulty, mistakes/hints used, "New game" / "Replay".
- [x] Celebrate completed digits with the diagonal flash.
- [x] Progress indicator (thin % filled bar under the top bar).

---

## 3. Settings screen (P1)
- [x] Theme (system/light/dark).
- [x] Check-against-solution (strict vs calm).
- [x] Auto-remove notes toggle.
- [x] Highlight same digit / highlight row-col-box toggles.
- [x] Left-handed number pad option.
- [x] Persist settings alongside the save file.

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
- [x] App icon + launch splash (adaptive vector icon, themed splash window).
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
