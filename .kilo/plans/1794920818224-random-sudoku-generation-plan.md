# Plan: Random Sudoku Game Generation

## Problem
- App always starts with the same puzzle (hardcoded seed=1 in `main.rs:11`)
- Saved games restore the exact same puzzle/seed
- "New Game" only increments seed by 1 (`wrapping_add(1)`), making puzzles predictable
- User expects truly random puzzles on startup and when clicking "New Game"

## Root Cause
The `SplitMix64` PRNG in `generator.rs` is deterministic - same seed = same puzzle. The seed is:
1. Hardcoded to `1` in `main.rs`
2. Persisted in save files and restored on launch
3. Only incremented by 1 for "New Game"

## Solution
Add cryptographically secure randomness via `getrandom` crate and use it for:
1. **Initial seed** on first launch (no save file)
2. **New Game** button - generate fresh random seed
3. Keep **Daily Puzzle** as-is (date-based seed is intentional)

## Implementation Steps

### 1. Add `getrandom` dependency
- Add `getrandom = "0.2"` to `Cargo.toml` (works on desktop + Android)

### 2. Create random seed utility (`src/random_seed.rs`)
```rust
use getrandom::getrandom;

pub fn random_seed() -> u64 {
    let mut bytes = [0u8; 8];
    getrandom(&mut bytes).expect("getrandom failed");
    u64::from_le_bytes(bytes)
}
```

### 3. Update `main.rs`
- Generate random seed on startup instead of hardcoded `1`
- Pass random seed to `SudokuApp::with_storage`

### 4. Update `ui.rs` - `SudokuApp::with_storage`
- When no save exists: use the passed random seed (already works)
- When save exists but game is complete: generate new random seed for fresh game
- Keep current behavior for in-progress games (resume)

### 5. Update `ui.rs` - `new_game()`
- Replace `self.seed.wrapping_add(1)` with `random_seed()`
- This ensures each "New Game" gets a truly random puzzle

### 6. Update `ui.rs` - `new_daily()`
- Keep existing date-based logic (this is correct for daily puzzles)

## Affected Files
- `Cargo.toml` - add getrandom dependency
- `src/random_seed.rs` - new file for random seed generation
- `src/main.rs` - use random seed on startup
- `src/ui.rs` - use random seed for new games

## Validation
- Build and run: verify different puzzle on each fresh launch
- Click "New Game" multiple times: verify different puzzles each time
- Daily puzzle: verify same puzzle for same calendar day
- Resume saved game: verify it restores exact state
- Run existing tests: `cargo test`