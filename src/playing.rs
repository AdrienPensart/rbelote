use crate::card::Card;
use crate::card::Color;
use crate::constants::{MAX_CARDS_BY_PLAYER, MAX_PLAYERS};
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hands::{Hand, Hands};
use crate::initial::Initial;
use crate::order::Order;
use crate::position::Position;
use crate::team::Team;
use crate::traits::BeloteRebelote;
use crate::traits::PlayingOrder;
use crate::turn::Turn;
use derive_more::Constructor;
use inquire::Select;
use rand::seq::IteratorRandom;
use std::str::FromStr;
use tracing::{info, warn};

#[derive(Constructor)]
pub struct Playing {
    taker: Position,
    hands: Hands,
    trump_color: Color,
    initial: Initial,
}

impl Playing {
    pub fn hand(&self, position: Position) -> &Hand {
        &self.hands[position]
    }

    pub fn initial(self) -> Initial {
        self.initial
    }

    pub fn add_litige(&mut self, litige: u64) {
        self.initial.add_litige(litige);
    }

    pub fn reset_litige(&mut self) -> u64 {
        self.initial.reset_litige()
    }

    pub const fn taker(&self) -> Position {
        self.taker
    }

    pub const fn order(&self) -> Order {
        self.initial.order()
    }

    pub const fn hands(&self) -> Hands {
        self.hands
    }

    pub fn hand_mut(&mut self, position: Position) -> &mut Hand {
        &mut self.hands[position]
    }

    pub const fn trump_color(&self) -> Color {
        self.trump_color
    }
}

impl PlayingOrder for Game<Playing> {
    fn order(&self) -> Order {
        self.state().order()
    }
}

pub enum NextGameOrInterrupt {
    NextGame(Game<Initial>),
    Interrupted,
}

impl Game<Playing> {
    pub fn play(mut self) -> Result<NextGameOrInterrupt, BeloteErrorKind> {
        let mut belote_rebelote: Option<Team> = None;
        for position in self.order() {
            if self
                .state()
                .hand(position)
                .belote_rebelote(self.state().trump_color())
            {
                belote_rebelote = Some(position.team());
            }
        }
        let mut current_position = self.order()[0];
        let mut attack_points: u64 = 0;
        let mut defense_points: u64 = 0;
        let mut deck = Deck::default();

        for turn_number in 0..MAX_CARDS_BY_PLAYER {
            let mut turn = Turn::new(turn_number as u64 + 1, self.state().order());
            loop {
                info!("{current_position} to play for {turn}");
                info!(
                    "Hand of {current_position} before playing : {}",
                    self.state().hand(current_position)
                );

                let choices = &self.players()[current_position].choices(
                    self.state().hand(current_position),
                    &current_position,
                    &turn,
                    self.state().trump_color(),
                )?;
                if choices.is_empty() {
                    return Err(BeloteErrorKind::InvalidCase(
                        "no choices available".to_string(),
                    ));
                }

                turn.called_color().map_or_else(
                    || info!("{current_position} is first to play, you can choose a color"),
                    |called_color| info!("{current_position} must play color {called_color}"),
                );

                let chosen_card = if self.players()[current_position].random() {
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
                                info!("Interrupted.");
                                return Ok(NextGameOrInterrupt::Interrupted);
                            }
                            Err(_) => {
                                info!("Error with questionnaire, try again.");
                                continue;
                            }
                        }
                    }
                };

                if !self
                    .state_mut()
                    .hand_mut(current_position)
                    .give(&chosen_card)
                {
                    return Err(BeloteErrorKind::InvalidCase(
                        "cannot give chosen card".to_string(),
                    ));
                }
                info!(
                    "Hand of {current_position} after playing : {}",
                    self.state().hand(current_position)
                );
                turn.put(self.state().trump_color(), current_position, &chosen_card);
                if turn.finished() {
                    break;
                }
                current_position = current_position.next();
            }

            current_position = turn.master_position();
            info!("Fold master is player {}", turn.master_position());
            let master_team = turn.master_team();
            let Some(cards) = turn.take() else {
                return Err(BeloteErrorKind::InvalidCase(
                    "Cannot take turn cards".to_string(),
                ));
            };
            for card in cards {
                let points = card.points(self.state().trump_color());
                warn!("{card} : {points} points");
                if self.state().taker().team() == master_team {
                    attack_points += points;
                } else {
                    defense_points += points;
                }
                deck.append_card(&card);
            }

            if deck.len() != (turn_number + 1) * MAX_PLAYERS {
                return Err(BeloteErrorKind::InvalidCase(format!(
                    "bad deck length {} it should be {}",
                    deck.len(),
                    turn_number * MAX_PLAYERS
                )));
            };
            info!("New attack points = {attack_points}");
            info!("New defense points = {defense_points}");
        }

        if current_position.team() == self.state().taker().team() {
            attack_points += 10;
        }

        let (contract, total_points) = if belote_rebelote.is_some() {
            (92, 182)
        } else {
            (81, 162)
        };

        let (final_attack_points, final_defense_points): (u64, u64) = if attack_points == 0 {
            // capot for defense
            (0, 252 + self.state_mut().reset_litige())
        } else if attack_points >= 1 && attack_points < contract {
            // dedans
            (0, 182 + self.state_mut().reset_litige())
        } else if attack_points == contract {
            // litige
            self.state_mut().add_litige(contract);
            (0, contract)
        } else if attack_points > contract && attack_points < total_points {
            // reussite
            (
                attack_points + self.state_mut().reset_litige(),
                total_points - attack_points,
            )
        } else if attack_points == total_points {
            (252 + self.state_mut().reset_litige(), 0)
        } else {
            return Err(BeloteErrorKind::InvalidCase(format!(
                "bad points number : {attack_points}"
            )));
        };

        if let Some(belote_rebelote_team) = belote_rebelote {
            self.add_points(belote_rebelote_team, 20);
        }
        self.add_points(self.state().taker().team(), final_attack_points);
        self.add_points(self.state().taker().team().other(), final_defense_points);

        Ok(NextGameOrInterrupt::NextGame(Game::new(
            self.players(),
            self.points(),
            self.consume().initial().next(deck),
        )))
    }
}
