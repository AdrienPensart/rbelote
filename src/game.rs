use crate::card::Card;
use crate::constants::{MAX_CARDS_BY_PLAYER, MAX_PLAYERS};
use crate::distribution::Distribution;
use crate::errors::BeloteErrorKind;
use crate::hands::Hands;
use crate::initial::Initial;
use crate::order::Order;
use crate::players::Players;
use crate::playing::Playing;
use crate::points::Points;
use crate::stack::Stack;
use crate::team::Team;
use crate::traits::BeloteRebelote;
use crate::traits::PlayingOrder;
use crate::turn::Turn;
use derive_more::Constructor;
use inquire::Select;
use rand::seq::IteratorRandom;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Debug, Constructor)]
pub struct Game<State> {
    players: Players,
    points: Points,
    state: State,
}

impl<State> Game<State> {
    pub const fn state(&self) -> &State {
        &self.state
    }

    pub const fn full_random(&self) -> bool {
        self.players.full_random()
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub const fn points(&self) -> Points {
        self.points
    }

    pub const fn players(&self) -> Players {
        self.players
    }
}

impl PlayingOrder for Game<Initial> {
    fn order(&self) -> Order {
        self.state.order()
    }
}

impl Game<Initial> {
    pub const fn number(&self) -> u64 {
        self.state.number()
    }

    pub fn default(players: Players, order: Order) -> Self {
        Self {
            players,
            points: Points::default(),
            state: Initial::new(order),
        }
    }

    pub fn distribute(mut self) -> Game<Distribution> {
        let mut hands = Hands::default();

        for position in self.order() {
            for card in self.state_mut().stack_mut().next_three_cards() {
                hands[position].take(*card);
            }
        }
        for position in self.order() {
            for card in self.state_mut().stack_mut().next_two_cards() {
                hands[position].take(*card);
            }
            println!("{position} : {}", hands[position]);
        }
        Game::new(
            self.players,
            self.points,
            Distribution::new(hands, self.state),
        )
    }
}

impl PlayingOrder for Game<Playing> {
    fn order(&self) -> Order {
        self.state().order()
    }
}

pub enum PlayingResult {
    NextGame(Game<Initial>),
    Interrupted,
}

impl Game<Playing> {
    pub fn play(mut self) -> Result<PlayingResult, BeloteErrorKind> {
        let mut belote_rebelote: Option<Team> = None;
        for position in self.order() {
            if self
                .state
                .hand(position)
                .belote_rebelote(self.state.trump_color())
            {
                belote_rebelote = Some(position.team());
            }
        }
        let mut current_position = self.order()[0];
        let mut attack_points: u64 = 0;
        let mut stack = Stack::default();

        for turn_number in 0..MAX_CARDS_BY_PLAYER {
            let mut turn = Turn::new(turn_number as u64 + 1, self.state().order());
            loop {
                print!("{current_position} to play for {turn}");
                println!(
                    "Hand of {current_position} before playing : {}",
                    self.state.hand(current_position)
                );

                let choices = &self.players[current_position].choices(
                    self.state.hand(current_position),
                    &current_position,
                    &turn,
                    self.state.trump_color(),
                )?;
                if choices.is_empty() {
                    return Err(BeloteErrorKind::InvalidCase(
                        "no choices available".to_string(),
                    ));
                }

                turn.called_color().map_or_else(
                    || println!("{current_position} is first to play, you can choose a color"),
                    |called_color| println!("{current_position} must play color {called_color}"),
                );

                let chosen_card = if self.players[current_position].random() {
                    let mut rng = rand::thread_rng();
                    let Some(random_card): Option<Card> = choices.iter().choose(&mut rng).copied()
                    else {
                        return Err(BeloteErrorKind::InvalidCase(
                            "cannot find a random card choice".to_string(),
                        ));
                    };
                    random_card
                } else {
                    loop {
                        let cards: Vec<String> = choices
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect();
                        let page_size = cards.len();
                        let chosen_card =
                            Select::new("Which card do you choose ? (ESC to cancel)", cards)
                                .with_page_size(page_size)
                                .prompt_skippable();
                        match chosen_card {
                            Ok(Some(maybe_chosen_card)) => {
                                match Card::from_str(&maybe_chosen_card) {
                                    Ok(chosen_card) => break chosen_card,
                                    Err(e) => return Err(e),
                                }
                            }
                            Ok(None) => {
                                eprintln!("Interrupted.");
                                return Ok(PlayingResult::Interrupted);
                            }
                            Err(_) => {
                                eprintln!("Error with questionnaire, try again.");
                                continue;
                            }
                        }
                    }
                };

                if !self.state.hand_mut(current_position).give(&chosen_card) {
                    return Err(BeloteErrorKind::InvalidCase(
                        "cannot give chosen card".to_string(),
                    ));
                }
                println!(
                    "Hand of {current_position} after playing : {}",
                    self.state.hand(current_position)
                );
                turn.put(self.state.trump_color(), current_position, &chosen_card);
                if turn.finished() {
                    break;
                }
                current_position = *(current_position.next());
            }

            current_position = turn.master_position();
            println!("Fold master is player {}", turn.master_position());
            let master_team = turn.master_team();
            let Some(cards) = turn.take() else {
                return Err(BeloteErrorKind::InvalidCase(
                    "Cannot take turn cards".to_string(),
                ));
            };
            for (index, card) in cards.iter().enumerate() {
                let points = card.points(self.state.trump_color());
                if self.state.taker().team() == master_team {
                    attack_points += points;
                }
                let stack_index = turn_number * MAX_PLAYERS + index;
                println!("Replacing {card} at stack index {stack_index}");
                stack.set(stack_index, card);
            }
            println!("New attack points = {attack_points}");
        }

        if current_position.team() == self.state.taker().team() {
            attack_points += 10;
        }

        let (final_attack_points, final_defense_points): (u64, u64) = match attack_points {
            0 => (0, 252),                                    // capot for defense
            1..=80 => (0, 162),                               // chute
            81 => (81, 81),                                   // litige
            82..=161 => (attack_points, 162 - attack_points), // reussite
            162 => (252, 0),                                  // capot for attack
            _ => {
                return Err(BeloteErrorKind::InvalidCase(format!(
                    "bad points number : {attack_points}"
                )))
            }
        };

        for team in Team::iter() {
            if belote_rebelote == Some(team) {
                self.points[team] += 20;
            }
            if team == self.state.taker().team() {
                self.points[team] += final_attack_points;
            } else {
                self.points[team] += final_defense_points;
            }
        }
        Ok(PlayingResult::NextGame(Game {
            players: self.players,
            state: self.state.initial().next(stack),
            points: self.points,
        }))
    }
}
