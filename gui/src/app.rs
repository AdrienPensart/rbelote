//! The application: a setup screen, then a table.

use crate::game::{Game, Settings};
use crate::textures::Textures;
use crate::view::{self, Outcome};
use rbelote_core::position::Position;
use strum::IntoEnumIterator;

enum Screen {
    Setup,
    Table(Box<Game>),
}

pub struct BeloteApp {
    settings: Settings,
    screen: Screen,
    textures: Textures,
    /// Why the last game could not start, if it could not.
    error: Option<String>,
}

impl BeloteApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        cc.egui_ctx.set_zoom_factor(1.0);
        Self {
            settings: Settings::default(),
            screen: Screen::Setup,
            textures: Textures::load(&cc.egui_ctx),
            error: None,
        }
    }

    /// The setup screen. Returns true when the player wants to start.
    fn setup(settings: &mut Settings, error: Option<&str>, ui: &mut egui::Ui) -> bool {
        let mut start = false;
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("Belote").size(36.0));
                ui.add_space(24.0);
                egui::Grid::new("setup")
                    .num_columns(2)
                    .spacing([24.0, 12.0])
                    .show(ui, |ui| {
                        ui.label("Your seat");
                        egui::ComboBox::from_id_salt("seat")
                            .selected_text(settings.seat.to_string())
                            .show_ui(ui, |ui| {
                                for seat in Position::iter() {
                                    ui.selectable_value(&mut settings.seat, seat, seat.to_string());
                                }
                            });
                        ui.end_row();

                        ui.label("Deals at most");
                        ui.add(egui::Slider::new(&mut settings.deals, 1..=50));
                        ui.end_row();

                        ui.label("Play to");
                        ui.add(
                            egui::Slider::new(&mut settings.target, 100..=3000)
                                .step_by(100.0)
                                .suffix(" points"),
                        );
                        ui.end_row();

                        ui.label("Bots");
                        ui.vertical(|ui| {
                            ui.checkbox(
                                &mut settings.attack,
                                "Bots never take, you play every contract",
                            );
                            ui.checkbox(
                                &mut settings.auto,
                                "Play for me when only one card is legal",
                            );
                        });
                        ui.end_row();

                        ui.label("Bot thinking time");
                        ui.add(
                            egui::Slider::new(&mut settings.bot_delay, 0.0..=3.0)
                                .suffix(" s")
                                .fixed_decimals(1),
                        );
                        ui.end_row();

                        ui.label("Finished trick stays");
                        ui.add(
                            egui::Slider::new(&mut settings.trick_delay, 0.0..=6.0)
                                .suffix(" s")
                                .fixed_decimals(1),
                        );
                        ui.end_row();
                    });
                ui.add_space(24.0);
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("Deal the cards").size(20.0),
                    ))
                    .clicked()
                {
                    start = true;
                }
                if let Some(error) = error {
                    ui.add_space(12.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 90, 90),
                        format!("The engine could not deal: {error}"),
                    );
                }
            });
        });
        start
    }
}

impl eframe::App for BeloteApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let next = match &mut self.screen {
            Screen::Setup => {
                if Self::setup(&mut self.settings, self.error.as_deref(), ui) {
                    match Game::new(&self.settings) {
                        Ok(game) => {
                            self.error = None;
                            Some(Screen::Table(Box::new(game)))
                        }
                        Err(error) => {
                            self.error = Some(error.to_string());
                            None
                        }
                    }
                } else {
                    None
                }
            }
            Screen::Table(game) => {
                game.tick(ui.ctx());
                match view::show(game, ui, &self.textures) {
                    Outcome::Continue => None,
                    Outcome::Quit => Some(Screen::Setup),
                }
            }
        };
        if let Some(screen) = next {
            self.screen = screen;
        }
    }
}
