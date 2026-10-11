//! Drawing the table, and reading clicks on it.

use crate::cards::{self, paint_suit, HAND_CARD, SMALL_CARD, TRICK_CARD};
use crate::game::{self, Game, Line, Tone, Trick};
use crate::log;
use crate::textures::Textures;
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Frame, Id, Pos2, Rect, Sense, Stroke,
    StrokeKind, Ui,
};
use rbelote_core::card::Card;
use rbelote_core::constants::MAX_PLAYERS;
use rbelote_core::contract::Contract;
use rbelote_core::game::{Answer, Request};
use rbelote_core::position::Position;
use rbelote_core::team::Team;
use std::cmp::Ordering;
use std::f32::consts::{FRAC_PI_2, TAU};
use strum::IntoEnumIterator;

const FELT: Color32 = Color32::from_rgb(28, 92, 58);
const FELT_DARK: Color32 = Color32::from_rgb(20, 70, 44);
const INK: Color32 = Color32::from_rgb(235, 235, 225);
const ACCENT: Color32 = Color32::from_rgb(255, 214, 90);
const BADGE: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 110);
/// The taker's name, wherever it is written.
const TAKER: Color32 = Color32::from_rgb(255, 150, 70);
/// The taker's partner's name.
const ALLY: Color32 = Color32::from_rgb(120, 200, 255);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Continue,
    Quit,
}

/// The whole table for one frame.
pub fn show(game: &mut Game, ui: &mut Ui, textures: &Textures) -> Outcome {
    let mut outcome = Outcome::Continue;
    let ctx = ui.ctx().clone();

    egui::Panel::top("bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            top_bar(game, ui);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Quit to menu").clicked() {
                    outcome = Outcome::Quit;
                }
                ui.toggle_value(&mut game.show_log, "Log");
                ui.toggle_value(&mut game.show_tricks, "Tricks won");
                ui.toggle_value(&mut game.show_last, "Last trick");
                ui.checkbox(&mut game.settings.auto, "Autoplay forced cards");
                ui.add(
                    egui::Slider::new(&mut game.settings.bot_delay, 0.0..=3.0)
                        .suffix(" s")
                        .fixed_decimals(1)
                        .text("Bot delay"),
                );
            });
        });
    });

    if game.show_log {
        egui::Panel::right("log")
            .default_size(320.0)
            .show(ui, |ui| {
                ui.heading("Table talk");
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        for line in &game.log {
                            ui.label(layout(ui, line));
                        }
                    });
            });
    }

    let felt = Frame::new().fill(FELT_DARK).inner_margin(12.0);
    let mut answer = None;
    egui::Panel::bottom("hand").frame(felt).show(ui, |ui| {
        answer = hand_panel(game, ui, textures);
    });

    egui::CentralPanel::default()
        .frame(Frame::new().fill(FELT))
        .show(ui, |ui| table(game, ui, textures));

    if let Some(answer) = answer {
        game.answer(answer);
    }

    if game.show_last {
        let mut open = true;
        egui::Window::new("Last trick")
            .open(&mut open)
            .resizable(false)
            .default_pos(ctx.content_rect().left_top() + vec2(20.0, 60.0))
            .show(&ctx, |ui| match &game.last_trick {
                Some(trick) => trick_entry(game, ui, "Last trick", trick, textures),
                None => {
                    ui.label("No trick finished yet");
                }
            });
        game.show_last = open;
    }

    if game.show_history {
        let mut open = true;
        egui::Window::new("The deal, trick by trick")
            .open(&mut open)
            .resizable(false)
            .default_pos(ctx.content_rect().left_top() + vec2(20.0, 60.0))
            .show(&ctx, |ui| history(game, ui, textures));
        game.show_history = open;
    }

    if game.show_tricks {
        let mut open = true;
        egui::Window::new("Tricks won")
            .open(&mut open)
            .resizable(false)
            .default_pos(ctx.content_rect().left_top() + vec2(20.0, 60.0))
            .show(&ctx, |ui| tricks_won(game, ui, textures));
        game.show_tricks = open;
    }

    if !game.paused() && game.is_finished() {
        egui::Window::new("Game over")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(&ctx, |ui| {
                let verdict = match game.winner() {
                    Some(team) if team == game.human().team() => {
                        ("Your team wins!", Color32::from_rgb(120, 230, 120))
                    }
                    Some(_) => ("Your team loses", Color32::from_rgb(255, 110, 110)),
                    None => ("A draw", INK),
                };
                ui.label(
                    egui::RichText::new(verdict.0)
                        .size(26.0)
                        .strong()
                        .color(verdict.1),
                );
                ui.add_space(8.0);
                egui::Grid::new("final")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for team in Team::iter() {
                            ui.label(team.to_string());
                            ui.label(game.engine.points()[team].to_string());
                            ui.end_row();
                        }
                        ui.label("Deals played");
                        ui.label(game.deals_played().to_string());
                        ui.end_row();
                    });
                ui.add_space(8.0);
                if ui.button("Back to menu").clicked() {
                    outcome = Outcome::Quit;
                }
            });
    }
    outcome
}

fn top_bar(game: &Game, ui: &mut Ui) {
    let deal = game.deal();
    ui.label(format!(
        "Deal {} of {}",
        (game.deals_played() + 1).min(game.settings.deals),
        game.settings.deals
    ));
    ui.separator();
    ui.label(format!("Dealer: {}", game.name(deal.dealer)));
    ui.separator();
    match (deal.taker, deal.trump) {
        (Some(taker), Some(trump)) => {
            ui.colored_label(TAKER, game.name(taker));
            ui.label("plays");
            ui.colored_label(cards::felt_rgb(trump), log::trump_label(trump));
            ui.label("with");
            ui.colored_label(ALLY, game.name(game::partner(taker)));
        }
        (Some(_) | None, None) | (None, Some(_)) => match deal.returned {
            Some(card) => {
                ui.label("Bidding on the");
                ui.colored_label(cards::felt_rgb(card.color()), cards::name(card));
            }
            None => {
                ui.label("Nobody took");
            }
        },
    }
    ui.separator();
    for team in Team::iter() {
        ui.label(format!("{team}: {}", game.engine.points()[team]));
    }
    if game.engine.litige() > 0 {
        ui.label(format!("({} points held over)", game.engine.litige()));
    }
    if let Some(error) = &game.error {
        ui.separator();
        ui.colored_label(
            Color32::from_rgb(255, 90, 90),
            format!("Engine error: {error}"),
        );
    }
}

/// Where a seat sits on the screen: the human at the bottom, then the seats
/// clockwise in playing order, so that the partner faces the human.
fn seat_direction(game: &Game, seat: Position) -> egui::Vec2 {
    let offset = (game::index(seat) + MAX_PLAYERS - game::index(game.human())) % MAX_PLAYERS;
    let angle = (offset as f32).mul_add(TAU / MAX_PLAYERS as f32, FRAC_PI_2);
    vec2(angle.cos(), angle.sin())
}

/// The color a name is written in: the attack stands out, once it is known.
fn name_color(game: &Game, seat: Position) -> Color32 {
    if game.taker() == Some(seat) {
        TAKER
    } else if game.ally() == Some(seat) {
        ALLY
    } else {
        INK
    }
}

/// A log line laid out with its names in color.
fn layout(ui: &Ui, line: &Line) -> egui::text::LayoutJob {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let plain = ui.visuals().text_color();
    let mut job = egui::text::LayoutJob::default();
    for span in line {
        let color = match span.tone {
            Tone::Plain => plain,
            Tone::Taker => TAKER,
            Tone::Ally => ALLY,
        };
        job.append(
            &span.text,
            0.0,
            egui::TextFormat::simple(font.clone(), color),
        );
    }
    job
}

fn table(game: &Game, ui: &Ui, textures: &Textures) {
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    let center = rect.center();
    let radius = vec2(rect.width() * 0.40, rect.height() * 0.38);

    for seat in Position::iter() {
        let direction = seat_direction(game, seat);
        let position = center + vec2(direction.x * radius.x, direction.y * radius.y);
        seat_badge(game, &painter, position, seat);
    }

    let trick_at = |seat: Position| -> Pos2 {
        let direction = seat_direction(game, seat);
        center + vec2(direction.x * radius.x * 0.42, direction.y * radius.y * 0.42)
    };

    if game.trick.cards.is_empty() {
        if let (Some(last), Some(left)) = (&game.last_trick, game.last_trick_showing()) {
            ui.ctx().request_repaint_after(left);
            paint_trick(&painter, textures, last, trick_at, true);
            if let Some(winner) = last.winner {
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    format!("{} wins", game.name(winner)),
                    FontId::proportional(18.0),
                    ACCENT,
                );
            }
        } else if let Some(card) = game.deal().returned {
            paint_returned(&painter, textures, center, card);
        }
    } else {
        paint_trick(&painter, textures, &game.trick, trick_at, false);
    }
}

fn paint_trick(
    painter: &egui::Painter,
    textures: &Textures,
    trick: &Trick,
    trick_at: impl Fn(Position) -> Pos2,
    faded: bool,
) {
    for (seat, card) in &trick.cards {
        let rect = Rect::from_center_size(trick_at(*seat), TRICK_CARD);
        cards::paint_card(painter, rect, *card, textures);
        if faded {
            painter.rect_filled(rect, CornerRadius::same(6), Color32::from_black_alpha(60));
        }
    }
}

/// The card turned up, in the middle of the table while the bidding lasts.
fn paint_returned(painter: &egui::Painter, textures: &Textures, center: Pos2, card: Card) {
    painter.text(
        center - vec2(0.0, TRICK_CARD.y.mul_add(0.5, 16.0)),
        Align2::CENTER_CENTER,
        "Turned up",
        FontId::proportional(16.0),
        INK,
    );
    cards::paint_card(
        painter,
        Rect::from_center_size(center, TRICK_CARD),
        card,
        textures,
    );
}

fn seat_badge(game: &Game, painter: &egui::Painter, position: Pos2, seat: Position) {
    let size = vec2(210.0, 66.0);
    let rect = Rect::from_center_size(position, size);
    let is_turn = game
        .pending()
        .is_some_and(|request| request.position() == seat);
    let edge = if is_turn {
        ACCENT
    } else {
        Color32::TRANSPARENT
    };
    painter.rect(
        rect,
        CornerRadius::same(8),
        BADGE,
        Stroke::new(2.0, edge),
        StrokeKind::Inside,
    );

    let name = game.name(seat);
    let dealer = if game.deal().dealer == seat {
        " (dealer)"
    } else {
        ""
    };
    let name_rect = painter.text(
        rect.min + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        format!("{name}{dealer}"),
        FontId::proportional(17.0),
        name_color(game, seat),
    );
    if let Some(bid) = game.bid_label(seat) {
        painter.text(
            pos2(name_rect.max.x + 8.0, name_rect.center().y),
            Align2::LEFT_CENTER,
            bid,
            FontId::proportional(15.0),
            ACCENT,
        );
    }
    painter.text(
        rect.right_top() + vec2(-10.0, 6.0),
        Align2::RIGHT_TOP,
        game.engine.points()[seat.team()].to_string(),
        FontId::proportional(17.0),
        INK,
    );

    let mut status: Vec<Piece> = Vec::new();
    if let Some(taker) = game.taker() {
        if taker == seat {
            status.push(Piece::Text("Taker".to_string()));
            if let Some(trump) = game.trump() {
                status.push(Piece::Suit(trump));
            }
        } else if taker.team() == seat.team() {
            status.push(Piece::Text("Partner".to_string()));
        } else {
            status.push(Piece::Text("Defender".to_string()));
        }
    }
    // Belote is a secret until it is played: only the human sees their own.
    if seat == game.human() && game.belote(seat) {
        status.push(Piece::Text("Belote!".to_string()));
    }
    status.push(Piece::Text(format!("{} cards", game.hand(seat).len())));
    paint_pieces(painter, rect.min + vec2(10.0, 36.0), &status);

    // Card points taken so far in this deal by the seat's team.
    if game.trump().is_some() {
        painter.text(
            rect.right_bottom() - vec2(10.0, 6.0),
            Align2::RIGHT_BOTTOM,
            format!("{} pts", game.team_points(seat.team())),
            FontId::proportional(13.0),
            INK,
        );
    }
}

/// A bit of a status line: words, or a suit symbol painted in its color.
enum Piece {
    Text(String),
    Suit(rbelote_core::card::Color),
}

/// Paint `pieces` on one line from `start`, separated by dots, a suit symbol
/// following its words without a separator.
fn paint_pieces(painter: &egui::Painter, start: Pos2, pieces: &[Piece]) {
    let font = FontId::proportional(13.0);
    let mut x = start.x;
    let mut first = true;
    for piece in pieces {
        match piece {
            Piece::Text(text) => {
                let text = if first {
                    text.clone()
                } else {
                    format!(" · {text}")
                };
                x = painter
                    .text(
                        pos2(x, start.y),
                        Align2::LEFT_CENTER,
                        text,
                        font.clone(),
                        INK,
                    )
                    .max
                    .x;
            }
            Piece::Suit(color) => {
                x += 8.0;
                paint_suit(
                    painter,
                    pos2(x, start.y),
                    12.0,
                    *color,
                    cards::felt_rgb(*color),
                );
                x += 8.0;
            }
        }
        first = false;
    }
}

/// The prompt and the human's hand. Returns the answer clicked, if any.
fn hand_panel(game: &mut Game, ui: &mut Ui, textures: &Textures) -> Option<Answer> {
    if game.paused() {
        deal_over(game, ui, textures);
        return None;
    }
    let human = game.human();
    let request = game.human_request();
    let mut answer = None;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(game.name(human))
                .size(18.0)
                .color(ACCENT),
        );
        ui.separator();
        match &request {
            None => {
                let waiting = game.pending().map_or_else(
                    || "Game over".to_string(),
                    |request| format!("Waiting for {}…", game.name(request.position())),
                );
                ui.label(waiting);
            }
            Some(Request::Take { card_returned, .. }) => {
                ui.label("Take the");
                ui.colored_label(
                    cards::felt_rgb(card_returned.color()),
                    cards::name(*card_returned),
                );
                ui.label(format!(
                    "and play {}?",
                    log::trump_label(card_returned.color())
                ));
                if ui.button("Take").clicked() {
                    answer = Some(Answer::Take(true));
                }
                if ui.button("Pass").clicked() {
                    answer = Some(Answer::Take(false));
                }
            }
            Some(Request::Trump { choices, .. }) => {
                ui.label("Nobody took. Name a trump:");
                for contract in choices {
                    let label = contract.color().map_or_else(
                        || egui::RichText::new(log::contract_label(*contract)),
                        |color| {
                            egui::RichText::new(log::trump_label(color))
                                .color(cards::felt_rgb(color))
                        },
                    );
                    if ui.button(label).clicked() {
                        answer = Some(Answer::Trump(*contract));
                    }
                }
                if !choices.contains(&Contract::Pass) && ui.button("Pass").clicked() {
                    answer = Some(Answer::Trump(Contract::Pass));
                }
            }
            Some(Request::Play { .. }) => {
                let called = game
                    .engine
                    .turn()
                    .and_then(rbelote_core::turn::Turn::called_color);
                match called {
                    Some(called) => {
                        ui.label("Play a card, following");
                        ui.colored_label(cards::felt_rgb(called), log::trump_label(called));
                    }
                    None => {
                        ui.label("Lead a card");
                    }
                }
            }
        }
    });

    let allowed: Option<Vec<Card>> = match &request {
        Some(Request::Play { choices, .. }) => Some(choices.clone()),
        Some(Request::Take { .. } | Request::Trump { .. }) | None => None,
    };
    let hand = game.hand(human);
    ui.add_space(6.0);
    if let Some(card) = card_row(ui, &hand, allowed.as_deref(), HAND_CARD, "hand", textures) {
        answer = Some(Answer::Play(card));
    }
    answer
}

/// The score sheet of the deal just played, shown where the hand was so that
/// the table, with its last trick, stays in view until the next deal is asked for.
fn deal_over(game: &mut Game, ui: &mut Ui, textures: &Textures) {
    let mut next = false;
    let Some(pause) = game.pause.clone() else {
        return;
    };
    ui.horizontal(|ui| {
        ui.vertical(|ui| match &pause.finished {
            None => {
                ui.heading("Nobody took");
                ui.label("The cards go back to the deck and are dealt again.");
                ui.add_space(8.0);
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("Deal again").size(18.0),
                    ))
                    .clicked()
                {
                    next = true;
                }
            }
            Some(finished) => {
                ui.heading("Deal over");
                let summary = &finished.summary;
                let (verdict, color) = match summary.mine.cmp(&summary.theirs) {
                    Ordering::Greater => ("win", Color32::from_rgb(120, 230, 120)),
                    Ordering::Less => ("lose", Color32::from_rgb(255, 110, 110)),
                    Ordering::Equal => ("tie", INK),
                };
                ui.label(
                    egui::RichText::new(format!(
                        "You {verdict}: {} points to {}",
                        summary.mine, summary.theirs
                    ))
                    .size(32.0)
                    .strong()
                    .color(color),
                );
                egui::Grid::new("summary")
                    .num_columns(2)
                    .spacing([24.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (label, value) in &summary.rows {
                            ui.label(label);
                            ui.label(value);
                            ui.end_row();
                        }
                    });
                ui.add_space(8.0);
                let label = if game.is_finished() {
                    "Final scores"
                } else {
                    "Next deal"
                };
                if ui
                    .add(egui::Button::new(egui::RichText::new(label).size(18.0)))
                    .clicked()
                {
                    next = true;
                }
                ui.add_space(4.0);
                ui.toggle_value(&mut game.show_history, "Review the tricks");
            }
        });
        if pause.finished.is_some() {
            ui.separator();
            ui.vertical(|ui| {
                for team in Team::iter() {
                    let mut won = game.team_cards(team);
                    won.sort_by_key(|card| (card.color(), card.value()));
                    ui.label(format!(
                        "{team}: {} cards, {} points",
                        won.len(),
                        game.team_points(team)
                    ));
                    card_row(
                        ui,
                        &won,
                        None,
                        SMALL_CARD,
                        &format!("final{team:?}"),
                        textures,
                    );
                }
            });
        }
    });
    if next {
        game.next_deal();
    }
}

/// Every trick of the deal in order, your own cards framed in gold.
fn history(game: &Game, ui: &mut Ui, textures: &Textures) {
    if game.tricks.is_empty() {
        ui.label("No trick played yet");
        return;
    }
    egui::ScrollArea::vertical()
        .max_height(ui.ctx().content_rect().height() * 0.7)
        .show(ui, |ui| {
            for (number, trick) in game.tricks.iter().enumerate() {
                trick_entry(game, ui, &format!("Trick {}", number + 1), trick, textures);
            }
        });
}

/// One trick: who led and won, then its cards in order, your own framed in gold.
fn trick_entry(game: &Game, ui: &mut Ui, title: &str, trick: &Trick, textures: &Textures) {
    let human = game.human();
    ui.horizontal(|ui| {
        ui.label(title);
        if let Some((leader, _)) = trick.cards.first() {
            ui.label("led by");
            ui.colored_label(name_color(game, *leader), game.name(*leader));
        }
        if let Some(winner) = trick.winner {
            ui.label("won by");
            ui.colored_label(name_color(game, winner), game.name(winner));
        }
    });
    ui.horizontal(|ui| {
        for (seat, card) in &trick.cards {
            ui.vertical(|ui| {
                ui.colored_label(name_color(game, *seat), game.name(*seat));
                let (rect, _) = ui.allocate_exact_size(SMALL_CARD, Sense::hover());
                cards::paint_card(ui.painter(), rect, *card, textures);
                if *seat == human {
                    cards::paint_highlight(ui.painter(), rect);
                }
            });
        }
    });
    ui.add_space(6.0);
}

/// Every card won so far, team by team.
fn tricks_won(game: &Game, ui: &mut Ui, textures: &Textures) {
    ui.set_min_width(640.0);
    if game.trump().is_none() {
        ui.label("Nobody took yet");
        return;
    }
    for team in Team::iter() {
        let mut won = game.team_cards(team);
        won.sort_by_key(|card| (card.color(), card.value()));
        ui.horizontal(|ui| {
            let color = if team == game.human().team() {
                ACCENT
            } else {
                INK
            };
            ui.colored_label(color, team.to_string());
            ui.label(format!(
                "{} cards, {} pts",
                won.len(),
                game.team_points(team)
            ));
        });
        if won.is_empty() {
            ui.label("nothing yet");
        } else {
            card_row(
                ui,
                &won,
                None,
                SMALL_CARD,
                &format!("won{team:?}"),
                textures,
            );
        }
        ui.add_space(4.0);
    }
}

/// A row of cards, overlapping when the row is too narrow. Returns the card clicked.
///
/// `allowed` is `None` when nothing can be clicked, else the cards that can.
fn card_row(
    ui: &mut Ui,
    cards: &[Card],
    allowed: Option<&[Card]>,
    size: egui::Vec2,
    salt: &str,
    textures: &Textures,
) -> Option<Card> {
    let lift = 12.0;
    let width = ui.available_width();
    let (area, _) = ui.allocate_exact_size(vec2(width, size.y + lift + 4.0), Sense::hover());
    if cards.is_empty() {
        return None;
    }
    let gap = 6.0;
    let natural = (size.x + gap).mul_add(cards.len() as f32, -gap);
    let step = if natural <= width {
        size.x + gap
    } else {
        (width - size.x) / (cards.len() - 1).max(1) as f32
    };
    let total = step.mul_add((cards.len() - 1) as f32, size.x);
    let start = total.mul_add(-0.5, area.center().x);
    let top = area.min.y + lift;

    let mut clicked = None;
    let mut hovered = None;
    // Hit testing first, so that a card lifted out of the row is drawn on top.
    for (index, card) in cards.iter().enumerate() {
        let rect = Rect::from_min_size(pos2((index as f32).mul_add(step, start), top), size);
        let legal = allowed.is_some_and(|allowed| allowed.contains(card));
        let response = ui.interact(rect, Id::new((salt, index)), Sense::click());
        if legal && response.hovered() {
            hovered = Some(index);
        }
        if legal && response.clicked() {
            clicked = Some(*card);
        }
    }
    let painter = ui.painter();
    for (index, card) in cards.iter().enumerate() {
        let raised = hovered == Some(index);
        let y = if raised { top - lift } else { top };
        let rect = Rect::from_min_size(pos2((index as f32).mul_add(step, start), y), size);
        cards::paint_card(painter, rect, *card, textures);
        if raised {
            cards::paint_highlight(painter, rect);
        }
        if allowed.is_some_and(|allowed| !allowed.contains(card)) {
            cards::paint_dim(painter, rect);
        }
    }
    clicked
}
