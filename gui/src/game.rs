//! Driving the engine from a window: bots answer on a timer, humans by clicking.

use crate::log;
use egui::Context;
use rbelote_core::bot;
use rbelote_core::card::{Card, Color, Value};
use rbelote_core::constants::MAX_PLAYERS;
use rbelote_core::contract::Contract;
use rbelote_core::errors::BeloteErrorKind;
use rbelote_core::game::{Answer, Event, Game as Engine, Request};
use rbelote_core::order::Order;
use rbelote_core::position::Position;
use rbelote_core::team::Team;
use std::cmp::Ordering;
use std::time::{Duration, Instant};
use strum::{IntoEnumIterator, VariantArray};

/// Everything chosen on the setup screen.
#[derive(Clone, Debug)]
pub struct Settings {
    /// The seat played by the human.
    pub seat: Position,
    /// The game stops after this many scored deals…
    pub deals: u64,
    /// …or as soon as a team reaches this many points.
    pub target: u64,
    /// Bots never take, so the human plays every contract.
    pub attack: bool,
    /// Answer by itself when only one card is legal.
    pub auto: bool,
    /// Seconds a finished trick stays on the table before it is folded.
    pub trick_delay: f32,
    /// Seconds a bot waits before answering, so that a human can follow.
    pub bot_delay: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            seat: Position::South,
            deals: 10,
            target: 1000,
            attack: false,
            auto: false,
            trick_delay: 2.0,
            bot_delay: 0.8,
        }
    }
}

/// Who a piece of text talks about, so that a name can be painted in its side's color.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tone {
    Plain,
    Taker,
    Ally,
}

/// A run of text in one tone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub text: String,
    pub tone: Tone,
}

/// One line of the table talk.
pub type Line = Vec<Span>;

/// What a seat said during the auction.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Bid {
    #[default]
    Silent,
    Pass,
    Took(Color),
}

/// The cards on the table, in the order they were put down.
#[derive(Clone, Debug, Default)]
pub struct Trick {
    pub cards: Vec<(Position, Card)>,
    pub winner: Option<Position>,
}

/// What a finished deal was worth, ready to be displayed.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub rows: Vec<(String, String)>,
    /// What the human's team scored on the deal.
    pub mine: u64,
    /// What the other team scored on the deal.
    pub theirs: u64,
}

/// The deal just scored, kept on the table while its score sheet is read.
#[derive(Clone, Debug)]
pub struct Finished {
    pub taker: Position,
    pub trump: Color,
    pub summary: Summary,
}

/// The table stands still: a deal just ended, or nobody took it.
#[derive(Clone, Debug)]
pub struct Pause {
    /// Who dealt the deal that just ended; the engine has already moved on.
    pub dealer: Position,
    /// The score sheet, or nothing when nobody took.
    pub finished: Option<Finished>,
}

/// What the table needs to know about the deal on it.
#[derive(Copy, Clone, Debug)]
pub struct DealInfo {
    pub dealer: Position,
    pub taker: Option<Position>,
    pub trump: Option<Color>,
    /// The card turned up, while the bidding lasts.
    pub returned: Option<Card>,
}

/// The seat across the table, in the same team.
pub const fn partner(position: Position) -> Position {
    position.next().next()
}

/// A seat's index around the table, in playing order.
pub fn index(position: Position) -> usize {
    Position::VARIANTS
        .iter()
        .position(|variant| *variant == position)
        .unwrap_or_default()
}

/// Who deals: the last seat in the playing order.
fn dealer(order: Order) -> Position {
    order.into_iter().last().unwrap_or(Position::West)
}

/// Where a card goes in a hand: suits in alternating colors, then by rank,
/// in the trump order for the trump suit.
fn rank(card: Card, trump: Option<Color>) -> (usize, u8) {
    let plain = match card.value() {
        Value::_7 => 0,
        Value::_8 => 1,
        Value::_9 => 2,
        Value::_10 => 3,
        Value::Jack => 4,
        Value::Queen => 5,
        Value::King => 6,
        Value::As => 7,
    };
    let within = match trump {
        Some(trump) if trump == card.color() => card.power(trump),
        Some(_) | None => plain,
    };
    let color = Color::iter()
        .position(|color| color == card.color())
        .unwrap_or_default();
    (color, within)
}

pub struct Game {
    pub settings: Settings,
    pub engine: Engine,
    pub log: Vec<Line>,
    /// What each seat said during the auction, in table order.
    pub bids: [Bid; MAX_PLAYERS],
    pub trick: Trick,
    pub last_trick: Option<Trick>,
    /// Every trick of the deal so far, in order.
    pub tricks: Vec<Trick>,
    /// Set while a score sheet, or a cancelled deal, is being read.
    pub pause: Option<Pause>,
    pub error: Option<String>,
    deals_played: u64,
    /// Whether the window listing the tricks won is open.
    pub show_tricks: bool,
    /// Whether the window showing the last trick is open.
    pub show_last: bool,
    /// Whether the table talk panel is shown.
    pub show_log: bool,
    /// Whether the window replaying the deal trick by trick is open.
    pub show_history: bool,
    /// Until when the last trick stays on the table.
    last_trick_until: Instant,
    next_bot_at: Instant,
}

impl Game {
    pub fn new(settings: &Settings) -> Result<Self, BeloteErrorKind> {
        let engine = Engine::new(Order::default())?;
        let mut game = Self {
            settings: settings.clone(),
            engine,
            log: Vec::new(),
            bids: [Bid::Silent; MAX_PLAYERS],
            trick: Trick::default(),
            last_trick: None,
            tricks: Vec::new(),
            pause: None,
            error: None,
            deals_played: 0,
            show_tricks: false,
            show_last: false,
            show_log: false,
            show_history: false,
            last_trick_until: Instant::now(),
            next_bot_at: Instant::now(),
        };
        game.say(&log::deal_started(
            game.deals_left(),
            &game.name(game.deal().dealer),
        ));
        Ok(game)
    }

    pub const fn human(&self) -> Position {
        self.settings.seat
    }

    /// What a seat is called. A method, so that the table is the one place
    /// deciding how seats are named.
    #[allow(clippy::unused_self)]
    pub fn name(&self, position: Position) -> String {
        position.to_string()
    }

    pub const fn paused(&self) -> bool {
        self.pause.is_some()
    }

    /// The deal on the table: the one being played, or the one just scored
    /// while its score sheet is being read.
    pub fn deal(&self) -> DealInfo {
        self.pause.as_ref().map_or_else(
            || DealInfo {
                dealer: dealer(self.engine.order()),
                taker: self.engine.taker(),
                trump: self.engine.trump_color(),
                returned: self.engine.card_returned(),
            },
            |pause| DealInfo {
                dealer: pause.dealer,
                taker: pause.finished.as_ref().map(|finished| finished.taker),
                trump: pause.finished.as_ref().map(|finished| finished.trump),
                returned: None,
            },
        )
    }

    pub fn taker(&self) -> Option<Position> {
        self.deal().taker
    }

    /// The taker's partner, once somebody took.
    pub fn ally(&self) -> Option<Position> {
        self.taker().map(partner)
    }

    pub fn trump(&self) -> Option<Color> {
        self.deal().trump
    }

    /// Card points taken so far by a team in this deal.
    pub fn team_points(&self, team: Team) -> u64 {
        let Some(trump) = self.trump() else {
            return 0;
        };
        self.tricks
            .iter()
            .filter(|trick| trick.winner.is_some_and(|winner| winner.team() == team))
            .flat_map(|trick| trick.cards.iter())
            .map(|(_, card)| card.points(trump))
            .sum()
    }

    /// Every card a team won so far in this deal.
    pub fn team_cards(&self, team: Team) -> Vec<Card> {
        self.tricks
            .iter()
            .filter(|trick| trick.winner.is_some_and(|winner| winner.team() == team))
            .flat_map(|trick| trick.cards.iter().map(|(_, card)| *card))
            .collect()
    }

    /// A seat's hand, sorted for display. Empty while a score sheet is read:
    /// the engine has already dealt the next one.
    pub fn hand(&self, position: Position) -> Vec<Card> {
        if self.paused() {
            return Vec::new();
        }
        let trump = self.trump();
        let mut hand: Vec<Card> = self.engine.hand(position).into_iter().flatten().collect();
        hand.sort_by_key(|card| rank(*card, trump));
        hand
    }

    /// Whether a seat holds the king and queen of trump.
    pub fn belote(&self, position: Position) -> bool {
        !self.paused()
            && self
                .trump()
                .is_some_and(|trump| self.engine.hand(position).belote_rebelote(trump))
    }

    pub const fn deals_left(&self) -> u64 {
        self.settings.deals.saturating_sub(self.deals_played)
    }

    pub const fn deals_played(&self) -> u64 {
        self.deals_played
    }

    pub fn is_finished(&self) -> bool {
        self.deals_left() == 0
            || Team::iter().any(|team| self.engine.points()[team] >= self.settings.target)
    }

    /// The team ahead, once the game is over.
    pub fn winner(&self) -> Option<Team> {
        let points = self.engine.points();
        match points[Team::NorthSouth].cmp(&points[Team::WestEast]) {
            Ordering::Greater => Some(Team::NorthSouth),
            Ordering::Less => Some(Team::WestEast),
            Ordering::Equal => None,
        }
    }

    /// The question the engine is waiting on, while the table is live.
    pub fn pending(&self) -> Option<Request> {
        if self.paused() || self.is_finished() {
            None
        } else {
            Some(self.engine.request())
        }
    }

    /// The question waiting for the human, if any.
    pub fn human_request(&self) -> Option<Request> {
        self.pending()
            .filter(|request| request.position() == self.human())
    }

    /// Whether the last trick is still on the table, and for how long.
    pub fn last_trick_showing(&self) -> Option<Duration> {
        self.last_trick_until.checked_duration_since(Instant::now())
    }

    /// What to write next to a name: the trump once somebody took, else
    /// what that seat bid so far.
    pub fn bid_label(&self, seat: Position) -> Option<String> {
        if let (Some(taker), Some(trump)) = (self.taker(), self.trump()) {
            return (taker == seat).then(|| trump.to_string());
        }
        match self.bids.get(index(seat)).copied().unwrap_or_default() {
            Bid::Silent => None,
            Bid::Pass => Some("passes".to_string()),
            Bid::Took(color) => Some(color.to_string()),
        }
    }

    /// Read the score sheet, then let the next deal start.
    pub fn next_deal(&mut self) {
        self.pause = None;
        self.show_history = false;
        self.trick = Trick::default();
        self.last_trick = None;
        self.tricks.clear();
        self.bids = [Bid::Silent; MAX_PLAYERS];
        if self.is_finished() {
            self.say(&log::game_over());
        } else {
            self.say(&log::deal_started(
                self.deals_left(),
                &self.name(self.deal().dealer),
            ));
        }
        self.next_bot_at = Instant::now() + self.delay();
    }

    fn delay(&self) -> Duration {
        Duration::from_secs_f32(self.settings.bot_delay.max(0.0))
    }

    /// Names worth painting in a color right now.
    fn marks(&self) -> Vec<(String, Tone)> {
        [(self.taker(), Tone::Taker), (self.ally(), Tone::Ally)]
            .into_iter()
            .filter_map(|(seat, tone)| seat.map(|seat| (self.name(seat), tone)))
            .collect()
    }

    fn say(&mut self, text: &str) {
        let marks = self.marks();
        self.log.push(tag(text, &marks));
    }

    fn set_bid(&mut self, seat: Position, bid: Bid) {
        if let Some(slot) = self.bids.get_mut(index(seat)) {
            *slot = bid;
        }
    }

    /// Hand an answer to the engine, whoever it comes from.
    pub fn answer(&mut self, answer: Answer) {
        let request = self.engine.request();
        let dealer = dealer(self.engine.order());
        let event = match self.engine.answer(answer) {
            Ok(event) => event,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        let seat = request.position();
        match answer {
            Answer::Take(true) => {
                if let Request::Take { card_returned, .. } = &request {
                    self.set_bid(seat, Bid::Took(card_returned.color()));
                }
            }
            Answer::Take(false) | Answer::Trump(Contract::Pass) => {
                self.set_bid(seat, Bid::Pass);
            }
            Answer::Trump(contract) => {
                if let Some(color) = contract.color() {
                    self.set_bid(seat, Bid::Took(color));
                }
            }
            Answer::Play(card) => self.trick.cards.push((seat, card)),
        }
        self.say(&log::answered(&self.name(seat), &request, answer));

        let mut turn_won = false;
        match event {
            Event::Continue => {}
            Event::TrickWon(winner) => {
                self.fold(winner);
                turn_won = true;
            }
            Event::Redeal => {
                self.say(&log::redeal());
                self.pause = Some(Pause {
                    dealer,
                    finished: None,
                });
            }
            Event::DealEnded(outcome) => {
                self.fold(outcome.last_trick);
                self.deals_played += 1;
                let summary = log::summary(&outcome, self.engine.points(), self.human(), |seat| {
                    seat.to_string()
                });
                let line = log::deal_scored(&outcome, &self.name(outcome.taker));
                // The engine has already moved on, so the table no longer knows
                // the taker: mark the names by hand.
                let marks = vec![
                    (self.name(outcome.taker), Tone::Taker),
                    (self.name(partner(outcome.taker)), Tone::Ally),
                ];
                self.log.push(tag(&line, &marks));
                self.pause = Some(Pause {
                    dealer,
                    finished: Some(Finished {
                        taker: outcome.taker,
                        trump: outcome.trump_color,
                        summary,
                    }),
                });
            }
        }
        // A finished trick stays on the table before the bots go on.
        let wait = if turn_won {
            self.delay()
                .max(Duration::from_secs_f32(self.settings.trick_delay.max(0.0)))
        } else {
            self.delay()
        };
        self.next_bot_at = Instant::now() + wait;
    }

    /// Put the trick on the table away, won by `winner`.
    fn fold(&mut self, winner: Position) {
        let mut done = std::mem::take(&mut self.trick);
        done.winner = Some(winner);
        let cards = done.cards.len();
        self.tricks.push(done.clone());
        self.last_trick = Some(done);
        self.last_trick_until =
            Instant::now() + Duration::from_secs_f32(self.settings.trick_delay.max(0.0));
        self.say(&log::trick_won(&self.name(winner), cards));
    }

    /// What the human does not even need to be asked.
    const fn automatic(&self, request: &Request) -> Option<Answer> {
        match request {
            Request::Play { choices, .. } => match choices.as_slice() {
                [only] if self.settings.auto => Some(Answer::Play(*only)),
                _ => None,
            },
            Request::Take { .. } | Request::Trump { .. } => None,
        }
    }

    fn bot_answer(&self, request: &Request) -> Result<Answer, BeloteErrorKind> {
        if self.settings.attack {
            match request {
                Request::Take { .. } => return Ok(Answer::Take(false)),
                Request::Trump { .. } => return Ok(Answer::Trump(Contract::Pass)),
                Request::Play { .. } => {}
            }
        }
        bot::answer(&self.engine, request)
    }

    /// Let the bots play, as far as the clock allows. Called once per frame.
    pub fn tick(&mut self, ctx: &Context) {
        loop {
            if self.error.is_some() {
                return;
            }
            let Some(request) = self.pending() else {
                return;
            };
            if request.position() == self.human() {
                match self.automatic(&request) {
                    Some(answer) => {
                        self.answer(answer);
                        continue;
                    }
                    None => return,
                }
            }
            let now = Instant::now();
            if now < self.next_bot_at {
                ctx.request_repaint_after(self.next_bot_at - now);
                return;
            }
            match self.bot_answer(&request) {
                Ok(answer) => self.answer(answer),
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            }
        }
    }
}

/// Split a sentence so that every name in `marks` gets its tone.
fn tag(text: &str, marks: &[(String, Tone)]) -> Line {
    let mut line = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let hit = marks
            .iter()
            .filter(|(name, _)| !name.is_empty())
            .filter_map(|(name, tone)| rest.find(name.as_str()).map(|at| (at, name, *tone)))
            .min_by_key(|(at, ..)| *at);
        let Some((at, name, tone)) = hit else {
            line.push(Span {
                text: rest.to_string(),
                tone: Tone::Plain,
            });
            break;
        };
        let (before, after) = rest.split_at(at);
        if !before.is_empty() {
            line.push(Span {
                text: before.to_string(),
                tone: Tone::Plain,
            });
        }
        line.push(Span {
            text: name.clone(),
            tone,
        });
        rest = after.get(name.len()..).unwrap_or_default();
    }
    line
}

#[cfg(test)]
mod tests {
    use super::{tag, Game, Settings, Span, Tone};
    use rbelote_core::bot;
    use rbelote_core::position::Position;
    use strum::IntoEnumIterator;

    /// A whole game where the human answers at random, with instant bots.
    #[test]
    fn a_game_runs_to_the_end_from_every_seat() {
        for seat in Position::iter() {
            let settings = Settings {
                seat,
                deals: 3,
                target: 100_000,
                bot_delay: 0.0,
                trick_delay: 0.0,
                ..Settings::default()
            };
            let Ok(mut game) = Game::new(&settings) else {
                panic!("cannot start");
            };
            let ctx = egui::Context::default();
            let mut guard = 0;
            while !game.is_finished() {
                guard += 1;
                assert!(guard < 10_000, "the game does not end from seat {seat}");
                game.tick(&ctx);
                assert_eq!(game.error, None);
                if game.paused() {
                    game.next_deal();
                    continue;
                }
                if let Some(request) = game.human_request() {
                    let Ok(answer) = bot::random_answer(&request) else {
                        panic!("no legal answer");
                    };
                    game.answer(answer);
                }
            }
            // The last score sheet still waits to be read when the human plays last.
            if game.paused() {
                game.next_deal();
            }
            let text = |line: &Vec<Span>| {
                line.iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>()
            };
            assert!(game.log.iter().any(|line| text(line).contains("Game over")));
            assert_eq!(game.deals_played(), 3);
        }
    }

    #[test]
    fn the_game_stops_at_the_target_score() {
        let settings = Settings {
            deals: 1_000,
            target: 1,
            bot_delay: 0.0,
            trick_delay: 0.0,
            attack: false,
            ..Settings::default()
        };
        let Ok(mut game) = Game::new(&settings) else {
            panic!("cannot start");
        };
        let ctx = egui::Context::default();
        let mut guard = 0;
        while !game.is_finished() {
            guard += 1;
            assert!(guard < 10_000);
            game.tick(&ctx);
            if game.paused() {
                game.next_deal();
            } else if let Some(request) = game.human_request() {
                let Ok(answer) = bot::answer(&game.engine, &request) else {
                    panic!("no answer");
                };
                game.answer(answer);
            }
        }
        assert!(game.deals_played() < 1_000);
    }

    #[test]
    fn names_get_their_tone() {
        let marks = vec![
            ("North".to_string(), Tone::Taker),
            ("East".to_string(), Tone::Ally),
        ];
        let line = tag("East follows North, North wins", &marks);
        let tones: Vec<(&str, Tone)> = line
            .iter()
            .map(|span| (span.text.as_str(), span.tone))
            .collect();
        assert_eq!(
            tones,
            vec![
                ("East", Tone::Ally),
                (" follows ", Tone::Plain),
                ("North", Tone::Taker),
                (", ", Tone::Plain),
                ("North", Tone::Taker),
                (" wins", Tone::Plain),
            ]
        );
        assert_eq!(tag("Everyone passed", &marks).len(), 1);
    }
}
