//! The egui UI: a responsive three-zone layout (top bar / board / number pad)
//! with overlays for pause, settings, new-game confirmation and the win screen.
//!
//! State lives in [`SudokuApp`] so it survives screen rotation. The game and
//! settings are autosaved to a [`Storage`] backend and restored on launch.

use crate::generator::{Difficulty, generate};
use crate::icons::{Icon, IconStore};
use crate::platform::{Insets, InsetsProvider};
use crate::random_seed::random_seed;
use crate::settings::{Settings, ThemeMode};
use crate::storage::{SAVE_KEY, SETTINGS_KEY, Storage};
use crate::theme::{Palette, Theme, chrome_frame, text};
use crate::{Game, SIZE};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Rect, RichText, Sense, Stroke, Vec2,
};

/// Logical dp target for a comfortable touch target.
const TAP_MIN: f32 = 48.0;

/// Which overlay (if any) is currently shown on top of the game.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Dialog {
    #[default]
    None,
    Settings,
    NewGame,
    Win,
}

/// The application state. Everything the game needs lives here, so it survives
/// screen rotation and is fully persisted to [`Storage`] for save/resume.
pub struct SudokuApp {
    game: Game,
    difficulty: Difficulty,
    seed: u64,
    selected: Option<(usize, usize)>,
    note_mode: bool,
    settings: Settings,
    storage: Box<dyn Storage>,

    // Timer
    elapsed: f64,
    /// The clock has started (first move made).
    started: bool,
    /// The player paused the game; the board is hidden and the clock stops.
    paused: bool,
    /// Monotonic baseline (seconds) of the last clock update.
    clock_anchor: f64,

    // Dialog / transient UI state
    dialog: Dialog,
    /// A difficulty/mode change attempted while a game is in progress.
    pending_start: Option<(Difficulty, bool)>,

    // Feedback
    flash_digit: Option<u8>,
    flash_time: f64,
    accent_time: f64,

    /// Last frame's focus/viewport visibility, for auto-pause detection.
    was_visible: bool,
    settings_dirty: bool,

    /// Safe-area inset provider (Android notches / system bars).
    insets_provider: InsetsProvider,
    /// Latest resolved insets in logical points.
    insets: Insets,
    /// Lazily-uploaded UI icon textures.
    icons: IconStore,
}

impl SudokuApp {
    #[cfg(not(all(feature = "desktop", target_os = "android")))]
    pub fn new(seed: u64) -> Self {
        Self::with_storage(
            seed,
            Box::new(crate::storage::MemoryStorage::new()),
            InsetsProvider::new(),
        )
    }

    /// Build the app, restoring settings and any in-progress game from `storage`.
    ///
    /// `insets_provider` supplies safe-area insets (Android notches / system
    /// bars); it is a no-op [`InsetsProvider`] on desktop.
    pub fn with_storage(
        seed: u64,
        storage: Box<dyn Storage>,
        insets_provider: InsetsProvider,
    ) -> Self {
        let settings = storage
            .read(SETTINGS_KEY)
            .map(|text| Settings::from_text(&text))
            .unwrap_or_default();

        let mut elapsed = 0.0;
        let (game, difficulty, seed) = match storage.read(SAVE_KEY) {
            Some(text) => {
                let (game_text, loaded_elapsed) = split_save_header(&text);
                elapsed = loaded_elapsed;
                match Game::decode(&game_text) {
                    Some(mut game) => {
                        let difficulty = game.difficulty();
                        let seed = game.seed();
                        game.set_settings(settings);
                        // If the saved game is complete, start a fresh random game
                        if game.is_complete() {
                            let new_seed = random_seed();
                            let mut new_game =
                                Game::with_settings(generate(difficulty, new_seed), settings);
                            new_game.set_seed(new_seed);
                            (new_game, difficulty, new_seed)
                        } else {
                            (game, difficulty, seed)
                        }
                    }
                    None => {
                        let difficulty = Difficulty::Medium;
                        let mut game = Game::with_settings(generate(difficulty, seed), settings);
                        game.set_seed(seed);
                        (game, difficulty, seed)
                    }
                }
            }
            None => {
                let difficulty = Difficulty::Medium;
                let mut game = Game::with_settings(generate(difficulty, seed), settings);
                game.set_seed(seed);
                (game, difficulty, seed)
            }
        };

        let started = elapsed > 0.0 && !game.is_complete();
        let app = Self {
            game,
            difficulty,
            seed,
            selected: None,
            note_mode: false,
            settings,
            storage,
            elapsed,
            started,
            paused: false,
            clock_anchor: 0.0,
            dialog: Dialog::None,
            pending_start: None,
            flash_digit: None,
            flash_time: 0.0,
            accent_time: 0.0,
            was_visible: true,
            settings_dirty: false,
            insets_provider,
            insets: Insets::ZERO,
            icons: IconStore::new(),
        };
        // Ensure a save exists immediately so closing at any point resumes.
        app.save_now();
        app
    }

    fn new_game(&mut self, difficulty: Difficulty) {
        self.seed = random_seed();
        self.difficulty = difficulty;
        let mut game = Game::with_settings(generate(difficulty, self.seed), self.settings);
        game.set_seed(self.seed);
        self.game = game;
        self.selected = None;
        self.note_mode = false;
        self.elapsed = 0.0;
        self.started = false;
        self.paused = false;
        self.dialog = Dialog::None;
        self.flash_digit = None;
        self.save_now();
    }

    fn daily_seed() -> u64 {
        // Days since the Unix epoch, so the same calendar day shares a puzzle.
        let secs = now_seconds_unix();
        (secs / 86_400.0) as u64
    }

    fn new_daily(&mut self) {
        self.seed = Self::daily_seed().wrapping_mul(2_654_435_761);
        let difficulty = Difficulty::Medium;
        self.difficulty = difficulty;
        let mut game = Game::with_settings(generate(difficulty, self.seed), self.settings);
        game.set_seed(self.seed);
        self.game = game;
        self.selected = None;
        self.note_mode = false;
        self.elapsed = 0.0;
        self.started = false;
        self.paused = false;
        self.dialog = Dialog::None;
        self.save_now();
    }

    fn set_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.game.set_settings(settings);
        self.settings_dirty = true;
    }

    /// The clock is actively counting: game started and not paused.
    fn running(&self) -> bool {
        self.started && !self.paused
    }

    fn save_now(&self) {
        self.storage.write(SETTINGS_KEY, &self.settings.to_text());
        // Persist elapsed time as a header line so resume continues the clock.
        let save = format!("t={:.1}\n{}", self.elapsed, self.game.encode());
        self.storage.write(SAVE_KEY, &save);
    }

    fn resolution(&self, system: Option<egui::Theme>) -> Theme {
        Theme::resolve(self.settings.theme, system)
    }

    fn select(&mut self, row: usize, col: usize) {
        self.selected = Some((row, col));
    }

    /// Whether any progress has been made since the puzzle was loaded.
    fn in_progress(&self) -> bool {
        self.elapsed > 0.0 || self.game.can_undo()
    }

    fn input_digit(&mut self, digit: u8) {
        let Some((row, col)) = self.selected else {
            return;
        };
        if self.game.is_given(row, col) {
            return;
        }

        if self.note_mode {
            self.game.toggle_note(row, col, digit);
        } else {
            self.start_clock();
            self.game.play(row, col, digit);
            self.after_move();
        }
    }

    fn after_move(&mut self) {
        if let Some(digit) = self.game.take_flash() {
            self.flash_digit = Some(digit);
            self.flash_time = 0.0;
        }
        if self.game.is_complete() {
            self.started = false;
            self.paused = false;
            self.dialog = Dialog::Win;
        }
        self.save_now();
    }

    fn erase(&mut self) {
        if let Some((row, col)) = self.selected
            && !self.game.is_given(row, col)
        {
            self.game.play(row, col, 0);
            self.after_move();
        }
    }

    fn hint(&mut self) {
        let target = self
            .selected
            .filter(|&(r, c)| !self.game.is_given(r, c) && self.game.value(r, c) == 0);
        let target = target.or_else(|| self.game.first_empty());
        if let Some((row, col)) = target {
            self.start_clock();
            self.game.apply_hint(row, col);
            self.selected = Some((row, col));
            self.after_move();
        }
    }

    /// Begin (or resume) the clock. Safe to call repeatedly.
    fn start_clock(&mut self) {
        self.started = true;
        self.paused = false;
        self.clock_anchor = now_seconds();
    }

    fn toggle_pause(&mut self) {
        if self.game.is_complete() {
            return;
        }
        if self.paused {
            self.paused = false;
            self.clock_anchor = now_seconds();
        } else {
            // Pausing before the first move simply keeps the board hidden.
            self.paused = true;
        }
        self.save_now();
    }

    /// Advance the clock and handle auto-pause when unfocused/hidden.
    fn tick(&mut self, ctx: &egui::Context) {
        let now = now_seconds();
        let visible =
            !ctx.input(|i| i.viewport().minimized.unwrap_or(false)) && ctx.input(|i| i.focused);
        if self.was_visible && !visible && self.started && self.settings.auto_pause {
            self.paused = true;
        }
        self.was_visible = visible;

        if self.running() {
            let delta = (now - self.clock_anchor).clamp(0.0, 5.0);
            self.elapsed += delta;
        }
        self.clock_anchor = now;

        self.accent_time = now;
        if self.flash_digit.is_some() {
            self.flash_time += 1.0 / 60.0;
        }

        if self.running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
            ctx.request_repaint();
        } else if self.flash_digit.is_some() {
            ctx.request_repaint();
        }

        if self.settings_dirty {
            self.storage.write(SETTINGS_KEY, &self.settings.to_text());
            self.settings_dirty = false;
        }
    }

    // ---------------------------------------------------------------- board

    fn cell_rect(grid_rect: Rect, cell: f32, row: usize, col: usize) -> Rect {
        let min = grid_rect.min + Vec2::new(col as f32 * cell, row as f32 * cell);
        Rect::from_min_size(min, Vec2::splat(cell))
    }

    fn draw_board(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let available = ui.available_size();
        let size = available.x.min(available.y).clamp(120.0, 620.0);
        let (grid_rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
        let cell = size / SIZE as f32;
        let painter = ui.painter_at(grid_rect);
        let radius = CornerRadius::same((cell * 0.18) as u8);

        painter.rect_filled(grid_rect, CornerRadius::same(12), palette.board_bg);

        // Alternating 3×3 box shading for a cleaner look.
        for br in 0..3 {
            for bc in 0..3 {
                if (br + bc) % 2 == 0 {
                    let min =
                        grid_rect.min + Vec2::new(bc as f32 * 3.0 * cell, br as f32 * 3.0 * cell);
                    painter.rect_filled(
                        Rect::from_min_size(min, Vec2::splat(cell * 3.0)),
                        radius,
                        palette.board_bg_alternate,
                    );
                }
            }
        }

        let conflicts = self.game.conflicts();

        if let Some((sel_row, sel_col)) = self.selected {
            if self.settings.highlight_row_col_box {
                for i in 0..SIZE {
                    painter.rect_filled(
                        Self::cell_rect(grid_rect, cell, sel_row, i),
                        radius,
                        palette.highlight_rowcol,
                    );
                    painter.rect_filled(
                        Self::cell_rect(grid_rect, cell, i, sel_col),
                        radius,
                        palette.highlight_rowcol,
                    );
                }
                let box_row = sel_row / 3 * 3;
                let box_col = sel_col / 3 * 3;
                for r in box_row..box_row + 3 {
                    for c in box_col..box_col + 3 {
                        painter.rect_filled(
                            Self::cell_rect(grid_rect, cell, r, c),
                            radius,
                            palette.highlight_box,
                        );
                    }
                }
            }

            let selected_value = self.game.value(sel_row, sel_col);
            if self.settings.highlight_same_digit && selected_value != 0 {
                for r in 0..SIZE {
                    for c in 0..SIZE {
                        if self.game.value(r, c) == selected_value {
                            painter.rect_filled(
                                Self::cell_rect(grid_rect, cell, r, c),
                                radius,
                                palette.highlight_same,
                            );
                        }
                    }
                }
            }

            // Animated selection ring.
            let pulse = 0.5 + 0.5 * (self.accent_time * 2.0).sin() as f32 * 0.15;
            let sel = Self::cell_rect(grid_rect, cell, sel_row, sel_col);
            painter.rect_filled(
                sel,
                radius,
                palette.highlight_selected.gamma_multiply(pulse),
            );
            painter.rect_stroke(
                sel.shrink(1.0),
                radius,
                Stroke::new(2.0, palette.accent),
                egui::StrokeKind::Inside,
            );
        }

        for &(r, c) in &conflicts {
            let rect = Self::cell_rect(grid_rect, cell, r, c);
            painter.rect_filled(rect, radius, palette.conflict_bg);
        }

        let big_font = FontId::proportional(cell * 0.62);
        let small_font = FontId::proportional(cell * 0.24);

        for row in 0..SIZE {
            for col in 0..SIZE {
                let rect = Self::cell_rect(grid_rect, cell, row, col);
                let value = self.game.value(row, col);

                if value != 0 {
                    let color = if self.game.is_given(row, col) {
                        palette.given_text
                    } else {
                        palette.user_text
                    };
                    painter.text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        value.to_string(),
                        big_font.clone(),
                        color,
                    );
                } else {
                    for digit in 1..=9u8 {
                        if !self.game.has_note(row, col, digit) {
                            continue;
                        }
                        let mini = Rect::from_min_size(
                            rect.min
                                + Vec2::new(
                                    ((digit - 1) % 3) as f32 * cell / 3.0,
                                    ((digit - 1) / 3) as f32 * cell / 3.0,
                                ),
                            Vec2::splat(cell / 3.0),
                        );
                        painter.text(
                            mini.center(),
                            Align2::CENTER_CENTER,
                            digit.to_string(),
                            small_font.clone(),
                            palette.note_text,
                        );
                    }
                }
            }
        }

        // Conflict cue that doesn't rely on color alone: a small triangle badge.
        for &(r, c) in &conflicts {
            let rect = Self::cell_rect(grid_rect, cell, r, c);
            let corner = rect.right_top() - Vec2::new(cell * 0.28, -cell * 0.02);
            let tri = vec![
                corner + Vec2::new(0.0, 0.0),
                corner + Vec2::new(-cell * 0.22, 0.0),
                corner + Vec2::new(0.0, cell * 0.22),
            ];
            painter.add(egui::Shape::convex_polygon(
                tri,
                palette.conflict_marker,
                Stroke::NONE,
            ));
        }

        // Completed-digit diagonal flash.
        if let Some(digit) = self.flash_digit {
            let t = self.flash_time;
            if t > 1.2 {
                self.flash_digit = None;
            } else {
                let alpha = (1.0 - t / 1.2).clamp(0.0, 1.0) as f32;
                let color = palette.flash.gamma_multiply(alpha);
                for i in 0..SIZE {
                    painter.rect_filled(Self::cell_rect(grid_rect, cell, i, i), radius, color);
                    painter.text(
                        Self::cell_rect(grid_rect, cell, i, i).center(),
                        Align2::CENTER_CENTER,
                        digit.to_string(),
                        big_font.clone(),
                        palette.flash.gamma_multiply(alpha * 0.9),
                    );
                }
            }
        }

        for i in 0..=SIZE {
            let thick = i % 3 == 0;
            let stroke = Stroke::new(
                if thick { 2.5 } else { 1.0 },
                if thick {
                    palette.line_thick
                } else {
                    palette.line_thin
                },
            );
            let x = grid_rect.min.x + i as f32 * cell;
            let y = grid_rect.min.y + i as f32 * cell;
            painter.line_segment(
                [
                    egui::pos2(x, grid_rect.min.y),
                    egui::pos2(x, grid_rect.max.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(grid_rect.min.x, y),
                    egui::pos2(grid_rect.max.x, y),
                ],
                stroke,
            );
        }

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let col = ((pos.x - grid_rect.min.x) / cell).floor() as isize;
            let row = ((pos.y - grid_rect.min.y) / cell).floor() as isize;
            if (0..SIZE as isize).contains(&row) && (0..SIZE as isize).contains(&col) {
                self.select(row as usize, col as usize);
            }
        }
    }

    // ------------------------------------------------------------ top bar

    fn draw_top_bar(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            let mut theme = self.settings.theme;
            let ctx = ui.ctx().clone();
            let menu_icon = self
                .icons
                .image(&ctx, Icon::Menu, text::TITLE, palette.given_text);
            ui.menu_button(menu_icon, |ui| {
                ui.set_min_width(200.0);
                ui.menu_button("New game", |ui| {
                    for difficulty in Difficulty::all() {
                        if ui.button(difficulty.label()).clicked() {
                            ui.close();
                            self.request_start(difficulty, false);
                        }
                    }
                });
                if ui.button("Daily puzzle").clicked() {
                    ui.close();
                    self.new_daily();
                }
                ui.separator();
                if ui.button("Settings").clicked() {
                    ui.close();
                    self.dialog = Dialog::Settings;
                }
                if ui.button("Pause").clicked() {
                    ui.close();
                    self.toggle_pause();
                }
                ui.separator();
                for mode in ThemeMode::all() {
                    let selected = theme == mode;
                    if ui
                        .selectable_label(selected, format!("Theme: {}", mode.label()))
                        .clicked()
                    {
                        theme = mode;
                    }
                }
                if theme != self.settings.theme {
                    let mut settings = self.settings;
                    settings.theme = theme;
                    self.set_settings(settings);
                }
            });

            ui.add_space(4.0);
            let pause_icon = if self.running() {
                Icon::Pause
            } else {
                Icon::Play
            };
            if icon_button(
                ui,
                &mut self.icons,
                pause_icon,
                text::TITLE,
                palette.given_text,
                None,
            )
            .clicked()
            {
                self.toggle_pause();
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let chip = egui::Frame::new()
                    .fill(palette.accent_weak)
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(10, 6));
                chip.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        let img =
                            self.icons
                                .image(&ctx, Icon::Timer, text::BODY, palette.given_text);
                        ui.add(img);
                        ui.label(
                            RichText::new(format_time(self.elapsed))
                                .color(palette.given_text)
                                .monospace()
                                .size(text::BODY),
                        );
                    });
                });
                ui.add_space(6.0);
                let chip2 = egui::Frame::new()
                    .fill(palette.accent_weak)
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(10, 6));
                chip2.show(ui, |ui| {
                    ui.label(
                        RichText::new(self.difficulty.label())
                            .color(palette.given_text)
                            .size(text::BODY),
                    );
                });
            });
        });

        // Thin progress bar: how much of the board is filled.
        let filled = self.game.filled_count() as f32 / (SIZE * SIZE) as f32;
        let (bar_rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 4.0), Sense::hover());
        let painter = ui.painter_at(bar_rect);
        painter.rect_filled(bar_rect, CornerRadius::same(2), palette.accent_weak);
        if filled > 0.0 {
            let mut fill = bar_rect;
            fill.set_width(bar_rect.width() * filled);
            painter.rect_filled(fill, CornerRadius::same(2), palette.accent);
        }
    }

    fn request_start(&mut self, difficulty: Difficulty, daily: bool) {
        if self.in_progress() && !self.game.is_complete() {
            self.pending_start = Some((difficulty, daily));
            self.dialog = Dialog::NewGame;
        } else if daily {
            self.new_daily();
        } else {
            self.new_game(difficulty);
        }
    }

    // ---------------------------------------------------------- number pad

    fn draw_number_pad(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        // Wide layout (tablet/landscape): a single row of 9 + erase.
        // Narrow (phone portrait): two rows of 5 with the erase on the last.
        let wide = ui.available_width() >= 460.0;
        let per_row = if wide { 10 } else { 5 };
        let spacing = 6.0;
        let full = ui.available_width();
        let size = ((full - spacing * (per_row as f32 - 1.0)) / per_row as f32).min(TAP_MIN + 8.0);

        let digits: Vec<u8> = (1..=9u8).collect();
        let chunks: Vec<Vec<u8>> = if wide {
            vec![digits.clone()]
        } else {
            vec![digits[0..5].to_vec(), digits[5..9].to_vec()]
        };

        // Left-handed mode mirrors the pad: erase first (left), actions reversed.
        let left_handed = self.settings.left_handed;
        for (i, chunk) in chunks.iter().enumerate() {
            if i > 0 {
                ui.add_space(spacing);
            }
            let with_erase = wide || i == chunks.len() - 1;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                let mut digits: Vec<u8> = chunk.clone();
                if left_handed {
                    digits.reverse();
                }
                if left_handed && with_erase {
                    self.draw_pad_button(ui, palette, size, None);
                }
                for &digit in &digits {
                    self.draw_pad_button(ui, palette, size, Some(digit));
                }
                if !left_handed && with_erase {
                    self.draw_pad_button(ui, palette, size, None);
                }
            });
        }

        ui.add_space(8.0);

        // Action row: undo / redo / notes / hint.
        let actions = ui.available_width();
        let ab = (actions / 4.0) - 6.0;
        ui.with_layout(
            if left_handed {
                Layout::right_to_left(Align::Min)
            } else {
                Layout::left_to_right(Align::Min)
            },
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let ctx = ui.ctx().clone();
                let icon_size = ab.min(TAP_MIN) * 0.5;
                if icon_action_button(
                    ui,
                    &mut self.icons,
                    &ctx,
                    palette,
                    ab,
                    icon_size,
                    Icon::Undo,
                    palette.pad_text,
                    self.game.can_undo(),
                )
                .clicked()
                {
                    self.game.undo();
                    self.save_now();
                }
                if icon_action_button(
                    ui,
                    &mut self.icons,
                    &ctx,
                    palette,
                    ab,
                    icon_size,
                    Icon::Redo,
                    palette.pad_text,
                    self.game.can_redo(),
                )
                .clicked()
                {
                    self.game.redo();
                    self.save_now();
                }
                let (notes_fill, notes_tint) = if self.note_mode {
                    (palette.accent, Color32::WHITE)
                } else {
                    (palette.pad_bg, palette.pad_text)
                };
                if icon_action_button_fill(
                    ui,
                    &mut self.icons,
                    &ctx,
                    ab,
                    icon_size,
                    Icon::Notes,
                    notes_tint,
                    notes_fill,
                    true,
                )
                .clicked()
                {
                    self.note_mode = !self.note_mode;
                }
                let hints_left = 3u32.saturating_sub(self.game.stats().hints_used);
                if icon_action_button(
                    ui,
                    &mut self.icons,
                    &ctx,
                    palette,
                    ab,
                    icon_size,
                    Icon::Hint,
                    palette.pad_text,
                    hints_left > 0,
                )
                .on_hover_text(format!("Hint ({hints_left} left)"))
                .clicked()
                {
                    self.hint();
                }
            },
        );
    }

    fn draw_pad_button(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        size: f32,
        digit: Option<u8>,
    ) {
        let (label, sub, complete) = match digit {
            Some(d) => (
                d.to_string(),
                Some(format!("{}", self.game.digit_remaining(d))),
                self.game.is_digit_complete(d),
            ),
            None => ("⌫".to_string(), None, false),
        };

        let text_color = if complete {
            palette.pad_done_text
        } else {
            palette.pad_text
        };

        let button = match digit {
            Some(_) => egui::Button::new(
                RichText::new(&label)
                    .size(size * 0.42)
                    .color(text_color)
                    .strong(),
            ),
            None => {
                egui::Button::image(
                    self.icons
                        .image(ui.ctx(), Icon::Erase, size * 0.5, text_color),
                )
            }
        }
        .fill(palette.pad_bg)
        .corner_radius(CornerRadius::same(12))
        .min_size(Vec2::new(size, size));

        let response = ui
            .add_enabled(!complete, button)
            .on_hover_text(match digit {
                Some(_) => "Long-press to toggle a note",
                None => "Erase",
            });

        if let (Some(d), Some(sub)) = (digit, sub) {
            // Painting the remaining-count under the centered digit.
            let rect = response.rect;
            ui.painter().text(
                rect.center_bottom() - Vec2::new(0.0, size * 0.16),
                Align2::CENTER_CENTER,
                sub,
                FontId::proportional(size * 0.2),
                palette.pad_count_text,
            );

            if response.clicked() {
                self.input_digit(d);
            } else if response.long_touched() || response.secondary_clicked() {
                // Long-press a digit = switch to notes and toggle it.
                self.note_mode = true;
                self.input_digit(d);
            }
        } else if response.clicked() {
            self.erase();
        }
    }

    // ----------------------------------------------------------- overlays

    fn draw_dialogs(&mut self, ctx: &egui::Context, palette: &Palette) {
        match self.dialog {
            Dialog::None => {}
            Dialog::Settings => self.draw_settings(ctx, palette),
            Dialog::NewGame => self.draw_new_game(ctx, palette),
            Dialog::Win => self.draw_win(ctx, palette),
        }
    }

    fn draw_settings(&mut self, ctx: &egui::Context, palette: &Palette) {
        let mut settings = self.settings;
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::unique("settings_modal")).show(ctx, |ui| {
            ui.set_min_width(300.0);
            ui.heading("Settings");
            ui.add_space(8.0);
            ui.label(RichText::new("Theme").strong());
            ui.horizontal(|ui| {
                for mode in ThemeMode::all() {
                    if ui
                        .selectable_label(settings.theme == mode, mode.label())
                        .clicked()
                    {
                        settings.theme = mode;
                    }
                }
            });
            ui.add_space(10.0);
            ui.checkbox(
                &mut settings.check_against_solution,
                "Check against solution (strict)",
            );
            ui.checkbox(&mut settings.auto_remove_notes, "Clean notes automatically");
            ui.checkbox(&mut settings.highlight_same_digit, "Highlight same digit");
            ui.checkbox(
                &mut settings.highlight_row_col_box,
                "Highlight row / column / box",
            );
            ui.checkbox(&mut settings.auto_pause, "Auto-pause when backgrounded");
            ui.checkbox(&mut settings.left_handed, "Left-handed number pad");
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("Done").fill(palette.accent))
                    .clicked()
                {
                    close = true;
                }
            });
        });
        if settings != self.settings {
            self.set_settings(settings);
        }
        if close || modal.should_close() {
            self.dialog = Dialog::None;
        }
    }

    fn draw_new_game(&mut self, ctx: &egui::Context, palette: &Palette) {
        let Some((difficulty, daily)) = self.pending_start else {
            self.dialog = Dialog::None;
            return;
        };
        let mut choice: Option<bool> = None;
        let modal = egui::Modal::new(egui::Id::unique("newgame_modal")).show(ctx, |ui| {
            ui.set_min_width(280.0);
            ui.heading("New game");
            ui.add_space(6.0);
            ui.label("Discard the game in progress?");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("Discard").fill(palette.danger))
                    .clicked()
                {
                    choice = Some(true);
                }
                if ui.button("Cancel").clicked() {
                    choice = Some(false);
                }
            });
        });
        if choice.is_none() && modal.should_close() {
            choice = Some(false);
        }
        match choice {
            Some(true) => {
                self.pending_start = None;
                if daily {
                    self.new_daily();
                } else {
                    self.new_game(difficulty);
                }
            }
            Some(false) => {
                self.pending_start = None;
                self.dialog = Dialog::None;
            }
            None => {}
        }
    }

    fn draw_win(&mut self, ctx: &egui::Context, palette: &Palette) {
        let stats = self.game.stats();
        let mut action: Option<bool> = None;
        egui::Modal::new(egui::Id::unique("win_modal")).show(ctx, |ui| {
            ui.set_min_width(280.0);
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - 110.0) / 2.0).max(0.0));
                    let img = self.icons.image(ctx, Icon::Win, 28.0, palette.positive);
                    ui.add(img);
                    ui.label(
                        RichText::new("Solved!")
                            .color(palette.positive)
                            .size(text::TITLE)
                            .strong(),
                    );
                });
                ui.label(format!("Time: {}", format_time(self.elapsed)));
                ui.label(format!("Difficulty: {}", self.difficulty.label()));
                ui.label(format!(
                    "Mistakes: {}    Hints: {}",
                    stats.mistakes, stats.hints_used
                ));
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Button::new("New game").fill(palette.accent))
                        .clicked()
                    {
                        action = Some(true);
                    }
                    if ui.button("Replay").clicked() {
                        action = Some(false);
                    }
                });
            });
        });
        match action {
            Some(true) => self.new_game(self.difficulty),
            Some(false) => {
                let difficulty = self.difficulty;
                self.new_game(difficulty);
            }
            None => {}
        }
    }

    fn draw_pause_overlay(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let rect = ui.max_rect();
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(180));
        painter.text(
            rect.center() - Vec2::new(0.0, 20.0),
            Align2::CENTER_CENTER,
            "Paused",
            FontId::proportional(32.0),
            Color32::WHITE,
        );
        let mut resume_rect =
            Rect::from_center_size(rect.center() + Vec2::new(0.0, 36.0), Vec2::new(140.0, 48.0));
        resume_rect = resume_rect.intersect(rect);
        let response = ui.allocate_rect(resume_rect, Sense::click());
        painter.rect_filled(resume_rect, CornerRadius::same(12), palette.accent);
        painter.text(
            resume_rect.center(),
            Align2::CENTER_CENTER,
            "Resume",
            FontId::proportional(18.0),
            Color32::WHITE,
        );
        if response.clicked() {
            self.toggle_pause();
        }
    }
}

impl eframe::App for SudokuApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        let system = ctx.system_theme();
        let theme = self.resolution(system);
        ctx.set_visuals(crate::theme::visuals_for(theme));
        let palette = Palette::of(theme);

        // Resolve safe-area insets (Android). `screen_rect` is in points, so ask
        // the provider in physical pixels via the current pixels-per-point.
        let scale = ctx.pixels_per_point();
        let screen = ctx.viewport_rect();
        self.insets = self
            .insets_provider
            .insets((screen.width() * scale, screen.height() * scale), scale);
        let insets = self.insets;

        // Clear the status bar / notch at the top.
        if insets.top > 0.0 {
            egui::Panel::top("safe_top")
                .resizable(false)
                .frame(egui::Frame::new().fill(palette.panel_bg))
                .exact_size(insets.top)
                .show(ui, |_ui| {});
        }

        egui::Panel::top("top_bar")
            .frame(chrome_frame(palette.panel_bg))
            .show(ui, |ui| {
                self.draw_top_bar(ui, &palette);
            });

        // Clear the gesture/navigation bar at the bottom.
        if insets.bottom > 0.0 {
            egui::Panel::bottom("safe_bottom")
                .resizable(false)
                .frame(egui::Frame::new().fill(palette.panel_bg))
                .exact_size(insets.bottom)
                .show(ui, |_ui| {});
        }

        // Number pad hidden while paused, to reinforce the overlay.
        if !self.paused {
            egui::Panel::bottom("number_pad")
                .frame(chrome_frame(palette.panel_bg))
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    self.draw_number_pad(ui, &palette);
                    ui.add_space(2.0);
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette.panel_bg))
            .show(ui, |ui| {
                if self.paused {
                    let full = ui.max_rect();
                    ui.allocate_rect(full, Sense::hover());
                    self.draw_pause_overlay(ui, &palette);
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(6.0);
                        self.draw_board(ui, &palette);
                    });
                }
            });

        self.draw_dialogs(ui.ctx(), &palette);

        if !self.paused && self.dialog == Dialog::None {
            self.handle_keyboard(ui);
        }

        if self.running() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}

impl SudokuApp {
    fn handle_keyboard(&mut self, ui: &egui::Ui) {
        let keys = ui.ctx().input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => Some((*key, *modifiers)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });
        for (key, modifiers) in keys {
            match key {
                egui::Key::Num1 => self.input_digit(1),
                egui::Key::Num2 => self.input_digit(2),
                egui::Key::Num3 => self.input_digit(3),
                egui::Key::Num4 => self.input_digit(4),
                egui::Key::Num5 => self.input_digit(5),
                egui::Key::Num6 => self.input_digit(6),
                egui::Key::Num7 => self.input_digit(7),
                egui::Key::Num8 => self.input_digit(8),
                egui::Key::Num9 => self.input_digit(9),
                egui::Key::Backspace | egui::Key::Delete | egui::Key::Num0 => self.erase(),
                egui::Key::ArrowUp | egui::Key::W => self.move_selection(-1, 0),
                egui::Key::ArrowDown | egui::Key::S => self.move_selection(1, 0),
                egui::Key::ArrowLeft | egui::Key::A => self.move_selection(0, -1),
                egui::Key::ArrowRight | egui::Key::D => self.move_selection(0, 1),
                egui::Key::Z if modifiers.command => {
                    self.game.undo();
                    self.save_now();
                }
                egui::Key::Y if modifiers.command => {
                    self.game.redo();
                    self.save_now();
                }
                egui::Key::N => self.note_mode = !self.note_mode,
                egui::Key::H => self.hint(),
                _ => {}
            }
        }
    }

    fn move_selection(&mut self, dr: isize, dc: isize) {
        let (row, col) = self.selected.unwrap_or((0, 0));
        let row = (row as isize + dr).rem_euclid(SIZE as isize) as usize;
        let col = (col as isize + dc).rem_euclid(SIZE as isize) as usize;
        self.select(row, col);
    }
}

/// A square button showing only an [`Icon`], with an optional tooltip.
fn icon_button(
    ui: &mut egui::Ui,
    icons: &mut IconStore,
    icon: Icon,
    size: f32,
    tint: Color32,
    tooltip: Option<&str>,
) -> egui::Response {
    let image = icons.image(ui.ctx(), icon, size * 0.7, tint);
    let response = ui.add_sized(Vec2::splat(size), egui::Button::image(image));
    match tooltip {
        Some(text) => response.on_hover_text(text),
        None => response,
    }
}

/// A wide action button (undo/redo/hint) filled with the palette's pad color.
#[allow(clippy::too_many_arguments)]
fn icon_action_button(
    ui: &mut egui::Ui,
    icons: &mut IconStore,
    ctx: &egui::Context,
    palette: &Palette,
    width: f32,
    icon_size: f32,
    icon: Icon,
    tint: Color32,
    enabled: bool,
) -> egui::Response {
    icon_action_button_fill(
        ui,
        icons,
        ctx,
        width,
        icon_size,
        icon,
        tint,
        palette.pad_bg,
        enabled,
    )
}

/// A wide action button with an explicit fill (used for the Notes toggle).
#[allow(clippy::too_many_arguments)]
fn icon_action_button_fill(
    ui: &mut egui::Ui,
    icons: &mut IconStore,
    ctx: &egui::Context,
    width: f32,
    icon_size: f32,
    icon: Icon,
    tint: Color32,
    fill: Color32,
    enabled: bool,
) -> egui::Response {
    let image = icons.image(ctx, icon, icon_size, tint);
    let button = egui::Button::image(image)
        .fill(fill)
        .corner_radius(CornerRadius::same(12))
        .min_size(Vec2::new(width, TAP_MIN));
    ui.add_enabled(enabled, button)
}

fn format_time(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// Split the optional `t=<seconds>` header the UI prepends from the core game
/// encoding. Returns `(game_text, elapsed)`.
fn split_save_header(text: &str) -> (String, f64) {
    if let Some(rest) = text.strip_prefix("t=")
        && let Some((value, tail)) = rest.split_once('\n')
        && let Ok(seconds) = value.trim().parse::<f64>()
    {
        return (tail.to_string(), seconds);
    }
    (text.to_string(), 0.0)
}

/// Seconds since this app started, monotonic and clock-adjustment safe.
/// Uses `web-time` so it works identically on desktop and Android.
fn now_seconds() -> f64 {
    use std::sync::OnceLock;
    static START: OnceLock<web_time::Instant> = OnceLock::new();
    let start = START.get_or_init(web_time::Instant::now);
    start.elapsed().as_secs_f64()
}

/// Wall-clock seconds since the Unix epoch (used only for the daily seed).
fn now_seconds_unix() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
