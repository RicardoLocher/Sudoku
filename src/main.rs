#[cfg(all(feature = "desktop", not(target_os = "android")))]
fn main() -> eframe::Result<()> {
    let storage: Box<dyn sudoku::storage::Storage> = Box::new(sudoku::storage::FileStorage::new(
        sudoku::storage::FileStorage::default_dir(),
    ));
    eframe::run_native(
        "Sudoku",
        sudoku::native_options(),
        Box::new(move |_cc| {
            Ok(Box::new(sudoku::ui::SudokuApp::with_storage(
                1,
                storage,
                sudoku::platform::InsetsProvider::new(),
            )))
        }),
    )
}

#[cfg(all(feature = "desktop", target_os = "android"))]
fn main() {}

#[cfg(not(feature = "desktop"))]
fn main() {
    println!("GUI disabled. Build with the default `desktop` feature to run the app.");
}
