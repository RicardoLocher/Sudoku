use crate::grid::{EMPTY, Grid, SIZE, empty_grid, is_safe};
use crate::rng::SplitMix64;
use crate::solver::count_solutions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    pub fn min_clues(self) -> usize {
        match self {
            Difficulty::Easy => 40,
            Difficulty::Medium => 34,
            Difficulty::Hard => 28,
            Difficulty::Expert => 24,
        }
    }

    pub fn all() -> [Difficulty; 4] {
        [
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Hard,
            Difficulty::Expert,
        ]
    }

    /// Stable one-word name used for persistence.
    pub fn key(self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
            Difficulty::Expert => "expert",
        }
    }

    pub fn from_key(key: &str) -> Option<Difficulty> {
        match key {
            "easy" => Some(Difficulty::Easy),
            "medium" => Some(Difficulty::Medium),
            "hard" => Some(Difficulty::Hard),
            "expert" => Some(Difficulty::Expert),
            _ => None,
        }
    }

    /// Friendly label for menus and the win screen.
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Expert",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puzzle {
    pub grid: Grid,
    pub solution: Grid,
    pub difficulty: Difficulty,
}

fn fill_full(grid: &mut Grid, rng: &mut SplitMix64) -> bool {
    for index in 0..SIZE * SIZE {
        let row = index / SIZE;
        let col = index % SIZE;
        if grid[row][col] != EMPTY {
            continue;
        }

        let mut order = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        rng.shuffle(&mut order);
        for val in order {
            if is_safe(grid, row, col, val) {
                grid[row][col] = val;
                if fill_full(grid, rng) {
                    return true;
                }
                grid[row][col] = EMPTY;
            }
        }
        return false;
    }
    true
}

fn carve(solution: &Grid, difficulty: Difficulty, rng: &mut SplitMix64) -> Grid {
    let mut puzzle = *solution;
    let mut positions: Vec<(usize, usize)> = (0..SIZE)
        .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
        .collect();
    rng.shuffle(&mut positions);

    let target = difficulty.min_clues();
    let mut clues = SIZE * SIZE;

    for (row, col) in positions {
        if clues <= target {
            break;
        }

        let backup = puzzle[row][col];
        puzzle[row][col] = EMPTY;

        let mut probe = puzzle;
        if count_solutions(&mut probe, 2) == 1 {
            clues -= 1;
        } else {
            puzzle[row][col] = backup;
        }
    }

    puzzle
}

pub fn generate(difficulty: Difficulty, seed: u64) -> Puzzle {
    let mut rng = SplitMix64::new(seed);
    let mut solution = empty_grid();
    fill_full(&mut solution, &mut rng);
    let grid = carve(&solution, difficulty, &mut rng);
    Puzzle {
        grid,
        solution,
        difficulty,
    }
}

pub fn clue_count(grid: &Grid) -> usize {
    grid.iter()
        .flat_map(|row| row.iter())
        .filter(|&&v| v != EMPTY)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{is_safe, is_solved};

    #[test]
    fn generated_solution_is_valid() {
        for seed in 0..25 {
            let puzzle = generate(Difficulty::Medium, seed);
            assert!(is_solved(&puzzle.solution));
            for row in 0..SIZE {
                for col in 0..SIZE {
                    let value = puzzle.solution[row][col];
                    let mut cleared = puzzle.solution;
                    cleared[row][col] = EMPTY;
                    assert!(is_safe(&cleared, row, col, value));
                }
            }
        }
    }

    #[test]
    fn generated_puzzles_have_unique_solution() {
        for &difficulty in &Difficulty::all() {
            let puzzle = generate(difficulty, 12345);
            let mut copy = puzzle.grid;
            assert_eq!(count_solutions(&mut copy, 2), 1);
        }
    }

    #[test]
    fn givens_match_solution() {
        let puzzle = generate(Difficulty::Hard, 99);
        for row in 0..SIZE {
            for col in 0..SIZE {
                if puzzle.grid[row][col] != EMPTY {
                    assert_eq!(puzzle.grid[row][col], puzzle.solution[row][col]);
                }
            }
        }
    }

    #[test]
    fn harder_tiers_have_fewer_clues() {
        let easy = clue_count(&generate(Difficulty::Easy, 7).grid);
        let expert = clue_count(&generate(Difficulty::Expert, 7).grid);
        assert!(expert <= easy);
    }

    #[test]
    fn same_seed_is_reproducible() {
        assert_eq!(
            generate(Difficulty::Medium, 2024),
            generate(Difficulty::Medium, 2024)
        );
    }
}
