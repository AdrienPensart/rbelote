//! The game as a state machine.
//!
//! [`Game`] never asks anybody anything: it says what it needs with
//! [`Game::request`] and waits until it is handed a [`Answer`] through
//! [`Game::answer`]. Nothing blocks, so the same engine drives a terminal, a
//! browser event loop or a server. Who decides — a human, an AI, a remote
//! player — is entirely the caller's business.
//!
//! ```no_run
//! # use rbelote_core::{game::{Game, Answer, Request}, order::Order};
//! # fn pick(_: &Request) -> Answer { unimplemented!() }
//! # fn main() -> Result<(), rbelote_core::errors::BeloteErrorKind> {
//! let mut game = Game::new(Order::default())?;
//! loop {
//!     let request = game.request();
//!     game.answer(pick(&request))?;
//! }
//! # }
//! ```

use crate::card::{Card, Color};
use crate::constants::{
    ALL_CARD_POINTS, BELOTE_POINTS, LAST_TRICK_POINTS, MAX_CARDS_BY_PLAYER, MAX_PLAYERS,
};
use crate::contract::Contract;
use crate::errors::BeloteErrorKind;
use crate::hand::Hand;
use crate::hands::Hands;
use crate::order::Order;
use crate::points::Points;
use crate::position::Position;
use crate::rules;
use crate::score::{DealPoints, DealScore};
use crate::stack::Stack;
use crate::team::Team;
use crate::turn::Turn;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use tracing::{info, warn};

/// What the engine needs before it can move on.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Request {
    /// First bidding turn: does `position` take `card_returned` as trump?
    Take {
        position: Position,
        card_returned: Card,
    },
    /// Second bidding turn: which contract does `position` name?
    Trump {
        position: Position,
        card_returned: Card,
        choices: Vec<Contract>,
    },
    /// `position` must play one of `choices`.
    Play {
        position: Position,
        choices: Vec<Card>,
    },
}

impl Request {
    /// Whose decision is awaited.
    pub const fn position(&self) -> Position {
        match self {
            Self::Take { position, .. }
            | Self::Trump { position, .. }
            | Self::Play { position, .. } => *position,
        }
    }
}

/// The decision handed back to the engine, answering a [`Request`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Answer {
    Take(bool),
    Trump(Contract),
    Play(Card),
}

/// What just happened. A front-end uses these to animate, pause or redraw.
#[derive(Debug)]
pub enum Event {
    /// Another answer is needed to move on.
    Continue,
    /// The trick is complete; this position won it and leads the next one.
    TrickWon(Position),
    /// Nobody took: the hands went back to the deck and a new deal was dealt.
    Redeal,
    /// The deal was scored and a new one was dealt.
    DealEnded(Box<DealOutcome>),
}

/// The full result of a deal, once scored.
#[derive(Debug)]
pub struct DealOutcome {
    pub taker: Position,
    pub trump_color: Color,
    pub belote: Option<Team>,
    /// who won the last trick, and with it the "dix de der"
    pub last_trick: Position,
    pub points: DealPoints,
    pub score: DealScore,
}

/// The bidding, before anybody took.
#[derive(Debug, Clone, Copy)]
struct Auction {
    card_returned: Card,
    hands: Hands,
    /// index in `order` of the position being asked
    asked: usize,
    /// the first turn takes the returned card, the second names a color
    second_turn: bool,
}

/// The eight tricks, once a taker is known.
#[derive(Debug)]
struct Playing {
    taker: Position,
    trump_color: Color,
    hands: Hands,
    belote: Option<Team>,
    turn: Turn,
    /// trick number, 1 to `MAX_CARDS_BY_PLAYER`
    number: u64,
    current: Position,
    /// legal moves for `current`, kept ready so `request` cannot fail
    choices: Vec<Card>,
    attack_points: u64,
    defense_points: u64,
    attack_tricks: u64,
    /// The tricks won by each side, card by card in the order they were played.
    attack_pile: Stack,
    defense_pile: Stack,
}

#[derive(Debug)]
enum Phase {
    Auction(Auction),
    Playing(Box<Playing>),
}

#[derive(Debug)]
pub struct Game {
    points: Points,
    order: Order,
    number: u64,
    stack: Stack,
    /// Shuffles once and cuts between deals: the same seed deals the same cards.
    rng: StdRng,
    /// Who cut the stack before the current deal, nobody for the first one.
    cutter: Option<Position>,
    litige: u64,
    phase: Phase,
}

/// Five cards each, three then two, then the card turned face up.
fn deal(stack: &mut Stack, order: Order) -> Result<Auction, BeloteErrorKind> {
    let mut hands = Hands::default();
    for position in order {
        for _ in 0..3 {
            hands[position].take(stack.give_card()?)?;
        }
    }
    for position in order {
        for _ in 0..2 {
            hands[position].take(stack.give_card()?)?;
        }
        info!("{position} : {}", hands[position]);
    }
    let card_returned = stack.give_card()?;
    info!("Card returned: {card_returned}");
    info!("First bidding turn");
    Ok(Auction {
        card_returned,
        hands,
        asked: 0,
        second_turn: false,
    })
}

impl Game {
    /// Shuffles, deals, and stops on the first bid.
    pub fn new(order: Order) -> Result<Self, BeloteErrorKind> {
        Self::with_seed(order, rand::rng().random())
    }

    /// The same as [`Game::new`], with a shuffle and cuts fixed by `seed`:
    /// two games built with the same seed deal the same first hands.
    pub fn with_seed(order: Order, seed: u64) -> Result<Self, BeloteErrorKind> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut stack = Stack::random(&mut rng);
        let phase = Phase::Auction(deal(&mut stack, order)?);
        Ok(Self {
            points: Points::default(),
            order,
            number: 0,
            stack,
            rng,
            cutter: None,
            litige: 0,
            phase,
        })
    }

    /// What the engine is waiting for. Never fails: the legal moves were
    /// computed when the situation arose.
    pub fn request(&self) -> Request {
        match &self.phase {
            Phase::Auction(auction) => {
                let position = self.order[auction.asked];
                let card_returned = auction.card_returned;
                if auction.second_turn {
                    Request::Trump {
                        position,
                        card_returned,
                        choices: rules::biddable_contracts(card_returned).collect(),
                    }
                } else {
                    Request::Take {
                        position,
                        card_returned,
                    }
                }
            }
            Phase::Playing(playing) => Request::Play {
                position: playing.current,
                choices: playing.choices.clone(),
            },
        }
    }

    /// Feeds a decision back and moves on. The answer must match the pending
    /// [`Request`] and must be one of the choices it offered.
    pub fn answer(&mut self, answer: Answer) -> Result<Event, BeloteErrorKind> {
        match answer {
            Answer::Take(take) => self.answer_take(take),
            Answer::Trump(contract) => self.answer_trump(contract),
            Answer::Play(card) => self.answer_play(card),
        }
    }

    fn auction(&self) -> Result<Auction, BeloteErrorKind> {
        match &self.phase {
            Phase::Auction(auction) => Ok(*auction),
            Phase::Playing(_) => Err(BeloteErrorKind::UnexpectedAnswer(
                "bidding is over, a card is expected".to_string(),
            )),
        }
    }

    fn answer_take(&mut self, take: bool) -> Result<Event, BeloteErrorKind> {
        let mut auction = self.auction()?;
        if auction.second_turn {
            return Err(BeloteErrorKind::UnexpectedAnswer(
                "a color is expected, not a take".to_string(),
            ));
        }
        if take {
            return self.start_playing(auction, auction.card_returned.color());
        }
        info!("{} did not take at first glance", self.order[auction.asked]);
        auction.asked += 1;
        if auction.asked == MAX_PLAYERS {
            auction.asked = 0;
            auction.second_turn = true;
            info!("Second bidding turn");
        }
        self.phase = Phase::Auction(auction);
        Ok(Event::Continue)
    }

    fn answer_trump(&mut self, contract: Contract) -> Result<Event, BeloteErrorKind> {
        let mut auction = self.auction()?;
        if !auction.second_turn {
            return Err(BeloteErrorKind::UnexpectedAnswer(
                "a take is expected, not a color".to_string(),
            ));
        }
        if !rules::biddable_contracts(auction.card_returned).any(|biddable| biddable == contract) {
            return Err(BeloteErrorKind::IllegalContract(contract.to_string()));
        }
        if let Some(chosen_color) = contract.color() {
            return self.start_playing(auction, chosen_color);
        }
        auction.asked += 1;
        if auction.asked < MAX_PLAYERS {
            self.phase = Phase::Auction(auction);
            return Ok(Event::Continue);
        }
        // nobody took: the hands go back to the deck and it is dealt again
        self.stack.append_card(auction.card_returned)?;
        for position in self.order {
            for card in auction.hands[position].into_iter().flatten() {
                self.stack.append_card(card)?;
            }
        }
        self.next_deal()?;
        Ok(Event::Redeal)
    }

    /// The taker takes the returned card, everybody is dealt up to eight.
    fn start_playing(
        &mut self,
        auction: Auction,
        trump_color: Color,
    ) -> Result<Event, BeloteErrorKind> {
        let taker = self.order[auction.asked];
        let mut hands = auction.hands;
        info!(
            "{taker} for color {trump_color}, we give him {}",
            auction.card_returned
        );
        hands[taker].take(auction.card_returned)?;
        for position in self.order {
            let count = if position == taker { 2 } else { 3 };
            info!("Giving {position} {count} more cards");
            for _ in 0..count {
                hands[position].take(self.stack.give_card()?)?;
            }
        }

        let mut belote = None;
        for position in self.order {
            if hands[position].belote_rebelote(trump_color) {
                belote = Some(position.team());
            }
        }

        let current = self.order.first();
        let turn = Turn::new(1, self.order);
        let choices = rules::choices(&hands[current], &current, &turn, trump_color)?;
        self.phase = Phase::Playing(Box::new(Playing {
            taker,
            trump_color,
            hands,
            belote,
            turn,
            number: 1,
            current,
            choices,
            attack_points: 0,
            defense_points: 0,
            attack_tricks: 0,
            attack_pile: Stack::default(),
            defense_pile: Stack::default(),
        }));
        Ok(Event::Continue)
    }

    fn answer_play(&mut self, card: Card) -> Result<Event, BeloteErrorKind> {
        let (master, number, piled) = {
            let Phase::Playing(playing) = &mut self.phase else {
                return Err(BeloteErrorKind::UnexpectedAnswer(
                    "bidding is not over, no card is expected".to_string(),
                ));
            };
            if !playing.choices.contains(&card) {
                return Err(BeloteErrorKind::IllegalCard(card.to_string()));
            }
            let current = playing.current;
            if playing.hands[current].give(&card).is_none() {
                return Err(BeloteErrorKind::InvalidCase(
                    "cannot give chosen card".to_string(),
                ));
            }
            if !playing.turn.put(playing.trump_color, current, &card) {
                return Err(BeloteErrorKind::InvalidCase(format!(
                    "{current} played twice in the same turn"
                )));
            }

            if !playing.turn.finished() {
                let next = current.next();
                let choices = rules::choices(
                    &playing.hands[next],
                    &next,
                    &playing.turn,
                    playing.trump_color,
                )?;
                playing.current = next;
                playing.choices = choices;
                return Ok(Event::Continue);
            }

            let Some(cards) = playing.turn.take() else {
                return Err(BeloteErrorKind::InvalidCase(
                    "Cannot take turn cards".to_string(),
                ));
            };
            let master = playing.turn.master_position();
            info!("Fold master is player {master}");
            let attacking = playing.taker.team() == playing.turn.master_team();
            if attacking {
                playing.attack_tricks += 1;
            }
            // RULE: each side keeps its tricks in one pile, in the order they fell
            let pile = if attacking {
                &mut playing.attack_pile
            } else {
                &mut playing.defense_pile
            };
            for card in cards {
                let points = card.points(playing.trump_color);
                warn!("{card} : {points} points");
                if attacking {
                    playing.attack_points += points;
                } else {
                    playing.defense_points += points;
                }
                pile.append_card(card)?;
            }
            info!("New attack points = {}", playing.attack_points);
            info!("New defense points = {}", playing.defense_points);
            let piled = playing.attack_pile.len() + playing.defense_pile.len();
            (master, playing.number, piled)
        };

        let expected = usize::try_from(number).unwrap_or(usize::MAX) * MAX_PLAYERS;
        if piled != expected {
            return Err(BeloteErrorKind::InvalidCase(format!(
                "bad piles length {piled} it should be {expected}"
            )));
        }

        if number == MAX_CARDS_BY_PLAYER as u64 {
            return self.finish_deal(master);
        }
        self.next_trick(master)?;
        Ok(Event::TrickWon(master))
    }

    /// The winner of the trick leads the next one.
    fn next_trick(&mut self, master: Position) -> Result<(), BeloteErrorKind> {
        let order = self.order;
        let Phase::Playing(playing) = &mut self.phase else {
            return Err(BeloteErrorKind::InvalidCase("not playing".to_string()));
        };
        playing.number += 1;
        playing.turn = Turn::new(playing.number, order);
        playing.current = master;
        playing.choices = rules::choices(
            &playing.hands[master],
            &master,
            &playing.turn,
            playing.trump_color,
        )?;
        Ok(())
    }

    fn finish_deal(&mut self, last_trick: Position) -> Result<Event, BeloteErrorKind> {
        let Phase::Playing(playing) = &self.phase else {
            return Err(BeloteErrorKind::InvalidCase("not playing".to_string()));
        };
        let (taker, trump_color, belote) = (playing.taker, playing.trump_color, playing.belote);
        let mut attack = playing.attack_points;
        let mut defense = playing.defense_points;
        let attack_tricks = playing.attack_tricks;
        let (attack_pile, defense_pile) = (playing.attack_pile, playing.defense_pile);

        // "dix de der", to whoever took the last trick
        if last_trick.team() == taker.team() {
            attack += LAST_TRICK_POINTS;
        } else {
            defense += LAST_TRICK_POINTS;
        }
        if attack + defense != ALL_CARD_POINTS {
            return Err(BeloteErrorKind::InvalidCase(format!(
                "bad points total : {attack} + {defense} should be {ALL_CARD_POINTS}"
            )));
        }

        let taker_team = taker.team();
        let (belote_attack, belote_defense) = belote.map_or((0, 0), |team| {
            if team == taker_team {
                (BELOTE_POINTS, 0)
            } else {
                (0, BELOTE_POINTS)
            }
        });
        let points = DealPoints {
            attack,
            defense,
            attack_tricks,
            belote_attack,
            belote_defense,
            litige: self.litige,
        };
        let score = points.score();
        info!("{points:?} => {score:?}");

        self.litige = score.litige;
        self.points[taker_team] += score.attack;
        self.points[taker_team.other()] += score.defense;

        let outcome = DealOutcome {
            taker,
            trump_color,
            belote,
            last_trick,
            points,
            score,
        };
        // RULE: the tricks go back on the stack as two piles, the attack's then
        // the defense's, and are never shuffled
        for card in attack_pile.cards().chain(defense_pile.cards()) {
            self.stack.append_card(card)?;
        }
        self.next_deal()?;
        Ok(Event::DealEnded(Box::new(outcome)))
    }

    /// Rotates the deal, cuts (belote never reshuffles) and deals again.
    ///
    /// RULE: the dealer of the deal that just ended cuts the stack for the next one.
    fn next_deal(&mut self) -> Result<(), BeloteErrorKind> {
        let cutter = self.order.last();
        self.order.rotate();
        self.number += 1;
        self.stack.cut(&mut self.rng)?;
        self.cutter = Some(cutter);
        self.phase = Phase::Auction(deal(&mut self.stack, self.order)?);
        Ok(())
    }

    // ----- what a front-end needs to draw the table -----

    pub const fn points(&self) -> Points {
        self.points
    }
    pub const fn number(&self) -> u64 {
        self.number
    }
    pub const fn order(&self) -> Order {
        self.order
    }
    /// Who cut the stack before the current deal: the previous dealer,
    /// nobody for the first deal of the game.
    pub const fn cutter(&self) -> Option<Position> {
        self.cutter
    }
    pub const fn litige(&self) -> u64 {
        self.litige
    }
    pub fn hand(&self, position: Position) -> &Hand {
        match &self.phase {
            Phase::Auction(auction) => &auction.hands[position],
            Phase::Playing(playing) => &playing.hands[position],
        }
    }
    /// The face up card, while the bidding lasts.
    pub const fn card_returned(&self) -> Option<Card> {
        match &self.phase {
            Phase::Auction(auction) => Some(auction.card_returned),
            Phase::Playing(_) => None,
        }
    }
    /// The trump, once somebody took.
    pub const fn trump_color(&self) -> Option<Color> {
        match &self.phase {
            Phase::Auction(_) => None,
            Phase::Playing(playing) => Some(playing.trump_color),
        }
    }
    pub const fn taker(&self) -> Option<Position> {
        match &self.phase {
            Phase::Auction(_) => None,
            Phase::Playing(playing) => Some(playing.taker),
        }
    }
    /// The cards on the table, and who is master of them.
    pub const fn turn(&self) -> Option<&Turn> {
        match &self.phase {
            Phase::Auction(_) => None,
            Phase::Playing(playing) => Some(&playing.turn),
        }
    }
}

#[cfg(test)]
mod game_tests {
    use super::{Answer, Event, Game, Phase, Request};
    use crate::card::Card;
    use crate::constants::{ALL_CARD_POINTS, MAX_CARDS, MAX_CARDS_BY_PLAYER, MIN_CUT};
    use crate::contract::Contract;
    use crate::order::Order;
    use crate::position::Position;
    use crate::stack::Stack;
    use strum::VariantArray;

    /// A policy that always takes and always plays its first legal card, so a
    /// deal is always played out.
    fn eager(request: &Request) -> Answer {
        match request {
            Request::Take { .. } => Answer::Take(true),
            Request::Trump { choices, .. } => choices
                .iter()
                .find(|contract| contract.color().is_some())
                .map_or(Answer::Trump(Contract::Pass), |contract| {
                    Answer::Trump(*contract)
                }),
            Request::Play { choices, .. } => choices
                .first()
                .map_or_else(|| panic!("no legal card"), |card| Answer::Play(*card)),
        }
    }

    #[test]
    fn a_whole_game_can_be_driven_from_outside() {
        let Ok(mut game) = Game::new(Order::default()) else {
            panic!("cannot start");
        };
        let mut deals = 0;
        let mut tricks = 0;
        while deals < 20 {
            let request = game.request();
            // the engine only ever asks the player whose turn it is
            let Ok(event) = game.answer(eager(&request)) else {
                panic!("engine refused a legal answer to {request:?}");
            };
            match event {
                Event::Continue => {}
                Event::TrickWon(_) => tricks += 1,
                Event::Redeal => panic!("an eager player always takes"),
                Event::DealEnded(outcome) => {
                    assert_eq!(
                        outcome.points.attack + outcome.points.defense,
                        ALL_CARD_POINTS,
                        "deal {deals} does not add up"
                    );
                    assert!(outcome.points.attack_tricks <= MAX_CARDS_BY_PLAYER as u64);
                    deals += 1;
                }
            }
        }
        // eight tricks per deal, the last of which ends the deal instead
        assert_eq!(tricks, deals * (MAX_CARDS_BY_PLAYER as u64 - 1));
    }

    #[test]
    fn the_tricks_go_back_as_two_team_piles_then_get_cut() {
        let Ok(mut game) = Game::with_seed(Order::default(), 8) else {
            panic!("cannot start");
        };
        let mut last_trick_cards: Vec<Card> = Vec::new();
        loop {
            let request = game.request();
            let answer = eager(&request);
            let (attack_pile, defense_pile, taker, last_trick) = match &game.phase {
                Phase::Playing(playing) => (
                    playing.attack_pile,
                    playing.defense_pile,
                    playing.taker,
                    playing.number == MAX_CARDS_BY_PLAYER as u64,
                ),
                Phase::Auction(_) => (Stack::default(), Stack::default(), Position::North, false),
            };
            if let (Answer::Play(card), true) = (answer, last_trick) {
                last_trick_cards.push(card);
            }
            let Ok(event) = game.answer(answer) else {
                panic!("engine refused a legal answer to {request:?}");
            };
            let Event::DealEnded(outcome) = event else {
                continue;
            };
            // the last trick joins the pile of the side that won it
            let mut attack: Vec<Card> = attack_pile.cards().collect();
            let mut defense: Vec<Card> = defense_pile.cards().collect();
            if outcome.last_trick.team() == taker.team() {
                attack.extend(&last_trick_cards);
            } else {
                defense.extend(&last_trick_cards);
            }
            let expected: Vec<Card> = attack.iter().chain(defense.iter()).copied().collect();
            assert_eq!(expected.len(), MAX_CARDS);

            // rebuild the cut stack: what is left of it, plus what the new deal popped
            let hands: Vec<Vec<Card>> = game
                .order()
                .into_iter()
                .map(|position| (*game.hand(position)).into_iter().flatten().collect())
                .collect();
            let mut popped: Vec<Card> = Vec::new();
            for hand in &hands {
                popped.extend(hand.iter().take(3));
            }
            for hand in &hands {
                popped.extend(hand.iter().skip(3).take(2));
            }
            popped.push(game.card_returned().expect("the bidding is open"));
            let mut rebuilt: Vec<Card> = game.stack.cards().collect();
            rebuilt.extend(popped.iter().rev());
            assert_eq!(rebuilt.len(), MAX_CARDS);

            let rotation = (0..MAX_CARDS).find(|&depth| {
                let mut turned = expected.clone();
                turned.rotate_left(depth);
                turned == rebuilt
            });
            let Some(depth) = rotation else {
                panic!("the stack is not the two piles, cut once");
            };
            assert!((MIN_CUT..=MAX_CARDS - MIN_CUT).contains(&depth), "{depth}");
            break;
        }
    }

    #[test]
    fn the_previous_dealer_cuts_for_the_next_deal() {
        let Ok(mut game) = Game::with_seed(Order::default(), 3) else {
            panic!("cannot start");
        };
        assert_eq!(game.cutter(), None, "the first stack is shuffled, not cut");
        let mut previous_dealer = game.order().last();
        let mut deals = 0;
        while deals < 4 {
            let request = game.request();
            let Ok(event) = game.answer(eager(&request)) else {
                panic!("engine refused a legal answer to {request:?}");
            };
            if matches!(event, Event::DealEnded(_)) {
                deals += 1;
                assert_eq!(game.cutter(), Some(previous_dealer));
                assert_ne!(game.order().last(), previous_dealer, "the deal moved on");
                previous_dealer = game.order().last();
            }
        }
    }

    #[test]
    fn the_same_seed_deals_the_same_first_hands() {
        let (Ok(first), Ok(second)) = (
            Game::with_seed(Order::default(), 42),
            Game::with_seed(Order::default(), 42),
        ) else {
            panic!("cannot start");
        };
        for position in Position::VARIANTS {
            assert_eq!(
                first.hand(*position).to_string(),
                second.hand(*position).to_string()
            );
        }
        assert_eq!(first.card_returned(), second.card_returned());
        let Ok(other) = Game::with_seed(Order::default(), 43) else {
            panic!("cannot start");
        };
        let same = Position::VARIANTS
            .iter()
            .all(|position| first.hand(*position).to_string() == other.hand(*position).to_string());
        assert!(!same, "another seed deals other cards");
    }

    #[test]
    fn the_engine_refuses_an_answer_it_did_not_ask_for() {
        let Ok(mut game) = Game::new(Order::default()) else {
            panic!("cannot start");
        };
        // bidding just opened: a card makes no sense yet
        let Request::Take { card_returned, .. } = game.request() else {
            panic!("expected a take request");
        };
        assert!(game.answer(Answer::Play(card_returned)).is_err());
        assert!(game.answer(Answer::Trump(Contract::Pass)).is_err());
        assert!(game.answer(Answer::Take(true)).is_ok());
    }

    #[test]
    fn the_engine_refuses_an_illegal_card() {
        let Ok(mut game) = Game::new(Order::default()) else {
            panic!("cannot start");
        };
        let Ok(_) = game.answer(Answer::Take(true)) else {
            panic!("cannot take");
        };
        let Request::Play { position, choices } = game.request() else {
            panic!("expected a card request");
        };
        // a card held but not among the legal moves must be refused
        let illegal = game
            .hand(position)
            .into_iter()
            .flatten()
            .find(|card| !choices.contains(card));
        if let Some(illegal) = illegal {
            assert!(game.answer(Answer::Play(illegal)).is_err());
        }
        // and a card that is not in the hand at all, either
        let outsider = choices.first().copied();
        if let Some(card) = outsider {
            assert!(game.answer(Answer::Play(card)).is_ok());
            assert!(game.answer(Answer::Play(card)).is_err());
        }
    }
}
