#[cfg(all(feature = "desktop", not(target_os = "android")))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Sudoku",
        sudoku::native_options(),
        Box::new(|_cc| Ok(Box::new(sudoku::ui::SudokuApp::new(1)))),
    )
}

#[cfg(all(feature = "desktop", target_os = "android"))]
fn main() {}

#[cfg(not(feature = "desktop"))]
fn main() {
    println!("GUI disabled. Build with the default `desktop` feature to run the app.");
}
