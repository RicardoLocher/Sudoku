//! Centralized color palette and egui style setup for the Sudoku UI.
//!
//! All board and widget colors live here so light/dark themes can swap cleanly
//! instead of being scattered `Color32` constants throughout the UI code.

use crate::settings::ThemeMode;
use eframe::egui::{self, Color32, CornerRadius, Shadow};

/// Whether the concrete palette to draw is the light or the dark one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// Resolve a user [`ThemeMode`] to a concrete theme, using `system` for
    /// [`ThemeMode::System`].
    pub fn resolve(mode: ThemeMode, system: Option<egui::Theme>) -> Theme {
        match mode {
            ThemeMode::Light => Theme::Light,
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::System => match system {
                Some(egui::Theme::Dark) => Theme::Dark,
                _ => Theme::Light,
            },
        }
    }
}

/// All colors used to render the app, resolved for a concrete [`Theme`].
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    // Board
    pub board_bg: Color32,
    pub board_bg_alternate: Color32,
    pub line_thin: Color32,
    pub line_thick: Color32,
    pub highlight_rowcol: Color32,
    pub highlight_box: Color32,
    pub highlight_selected: Color32,
    pub highlight_same: Color32,
    pub conflict_bg: Color32,
    pub conflict_marker: Color32,
    pub given_text: Color32,
    pub user_text: Color32,
    pub note_text: Color32,
    pub flash: Color32,
    // Chrome / widgets
    pub panel_bg: Color32,
    pub accent: Color32,
    pub accent_weak: Color32,
    pub positive: Color32,
    pub danger: Color32,
    pub pad_bg: Color32,
    pub pad_text: Color32,
    pub pad_done_text: Color32,
    pub pad_count_text: Color32,
    pub shadow: Color32,
}

impl Palette {
    pub fn of(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self::light(),
            Theme::Dark => Self::dark(),
        }
    }

    pub const fn light() -> Self {
        Self {
            board_bg: Color32::from_rgb(252, 252, 254),
            board_bg_alternate: Color32::from_rgb(243, 245, 249),
            line_thin: Color32::from_rgb(196, 200, 210),
            line_thick: Color32::from_rgb(74, 80, 96),
            highlight_rowcol: Color32::from_rgb(226, 232, 245),
            highlight_box: Color32::from_rgb(210, 221, 244),
            highlight_selected: Color32::from_rgb(180, 202, 238),
            highlight_same: Color32::from_rgb(206, 221, 246),
            conflict_bg: Color32::from_rgb(248, 205, 205),
            conflict_marker: Color32::from_rgb(190, 60, 60),
            given_text: Color32::from_rgb(30, 34, 45),
            user_text: Color32::from_rgb(36, 105, 200),
            note_text: Color32::from_rgb(110, 120, 140),
            flash: Color32::from_rgb(120, 190, 120),
            panel_bg: Color32::from_rgb(244, 246, 250),
            accent: Color32::from_rgb(48, 108, 198),
            accent_weak: Color32::from_rgb(206, 221, 246),
            positive: Color32::from_rgb(30, 150, 70),
            danger: Color32::from_rgb(190, 60, 60),
            pad_bg: Color32::from_rgb(255, 255, 255),
            pad_text: Color32::from_rgb(40, 46, 58),
            pad_done_text: Color32::from_rgb(180, 185, 195),
            pad_count_text: Color32::from_rgb(130, 138, 152),
            shadow: Color32::from_black_alpha(40),
        }
    }

    pub const fn dark() -> Self {
        Self {
            board_bg: Color32::from_rgb(30, 33, 40),
            board_bg_alternate: Color32::from_rgb(35, 38, 46),
            line_thin: Color32::from_rgb(66, 72, 86),
            line_thick: Color32::from_rgb(140, 148, 166),
            highlight_rowcol: Color32::from_rgb(44, 50, 64),
            highlight_box: Color32::from_rgb(52, 60, 80),
            highlight_selected: Color32::from_rgb(62, 84, 128),
            highlight_same: Color32::from_rgb(52, 64, 88),
            conflict_bg: Color32::from_rgb(88, 40, 44),
            conflict_marker: Color32::from_rgb(236, 120, 120),
            given_text: Color32::from_rgb(228, 231, 238),
            user_text: Color32::from_rgb(126, 180, 255),
            note_text: Color32::from_rgb(150, 158, 174),
            flash: Color32::from_rgb(96, 180, 120),
            panel_bg: Color32::from_rgb(24, 26, 32),
            accent: Color32::from_rgb(94, 150, 240),
            accent_weak: Color32::from_rgb(52, 66, 96),
            positive: Color32::from_rgb(96, 200, 130),
            danger: Color32::from_rgb(236, 120, 120),
            pad_bg: Color32::from_rgb(40, 44, 54),
            pad_text: Color32::from_rgb(226, 230, 238),
            pad_done_text: Color32::from_rgb(96, 102, 114),
            pad_count_text: Color32::from_rgb(140, 148, 164),
            shadow: Color32::from_black_alpha(120),
        }
    }
}

/// Build egui [`Visuals`] for the given theme with rounded, softer widgets.
pub fn visuals_for(theme: Theme) -> egui::Visuals {
    let palette = Palette::of(theme);
    let mut visuals = match theme {
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
    };

    let radius = CornerRadius::same(10);

    visuals.panel_fill = palette.panel_bg;
    visuals.window_fill = palette.panel_bg;
    visuals.extreme_bg_color = palette.board_bg;
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.window_shadow = Shadow {
        offset: [0, 6],
        blur: 20,
        spread: 0,
        color: palette.shadow,
    };
    visuals.popup_shadow = visuals.window_shadow;
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.5);
    visuals.hyperlink_color = palette.accent;
    visuals.warn_fg_color = palette.danger;
    if theme == Theme::Light {
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(232, 235, 242);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(232, 235, 242);
    }

    visuals.widgets.inactive.corner_radius = radius;
    visuals.widgets.hovered.corner_radius = radius;
    visuals.widgets.active.corner_radius = radius;

    visuals
}

/// Font size scale used across the UI (logical points, scaled by egui zoom).
pub mod text {
    pub const TITLE: f32 = 22.0;
    pub const BODY: f32 = 16.0;
    pub const SMALL: f32 = 13.0;
    pub const TINY: f32 = 11.0;
}

/// A rounded, subtly shadowed background frame used for the top bar and pad.
pub fn chrome_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .corner_radius(CornerRadius::same(14))
}
