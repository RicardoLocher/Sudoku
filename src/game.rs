use crate::generator::{Difficulty, Puzzle};
use crate::grid::{EMPTY, Grid, SIZE, peers};
use crate::settings::Settings;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Move {
    Value {
        row: usize,
        col: usize,
        previous: u8,
        next: u8,
        /// Note masks (index, mask) changed by this placement, so a single
        /// undo also reverts auto-note cleanup and the cell's own note wipe.
        notes: Vec<(usize, usize, u16)>,
    },
    Notes {
        row: usize,
        col: usize,
        previous: u16,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub mistakes: u32,
    pub hints_used: u32,
}

#[derive(Debug, Clone)]
pub struct Game {
    solution: Grid,
    givens: [[bool; SIZE]; SIZE],
    cells: Grid,
    notes: [[u16; SIZE]; SIZE],
    history: Vec<Move>,
    redo: Vec<Move>,
    stats: Stats,
    settings: Settings,
    difficulty: Difficulty,
    /// Seed the puzzle was generated from, kept so saves can be reconstructed.
    seed: u64,
    /// Set for one frame when a placement makes a digit reach nine copies, so
    /// the UI can play the diagonal flash animation.
    flash_digit: Option<u8>,
}

impl Game {
    pub fn new(puzzle: Puzzle) -> Self {
        Self::with_settings(puzzle, Settings::default())
    }

    pub fn with_settings(puzzle: Puzzle, settings: Settings) -> Self {
        let mut givens = [[false; SIZE]; SIZE];
        for (row, cells) in puzzle.grid.iter().enumerate() {
            for (col, &value) in cells.iter().enumerate() {
                givens[row][col] = value != EMPTY;
            }
        }

        Self {
            solution: puzzle.solution,
            givens,
            cells: puzzle.grid,
            notes: [[0; SIZE]; SIZE],
            history: Vec::new(),
            redo: Vec::new(),
            stats: Stats::default(),
            settings,
            difficulty: puzzle.difficulty,
            seed: 0,
            flash_digit: None,
        }
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn set_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub fn solution(&self) -> &Grid {
        &self.solution
    }

    pub fn value(&self, row: usize, col: usize) -> u8 {
        self.cells[row][col]
    }

    pub fn is_given(&self, row: usize, col: usize) -> bool {
        self.givens[row][col]
    }

    pub fn notes(&self, row: usize, col: usize) -> u16 {
        self.notes[row][col]
    }

    pub fn can_place(&self, row: usize, col: usize, value: u8) -> bool {
        !self.is_given(row, col) && self.solution[row][col] == value
    }

    /// Apply a value to a cell as if typed by the player, returning `true` when
    /// something actually changed. Passing the same value that is already there
    /// clears the cell. Digital notes on the cell are wiped on placement, and
    /// (when enabled) the digit is auto-removed from row/col/box neighbours.
    ///
    /// All of these changes are recorded as a single undo step.
    pub fn play(&mut self, row: usize, col: usize, value: u8) -> bool {
        if self.is_given(row, col) || self.cells[row][col] == value {
            return false;
        }

        let previous = self.cells[row][col];

        // Work out which notes to touch so the whole move is one undo step.
        let mut note_changes: Vec<(usize, usize, u16)> = Vec::new();
        if self.notes[row][col] != 0 {
            note_changes.push((row, col, self.notes[row][col]));
        }
        if value != EMPTY && self.settings.auto_remove_notes {
            let bit = 1u16 << value;
            for (r, c) in peers(row, col) {
                if r == row && c == col {
                    continue;
                }
                if self.notes[r][c] & bit != 0 {
                    note_changes.push((r, c, self.notes[r][c]));
                }
            }
        }

        self.history.push(Move::Value {
            row,
            col,
            previous,
            next: value,
            notes: note_changes.clone(),
        });
        self.redo.clear();

        // Apply: wipe this cell's notes, set the value, strip neighbour notes.
        self.notes[row][col] = 0;
        self.cells[row][col] = value;
        for (r, c, mask) in note_changes {
            if r == row && c == col {
                continue;
            }
            self.notes[r][c] = mask & !(1u16 << value);
        }

        if value != EMPTY {
            if self.settings.check_against_solution && value != self.solution[row][col] {
                self.stats.mistakes += 1;
            }
            if self.digit_count(value) == SIZE {
                self.flash_digit = Some(value);
            }
        }

        true
    }

    /// Place the correct digit for the selected cell (or a requested one),
    /// spending a hint. Returns the cell that was filled, if any.
    pub fn apply_hint(&mut self, row: usize, col: usize) -> Option<(usize, usize)> {
        if self.is_given(row, col) || self.cells[row][col] != EMPTY {
            return None;
        }
        let value = self.solution[row][col];
        self.stats.hints_used += 1;
        self.play(row, col, value);
        Some((row, col))
    }

    /// First empty cell in reading order, used for a random-ish hint target.
    pub fn first_empty(&self) -> Option<(usize, usize)> {
        for row in 0..SIZE {
            for col in 0..SIZE {
                if self.cells[row][col] == EMPTY && !self.is_given(row, col) {
                    return Some((row, col));
                }
            }
        }
        None
    }

    pub fn toggle_note(&mut self, row: usize, col: usize, digit: u8) {
        if self.is_given(row, col) || self.cells[row][col] != EMPTY {
            return;
        }

        self.history.push(Move::Notes {
            row,
            col,
            previous: self.notes[row][col],
        });
        self.redo.clear();

        let bit = 1u16 << digit;
        self.notes[row][col] ^= bit;
    }

    pub fn has_note(&self, row: usize, col: usize, digit: u8) -> bool {
        self.notes[row][col] & (1u16 << digit) != 0
    }

    pub fn undo(&mut self) -> bool {
        match self.history.pop() {
            Some(mv) => {
                self.revert(&mv);
                self.redo.push(mv);
                true
            }
            None => false,
        }
    }

    pub fn redo(&mut self) -> bool {
        match self.redo.pop() {
            Some(mv) => {
                self.reapply(&mv);
                self.history.push(mv);
                true
            }
            None => false,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn revert(&mut self, mv: &Move) {
        match mv {
            Move::Value {
                row,
                col,
                previous,
                notes,
                ..
            } => {
                self.cells[*row][*col] = *previous;
                for (r, c, mask) in notes {
                    self.notes[*r][*c] = *mask;
                }
            }
            Move::Notes { row, col, previous } => self.notes[*row][*col] = *previous,
        }
    }

    fn reapply(&mut self, mv: &Move) {
        match mv {
            Move::Value {
                row,
                col,
                next,
                notes,
                ..
            } => {
                self.cells[*row][*col] = *next;
                if *next != EMPTY {
                    self.notes[*row][*col] = 0;
                    for (r, c, mask) in notes {
                        if *r == *row && *c == *col {
                            continue;
                        }
                        self.notes[*r][*c] = mask & !(1u16 << next);
                    }
                } else {
                    for (r, c, mask) in notes {
                        self.notes[*r][*c] = *mask;
                    }
                }
            }
            Move::Notes { row, col, previous } => self.notes[*row][*col] = *previous,
        }
    }

    /// Take (and clear) the pending "digit completed" flash, if any.
    pub fn take_flash(&mut self) -> Option<u8> {
        self.flash_digit.take()
    }

    pub fn is_complete(&self) -> bool {
        self.cells == self.solution
    }

    /// Number of filled cells that match the solution.
    pub fn filled_count(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&v| v != EMPTY)
            .count()
    }

    pub fn digit_count(&self, digit: u8) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&value| value == digit)
            .count()
    }

    pub fn digit_remaining(&self, digit: u8) -> u8 {
        SIZE.saturating_sub(self.digit_count(digit)) as u8
    }

    pub fn is_digit_complete(&self, digit: u8) -> bool {
        self.digit_count(digit) >= SIZE
    }

    /// Cells that should be visually flagged right now. In calm mode these are
    /// duplicates within a row/col/box; in strict mode they are digits that
    /// differ from the known solution.
    pub fn conflicts(&self) -> Vec<(usize, usize)> {
        if self.settings.check_against_solution {
            let mut result = Vec::new();
            for row in 0..SIZE {
                for col in 0..SIZE {
                    let value = self.cells[row][col];
                    if value != EMPTY && value != self.solution[row][col] {
                        result.push((row, col));
                    }
                }
            }
            result
        } else {
            self.duplicate_cells()
        }
    }

    fn duplicate_cells(&self) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        for row in 0..SIZE {
            for col in 0..SIZE {
                let value = self.cells[row][col];
                if value == EMPTY {
                    continue;
                }
                let duplicate = peers(row, col)
                    .any(|(r, c)| !(r == row && c == col) && self.cells[r][c] == value);
                if duplicate {
                    result.push((row, col));
                }
            }
        }
        result
    }

    /// Compact, dependency-free encoding of the full game state.
    ///
    /// Layout (newline separated):
    ///   `sudoku-save-v1`
    ///   `difficulty\ttabs-separated settings`
    ///   `seed`
    ///   solution rows (9 lines, digits 0-9)
    ///   givens rows (9 lines, 0/1)
    ///   current grid rows (9 lines, digits 0-9)
    ///   note rows (9 lines, 16-bit values 0-511)
    ///   stats line: `mistakes hints`
    pub fn encode(&self) -> String {
        let mut out = String::from("sudoku-save-v1\n");
        out.push_str(self.difficulty.key());
        out.push('\t');
        out.push_str(
            &self
                .settings
                .to_text()
                .lines()
                .collect::<Vec<_>>()
                .join("\t"),
        );
        out.push('\n');
        out.push_str(&format!("{}\n", self.seed));
        encode_grid(&self.solution, &mut out, |v| v);
        encode_bools(&self.givens, &mut out);
        encode_grid(&self.cells, &mut out, |v| v);
        for row in 0..SIZE {
            let cells: Vec<String> = (0..SIZE).map(|c| self.notes[row][c].to_string()).collect();
            out.push_str(&cells.join(" "));
            out.push('\n');
        }
        out.push_str(&format!(
            "{}\t{}\n",
            self.stats.mistakes, self.stats.hints_used
        ));
        out
    }

    pub fn decode(text: &str) -> Option<Game> {
        let mut lines = text.lines();
        if lines.next()? != "sudoku-save-v1" {
            return None;
        }
        let header = lines.next()?;
        let (difficulty, settings_text) = header.split_once('\t').unwrap_or(("medium", header));
        let difficulty = Difficulty::from_key(difficulty.trim()).unwrap_or(Difficulty::Medium);
        let settings = Settings::from_text(&settings_text.replace('\t', "\n"));
        let seed: u64 = lines.next()?.trim().parse().unwrap_or(0);

        let mut solution = [[EMPTY; SIZE]; SIZE];
        parse_digit_rows(&mut lines, &mut solution)?;

        let mut givens = [[false; SIZE]; SIZE];
        for row in givens.iter_mut() {
            let line = lines.next()?;
            let bits: Vec<char> = line.chars().collect();
            if bits.len() != SIZE {
                return None;
            }
            for (col, ch) in bits.iter().enumerate() {
                row[col] = *ch == '1';
            }
        }

        let mut cells = [[EMPTY; SIZE]; SIZE];
        parse_digit_rows(&mut lines, &mut cells)?;

        let mut notes = [[0u16; SIZE]; SIZE];
        for row in notes.iter_mut() {
            let line = lines.next()?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != SIZE {
                return None;
            }
            for (col, part) in parts.iter().enumerate() {
                row[col] = part.parse().ok()?;
            }
        }

        let stats_line = lines.next()?;
        let mut stats_parts = stats_line.split_whitespace();
        let mistakes = stats_parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let hints_used = stats_parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(Game {
            solution,
            givens,
            cells,
            notes,
            history: Vec::new(),
            redo: Vec::new(),
            stats: Stats {
                mistakes,
                hints_used,
            },
            settings,
            difficulty,
            seed,
            flash_digit: None,
        })
    }
}

fn encode_grid(grid: &Grid, out: &mut String, f: impl Fn(u8) -> u8) {
    for row in grid.iter() {
        for value in row.iter() {
            out.push(char::from(b'0' + f(*value)));
        }
        out.push('\n');
    }
}

fn encode_bools(grid: &[[bool; SIZE]; SIZE], out: &mut String) {
    for row in grid.iter() {
        for value in row.iter() {
            out.push(if *value { '1' } else { '0' });
        }
        out.push('\n');
    }
}

fn parse_digit_rows<'a>(lines: &mut impl Iterator<Item = &'a str>, grid: &mut Grid) -> Option<()> {
    for row in grid.iter_mut() {
        let line = lines.next()?;
        let chars: Vec<char> = line.chars().collect();
        if chars.len() != SIZE {
            return None;
        }
        for (col, ch) in chars.iter().enumerate() {
            row[col] = ch.to_digit(10)? as u8;
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{Difficulty, generate};

    fn game() -> Game {
        Game::new(generate(Difficulty::Easy, 2026))
    }

    #[test]
    fn givens_are_immutable() {
        let mut game = game();
        let mut given = None;
        for row in 0..SIZE {
            for col in 0..SIZE {
                if game.is_given(row, col) {
                    given = Some((row, col));
                }
            }
        }
        let (row, col) = given.expect("puzzle has a given");
        let before = game.value(row, col);
        game.play(row, col, 0);
        assert_eq!(game.value(row, col), before);
    }

    #[test]
    fn placing_correct_value_and_undo_round_trips() {
        let mut game = game();
        let mut target = None;
        for row in 0..SIZE {
            for col in 0..SIZE {
                if !game.is_given(row, col) {
                    target = Some((row, col));
                    break;
                }
            }
        }
        let (row, col) = target.expect("puzzle has an empty cell");
        let value = game.solution[row][col];
        assert!(game.can_place(row, col, value));
        game.play(row, col, value);
        assert_eq!(game.value(row, col), value);
        assert!(game.undo());
        assert_eq!(game.value(row, col), EMPTY);
    }

    #[test]
    fn notes_toggle_and_undo() {
        let mut game = game();
        let mut target = None;
        for row in 0..SIZE {
            for col in 0..SIZE {
                if !game.is_given(row, col) {
                    target = Some((row, col));
                    break;
                }
            }
        }
        let (row, col) = target.expect("puzzle has an empty cell");
        game.toggle_note(row, col, 3);
        assert!(game.has_note(row, col, 3));
        game.undo();
        assert!(!game.has_note(row, col, 3));
    }

    #[test]
    fn digit_complete_after_nine_placements() {
        let mut game = game();
        let mut digit = 0;
        for row in 0..SIZE {
            for col in 0..SIZE {
                if !game.is_given(row, col) {
                    digit = game.solution[row][col];
                }
            }
        }
        assert!(digit != 0);
        assert!(!game.is_digit_complete(digit));

        for row in 0..SIZE {
            for col in 0..SIZE {
                if game.solution[row][col] == digit && !game.is_given(row, col) {
                    game.play(row, col, digit);
                }
            }
        }

        assert_eq!(game.digit_count(digit), 9);
        assert!(game.is_digit_complete(digit));
    }

    #[test]
    fn digit_remaining_saturates_on_duplicate_digits() {
        let mut game = game();
        let empty: Vec<(usize, usize)> = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .filter(|&(r, c)| !game.is_given(r, c))
            .collect();
        assert!(empty.len() > SIZE, "puzzle needs enough blanks");

        // Calm mode allows placing the same digit on every empty cell, which can
        // push the count past nine; digit_remaining must not underflow.
        for &(r, c) in empty.iter().take(SIZE + 1) {
            game.play(r, c, 5);
        }
        assert!(game.digit_count(5) > SIZE);
        assert_eq!(game.digit_remaining(5), 0);
    }

    #[test]
    fn redo_reapplies_after_undo() {
        let mut game = game();
        let (row, col) = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .find(|&(r, c)| !game.is_given(r, c))
            .expect("empty cell");
        let value = game.solution[row][col];

        game.play(row, col, value);
        assert!(game.undo());
        assert_eq!(game.value(row, col), EMPTY);
        assert!(game.can_redo());
        assert!(game.redo());
        assert_eq!(game.value(row, col), value);
        assert!(game.can_undo());
    }

    #[test]
    fn new_move_clears_redo_stack() {
        let mut game = game();
        let cells: Vec<(usize, usize)> = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .filter(|&(r, c)| !game.is_given(r, c))
            .take(2)
            .collect();
        let a = cells[0];
        let b = cells[1];

        game.play(a.0, a.1, game.solution[a.0][a.1]);
        game.undo();
        assert!(game.can_redo());

        game.play(b.0, b.1, game.solution[b.0][b.1]);
        assert!(!game.can_redo());
    }

    #[test]
    fn auto_clean_removes_neighbour_notes_and_undo_restores() {
        let mut game = game();
        let target = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .find(|&(r, c)| !game.is_given(r, c))
            .expect("empty cell");
        let value = game.solution[target.0][target.1];

        // Put a note in a row neighbour, then place `value` in `target`.
        let neighbour = (target.0, (target.1 + 1) % SIZE);
        if !game.is_given(neighbour.0, neighbour.1) && game.value(neighbour.0, neighbour.1) == 0 {
            game.toggle_note(neighbour.0, neighbour.1, value);
            assert!(game.has_note(neighbour.0, neighbour.1, value));

            game.play(target.0, target.1, value);
            assert!(
                !game.has_note(neighbour.0, neighbour.1, value),
                "auto-clean should strip the note from the row neighbour"
            );

            game.undo();
            assert!(
                game.has_note(neighbour.0, neighbour.1, value),
                "undo should restore the auto-cleaned note"
            );
            assert_eq!(game.value(target.0, target.1), EMPTY);
        }
    }

    #[test]
    fn hint_fills_correct_digit_and_counts() {
        let mut game = game();
        let target = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .find(|&(r, c)| !game.is_given(r, c))
            .expect("empty cell");

        let filled = game.apply_hint(target.0, target.1);
        assert_eq!(filled, Some(target));
        assert_eq!(
            game.value(target.0, target.1),
            game.solution[target.0][target.1]
        );
        assert_eq!(game.stats().hints_used, 1);

        // A second hint on the same (now filled) cell is a no-op.
        assert_eq!(game.apply_hint(target.0, target.1), None);
    }

    #[test]
    fn save_load_round_trips_full_state() {
        let mut game = game();
        let target = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .find(|&(r, c)| !game.is_given(r, c))
            .expect("empty cell");
        game.play(target.0, target.1, game.solution[target.0][target.1]);

        let encoded = game.encode();
        let restored = Game::decode(&encoded).expect("decodes");

        assert_eq!(
            restored.value(target.0, target.1),
            game.value(target.0, target.1)
        );
        assert_eq!(restored.difficulty(), game.difficulty());
        assert_eq!(restored.seed(), game.seed());
        assert_eq!(restored.encode(), encoded);
    }

    #[test]
    fn strict_mode_flags_wrong_digits() {
        let mut game = game();
        // Find an empty cell that has at least one "safe" wrong digit, so calm
        // mode (which only flags duplicates) stays quiet for that placement.
        let found = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .find_map(|(r, c)| {
                if game.is_given(r, c) {
                    return None;
                }
                let correct = game.solution[r][c];
                (1..=9u8)
                    .find(|&d| d != correct && crate::grid::is_safe(&game.cells, r, c, d))
                    .map(|wrong| (r, c, wrong))
            });
        let (row, col, wrong) = found.expect("a safe wrong digit exists somewhere");

        game.play(row, col, wrong);
        assert!(
            !game.conflicts().contains(&(row, col)),
            "calm mode flags only duplicates"
        );

        let mut strict = game.settings();
        strict.check_against_solution = true;
        game.set_settings(strict);
        assert!(game.conflicts().contains(&(row, col)));
    }

    #[test]
    fn mistakes_counted_only_in_strict_mode() {
        let mut game = game();
        let target = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .find(|&(r, c)| !game.is_given(r, c))
            .expect("empty cell");
        let correct = game.solution[target.0][target.1];
        let wrong = if correct == 1 { 2 } else { 1 };

        game.play(target.0, target.1, wrong);
        assert_eq!(game.stats().mistakes, 0, "calm mode never counts mistakes");

        let mut strict = game.settings();
        strict.check_against_solution = true;
        game.set_settings(strict);
        game.play(target.0, target.1, 0);
        game.play(target.0, target.1, wrong);
        assert_eq!(game.stats().mistakes, 1);
    }

    #[test]
    fn redo_reapplies_auto_cleaned_notes() {
        let mut game = game();
        let target = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .find(|&(r, c)| !game.is_given(r, c))
            .expect("empty cell");
        let value = game.solution[target.0][target.1];
        let neighbour = (target.0, (target.1 + 1) % SIZE);
        if game.is_given(neighbour.0, neighbour.1) || game.value(neighbour.0, neighbour.1) != 0 {
            return; // no usable neighbour for this seed
        }

        game.toggle_note(neighbour.0, neighbour.1, value);
        game.play(target.0, target.1, value);
        assert!(!game.has_note(neighbour.0, neighbour.1, value));

        game.undo();
        assert!(game.has_note(neighbour.0, neighbour.1, value));
        assert_eq!(game.value(target.0, target.1), EMPTY);

        game.redo();
        assert_eq!(game.value(target.0, target.1), value);
        assert!(
            !game.has_note(neighbour.0, neighbour.1, value),
            "redo should re-apply the note cleanup"
        );
    }
}
