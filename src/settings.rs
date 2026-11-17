//! User-facing settings plus the tiny key/value text format used to persist
//! them (and the in-progress game) without pulling in a serialization crate.

/// Which visual theme the app should use (UI resolves this to light/dark).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn all() -> [ThemeMode; 3] {
        [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark]
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub theme: ThemeMode,
    /// `true` = strict mode (flag digits that differ from the known solution);
    /// `false` = calm mode (only flag duplicates in row/col/box).
    pub check_against_solution: bool,
    /// Remove a placed digit from the notes of its row/col/box neighbours.
    pub auto_remove_notes: bool,
    pub highlight_same_digit: bool,
    pub highlight_row_col_box: bool,
    /// Mirror the number pad to the left for left-handed players.
    pub left_handed: bool,
    /// Dim and lock the row/col/box highlights behind a pause overlay.
    pub auto_pause: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            check_against_solution: false,
            auto_remove_notes: true,
            highlight_same_digit: true,
            highlight_row_col_box: true,
            left_handed: false,
            auto_pause: true,
        }
    }
}

impl Settings {
    /// Merge persisted `key = value` lines into the defaults.
    pub fn from_text(text: &str) -> Self {
        let mut settings = Settings::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "theme" => {
                    settings.theme = match value {
                        "light" => ThemeMode::Light,
                        "dark" => ThemeMode::Dark,
                        _ => ThemeMode::System,
                    }
                }
                "check_solution" => settings.check_against_solution = parse_bool(value),
                "auto_remove_notes" => settings.auto_remove_notes = parse_bool(value),
                "highlight_same" => settings.highlight_same_digit = parse_bool(value),
                "highlight_rowcol" => settings.highlight_row_col_box = parse_bool(value),
                "left_handed" => settings.left_handed = parse_bool(value),
                "auto_pause" => settings.auto_pause = parse_bool(value),
                _ => {}
            }
        }
        settings
    }

    pub fn to_text(self) -> String {
        format!(
            "theme={}\ncheck_solution={}\nauto_remove_notes={}\nhighlight_same={}\nhighlight_rowcol={}\nleft_handed={}\nauto_pause={}\n",
            match self.theme {
                ThemeMode::System => "system",
                ThemeMode::Light => "light",
                ThemeMode::Dark => "dark",
            },
            self.check_against_solution as u8,
            self.auto_remove_notes as u8,
            self.highlight_same_digit as u8,
            self.highlight_row_col_box as u8,
            self.left_handed as u8,
            self.auto_pause as u8,
        )
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(value, "1" | "true" | "yes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let settings = Settings {
            theme: ThemeMode::Dark,
            check_against_solution: true,
            left_handed: true,
            auto_remove_notes: false,
            ..Default::default()
        };
        let restored = Settings::from_text(&settings.to_text());
        assert_eq!(restored, settings);
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let restored = Settings::from_text("bogus=1\ntheme=light\n");
        assert_eq!(restored.theme, ThemeMode::Light);
        assert_eq!(
            restored.auto_remove_notes,
            Settings::default().auto_remove_notes
        );
    }
}
