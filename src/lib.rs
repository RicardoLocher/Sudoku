pub mod game;
pub mod generator;
pub mod grid;
pub mod platform;
pub mod random_seed;
pub mod rng;
pub mod settings;
pub mod solver;
pub mod storage;

#[cfg(feature = "desktop")]
pub mod theme;

#[cfg(feature = "desktop")]
pub mod icons;

#[cfg(feature = "desktop")]
pub mod ui;

pub use game::{Game, Stats};
pub use generator::{Difficulty, Puzzle, clue_count, generate};
pub use grid::{EMPTY, SIZE};
pub use settings::{Settings, ThemeMode};

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
    let storage_dir = app
        .internal_data_path()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let storage: Box<dyn storage::Storage> =
        Box::new(storage::FileStorage::new(storage_dir.join("sudoku")));

    // Keep a handle so the UI can query safe-area insets (notches / gesture bar)
    // every frame; `AndroidApp` shares state internally.
    let insets_provider = platform::InsetsProvider::new(app.clone());

    let mut options = native_options();
    options.android_app = Some(app);
    let seed = random_seed::random_seed();
    let _ = eframe::run_native(
        "Sudoku",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(ui::SudokuApp::with_storage(
                seed,
                storage,
                insets_provider,
            )))
        }),
    );
}
