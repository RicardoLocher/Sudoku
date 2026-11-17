//! Cross-platform persistent storage for settings and the in-progress game.
//!
//! On desktop this is a plain file inside the OS config dir. On Android we use
//! the app's internal storage path obtained from `AndroidApp::internal_data_path()`
//! via the `android-activity` glue. Access is gated behind a tiny trait so the
//! UI code stays platform-agnostic and testable.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

/// A minimal key/value store the UI can read/write synchronously.
pub trait Storage {
    fn read(&self, key: &str) -> Option<String>;
    fn write(&self, key: &str, value: &str);
}

/// In-memory storage, used on platforms/tests without a filesystem path.
#[derive(Default)]
pub struct MemoryStorage {
    map: RefCell<HashMap<String, String>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn read(&self, key: &str) -> Option<String> {
        self.map.borrow().get(key).cloned()
    }

    fn write(&self, key: &str, value: &str) {
        self.map
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }
}

/// File-backed storage rooted at `dir`.
pub struct FileStorage {
    dir: PathBuf,
}

impl FileStorage {
    pub fn new(dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        Self { dir }
    }

    /// Default desktop location (`$XDG_CONFIG_HOME/sudoku` or equivalent).
    #[cfg(all(feature = "desktop", not(target_os = "android")))]
    pub fn default_dir() -> PathBuf {
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(dir).join("sudoku")
        } else if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config").join("sudoku")
        } else {
            PathBuf::from(".").join("sudoku")
        }
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.txt"))
    }
}

impl Storage for FileStorage {
    fn read(&self, key: &str) -> Option<String> {
        std::fs::read_to_string(self.path(key)).ok()
    }

    fn write(&self, key: &str, value: &str) {
        let _ = std::fs::write(self.path(key), value);
    }
}

pub const SETTINGS_KEY: &str = "settings";
pub const SAVE_KEY: &str = "save";
