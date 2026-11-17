pub const SIZE: usize = 9;
pub const BOX: usize = 3;
pub const EMPTY: u8 = 0;
pub const DIGITS: [u8; SIZE] = [1, 2, 3, 4, 5, 6, 7, 8, 9];

pub type Grid = [[u8; SIZE]; SIZE];

pub fn empty_grid() -> Grid {
    [[EMPTY; SIZE]; SIZE]
}

pub fn is_safe(grid: &Grid, row: usize, col: usize, val: u8) -> bool {
    if grid[row].contains(&val) {
        return false;
    }
    if (0..SIZE).any(|i| grid[i][col] == val) {
        return false;
    }

    let box_row = row / BOX * BOX;
    let box_col = col / BOX * BOX;
    if grid[box_row..box_row + BOX]
        .iter()
        .any(|r| r[box_col..box_col + BOX].contains(&val))
    {
        return false;
    }

    true
}

pub fn candidates(grid: &Grid, row: usize, col: usize) -> Vec<u8> {
    DIGITS
        .into_iter()
        .filter(|&v| is_safe(grid, row, col, v))
        .collect()
}

pub fn is_solved(grid: &Grid) -> bool {
    grid.iter().all(|row| row.iter().all(|&v| v != EMPTY))
}

/// Iterate the row, column and 3×3 box peers of `(row, col)`, excluding the
/// cell itself (which is yielded first).
pub fn peers(row: usize, col: usize) -> impl Iterator<Item = (usize, usize)> {
    let box_row = row / BOX * BOX;
    let box_col = col / BOX * BOX;
    let mut cells = Vec::with_capacity(21);
    cells.push((row, col));
    for i in 0..SIZE {
        if i != col {
            cells.push((row, i));
        }
        if i != row {
            cells.push((i, col));
        }
    }
    for r in box_row..box_row + BOX {
        for c in box_col..box_col + BOX {
            if r != row && c != col {
                cells.push((r, c));
            }
        }
    }
    cells.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicates_in_row_col_box() {
        let mut grid = empty_grid();
        grid[0][0] = 5;
        assert!(!is_safe(&grid, 0, 8, 5));
        assert!(!is_safe(&grid, 8, 0, 5));
        assert!(!is_safe(&grid, 1, 1, 5));
        assert!(is_safe(&grid, 4, 4, 5));
    }

    #[test]
    fn candidates_shrink_with_neighbours() {
        let mut grid = empty_grid();
        grid[0][1] = 1;
        let cands = candidates(&grid, 0, 0);
        assert!(!cands.contains(&1));
        assert_eq!(cands.len(), 8);
    }
}
