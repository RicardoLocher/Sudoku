use crate::grid::{EMPTY, Grid, SIZE, is_safe};

fn find_mrv(grid: &Grid) -> Option<(usize, usize, Vec<u8>)> {
    let mut best: Option<(usize, usize, Vec<u8>)> = None;

    for row in 0..SIZE {
        for col in 0..SIZE {
            if grid[row][col] != EMPTY {
                continue;
            }
            let cands: Vec<u8> = (1..=9u8).filter(|&v| is_safe(grid, row, col, v)).collect();
            if cands.is_empty() {
                return Some((row, col, cands));
            }
            match &best {
                Some((_, _, current)) if current.len() <= cands.len() => {}
                _ => best = Some((row, col, cands)),
            }
        }
    }

    best
}

pub fn solve(grid: &mut Grid) -> bool {
    let (row, col, cands) = match find_mrv(grid) {
        None => return true,
        Some(cell) => cell,
    };

    for val in cands {
        grid[row][col] = val;
        if solve(grid) {
            return true;
        }
        grid[row][col] = EMPTY;
    }

    false
}

pub fn count_solutions(grid: &mut Grid, limit: u8) -> u8 {
    let (row, col, cands) = match find_mrv(grid) {
        None => return 1,
        Some(cell) => cell,
    };

    let mut count = 0u8;
    for val in cands {
        grid[row][col] = val;
        count += count_solutions(grid, limit - count);
        grid[row][col] = EMPTY;
        if count >= limit {
            break;
        }
    }

    count
}

pub fn has_unique_solution(grid: &mut Grid) -> bool {
    count_solutions(grid, 2) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn puzzle() -> Grid {
        [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ]
    }

    #[test]
    fn solves_known_puzzle() {
        let mut grid = puzzle();
        assert!(solve(&mut grid));
        assert_eq!(
            grid,
            [
                [5, 3, 4, 6, 7, 8, 9, 1, 2],
                [6, 7, 2, 1, 9, 5, 3, 4, 8],
                [1, 9, 8, 3, 4, 2, 5, 6, 7],
                [8, 5, 9, 7, 6, 1, 4, 2, 3],
                [4, 2, 6, 8, 5, 3, 7, 9, 1],
                [7, 1, 3, 9, 2, 4, 8, 5, 6],
                [9, 6, 1, 5, 3, 7, 2, 8, 4],
                [2, 8, 7, 4, 1, 9, 6, 3, 5],
                [3, 4, 5, 2, 8, 6, 1, 7, 9],
            ]
        );
    }

    #[test]
    fn detects_unique_solution() {
        let mut grid = puzzle();
        assert!(has_unique_solution(&mut grid));
    }

    #[test]
    fn detects_ambiguous_puzzle() {
        let mut grid = crate::grid::empty_grid();
        assert!(!has_unique_solution(&mut grid));
    }
}
