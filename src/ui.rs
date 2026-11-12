use crate::{Difficulty, Game, SIZE, generate};
use eframe::egui::{self, Align2, Color32, FontId, Rect, Sense, Stroke, Vec2};

const GRID_BG: Color32 = Color32::from_rgb(250, 250, 252);
const LINE_THIN: Color32 = Color32::from_rgb(180, 185, 195);
const LINE_THICK: Color32 = Color32::from_rgb(70, 75, 90);
const HIGHLIGHT_ROWCOL: Color32 = Color32::from_rgb(226, 232, 245);
const HIGHLIGHT_SELECTED: Color32 = Color32::from_rgb(196, 212, 240);
const HIGHLIGHT_SAME: Color32 = Color32::from_rgb(212, 224, 246);
const CONFLICT_BG: Color32 = Color32::from_rgb(245, 200, 200);
const GIVEN_TEXT: Color32 = Color32::from_rgb(30, 34, 45);
const USER_TEXT: Color32 = Color32::from_rgb(40, 110, 200);
const NOTE_TEXT: Color32 = Color32::from_rgb(110, 120, 140);
const DIGIT_DONE_TEXT: Color32 = Color32::from_rgb(170, 175, 185);

pub struct SudokuApp {
    game: Game,
    difficulty: Difficulty,
    seed: u64,
    selected: Option<(usize, usize)>,
    note_mode: bool,
    status: String,
}

impl SudokuApp {
    pub fn new(seed: u64) -> Self {
        let difficulty = Difficulty::Medium;
        Self {
            game: Game::new(generate(difficulty, seed)),
            difficulty,
            seed,
            selected: None,
            note_mode: false,
            status: String::new(),
        }
    }

    fn new_game(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.game = Game::new(generate(self.difficulty, self.seed));
        self.selected = None;
        self.note_mode = false;
        self.status.clear();
    }

    fn select(&mut self, row: usize, col: usize) {
        self.selected = Some((row, col));
        self.status.clear();
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
            self.game.set_value(row, col, digit);
        }

        if self.game.is_complete() {
            self.status = "Solved!".to_string();
        }
    }

    fn erase(&mut self) {
        if let Some((row, col)) = self.selected
            && !self.game.is_given(row, col)
        {
            self.game.set_value(row, col, 0);
        }
    }

    fn cell_rect(grid_rect: Rect, cell: f32, row: usize, col: usize) -> Rect {
        let min = grid_rect.min + Vec2::new(col as f32 * cell, row as f32 * cell);
        Rect::from_min_size(min, Vec2::splat(cell))
    }

    fn draw_board(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = available.x.min(available.y).clamp(180.0, 560.0);
        let (grid_rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
        let cell = size / SIZE as f32;
        let painter = ui.painter_at(grid_rect);

        painter.rect_filled(grid_rect, 0.0, GRID_BG);

        let conflicts = self.game.conflicts();

        if let Some((sel_row, sel_col)) = self.selected {
            let box_row = sel_row / 3 * 3;
            let box_col = sel_col / 3 * 3;
            for r in box_row..box_row + 3 {
                for c in box_col..box_col + 3 {
                    painter.rect_filled(
                        Self::cell_rect(grid_rect, cell, r, c),
                        0.0,
                        HIGHLIGHT_ROWCOL,
                    );
                }
            }
            for i in 0..SIZE {
                painter.rect_filled(
                    Self::cell_rect(grid_rect, cell, sel_row, i),
                    0.0,
                    HIGHLIGHT_ROWCOL,
                );
                painter.rect_filled(
                    Self::cell_rect(grid_rect, cell, i, sel_col),
                    0.0,
                    HIGHLIGHT_ROWCOL,
                );
            }

            let selected_value = self.game.value(sel_row, sel_col);
            if selected_value != 0 {
                for r in 0..SIZE {
                    for c in 0..SIZE {
                        if self.game.value(r, c) == selected_value {
                            painter.rect_filled(
                                Self::cell_rect(grid_rect, cell, r, c),
                                0.0,
                                HIGHLIGHT_SAME,
                            );
                        }
                    }
                }
            }

            painter.rect_filled(
                Self::cell_rect(grid_rect, cell, sel_row, sel_col),
                0.0,
                HIGHLIGHT_SELECTED,
            );
        }

        for &(r, c) in &conflicts {
            painter.rect_filled(Self::cell_rect(grid_rect, cell, r, c), 0.0, CONFLICT_BG);
        }

        let big_font = FontId::proportional(cell * 0.6);
        let small_font = FontId::proportional(cell * 0.24);

        for row in 0..SIZE {
            for col in 0..SIZE {
                let rect = Self::cell_rect(grid_rect, cell, row, col);
                let value = self.game.value(row, col);

                if value != 0 {
                    let color = if self.game.is_given(row, col) {
                        GIVEN_TEXT
                    } else {
                        USER_TEXT
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
                            NOTE_TEXT,
                        );
                    }
                }
            }
        }

        for i in 0..=SIZE {
            let thick = i % 3 == 0;
            let stroke = Stroke::new(
                if thick { 2.5 } else { 1.0 },
                if thick { LINE_THICK } else { LINE_THIN },
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

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Difficulty:");
            for difficulty in Difficulty::all() {
                if ui
                    .selectable_label(self.difficulty == difficulty, format!("{difficulty:?}"))
                    .clicked()
                {
                    self.difficulty = difficulty;
                    self.new_game();
                }
            }
        });

        ui.horizontal(|ui| {
            if ui.button("New game").clicked() {
                self.new_game();
            }
            if ui.button("Undo").clicked() {
                self.game.undo();
            }
            if ui.selectable_label(self.note_mode, "Notes").clicked() {
                self.note_mode = !self.note_mode;
            }
            if !self.status.is_empty() {
                ui.colored_label(Color32::from_rgb(30, 150, 70), &self.status);
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for digit in 1..=9u8 {
                let finished = self.game.is_digit_complete(digit);
                let mut text = egui::RichText::new(digit.to_string()).size(18.0);
                if finished {
                    text = text.color(DIGIT_DONE_TEXT);
                }
                let button = egui::Button::new(text).min_size(Vec2::splat(40.0));
                let response = ui.add_enabled(!finished, button);
                if !finished && response.clicked() {
                    self.input_digit(digit);
                }
            }
            if ui
                .add_sized(Vec2::splat(40.0), egui::Button::new("⌫"))
                .clicked()
            {
                self.erase();
            }
        });
    }
}

impl eframe::App for SudokuApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("controls").show(ui, |ui| {
            ui.add_space(4.0);
            self.draw_controls(ui);
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                self.draw_board(ui);
            });
        });
    }
}
