pub mod game;
pub mod generator;
pub mod grid;
pub mod rng;
pub mod solver;

#[cfg(feature = "desktop")]
pub mod ui;

pub use game::Game;
pub use generator::{Difficulty, Puzzle, clue_count, generate};
pub use grid::{EMPTY, SIZE};

#[cfg(feature = "desktop")]
pub fn native_options() -> eframe::NativeOptions {
    use eframe::egui;

    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([460.0, 760.0])
            .with_min_inner_size([340.0, 560.0])
            .with_title("Sudoku"),
        ..Default::default()
    }
}

/// Entry point used by the Android `NativeActivity` glue.
///
/// `android-activity` exports `ANativeActivity_onCreate` and calls this symbol.
#[cfg(all(feature = "desktop", target_os = "android"))]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    let mut options = native_options();
    options.android_app = Some(app);
    let _ = eframe::run_native(
        "Sudoku",
        options,
        Box::new(|_cc| Ok(Box::new(ui::SudokuApp::new(1)))),
    );
}
