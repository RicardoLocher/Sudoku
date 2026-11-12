use crate::generator::Puzzle;
use crate::grid::{EMPTY, Grid, SIZE};

#[derive(Debug, Clone, Copy)]
enum Move {
    Value {
        row: usize,
        col: usize,
        previous: u8,
    },
    Notes {
        row: usize,
        col: usize,
        previous: u16,
    },
}

#[derive(Debug, Clone)]
pub struct Game {
    solution: Grid,
    givens: [[bool; SIZE]; SIZE],
    cells: Grid,
    notes: [[u16; SIZE]; SIZE],
    history: Vec<Move>,
}

impl Game {
    pub fn new(puzzle: Puzzle) -> Self {
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
        }
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

    pub fn set_value(&mut self, row: usize, col: usize, value: u8) {
        if self.is_given(row, col) {
            return;
        }

        self.history.push(Move::Value {
            row,
            col,
            previous: self.cells[row][col],
        });

        if self.cells[row][col] == value {
            self.cells[row][col] = EMPTY;
        } else {
            self.cells[row][col] = value;
            self.notes[row][col] = 0;
        }
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

        let bit = 1u16 << digit;
        self.notes[row][col] ^= bit;
    }

    pub fn has_note(&self, row: usize, col: usize, digit: u8) -> bool {
        self.notes[row][col] & (1u16 << digit) != 0
    }

    pub fn undo(&mut self) -> bool {
        match self.history.pop() {
            Some(Move::Value { row, col, previous }) => {
                self.cells[row][col] = previous;
                true
            }
            Some(Move::Notes { row, col, previous }) => {
                self.notes[row][col] = previous;
                true
            }
            None => false,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.cells == self.solution
    }

    pub fn digit_count(&self, digit: u8) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&value| value == digit)
            .count()
    }

    pub fn is_digit_complete(&self, digit: u8) -> bool {
        self.digit_count(digit) >= SIZE
    }

    pub fn conflicts(&self) -> Vec<(usize, usize)> {
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
    }
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
        game.set_value(row, col, 0);
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
        game.set_value(row, col, value);
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
                    game.set_value(row, col, digit);
                }
            }
        }

        assert_eq!(game.digit_count(digit), 9);
        assert!(game.is_digit_complete(digit));
    }
}
