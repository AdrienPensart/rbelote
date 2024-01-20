use crate::card::{Card, Color};
use crate::constants::RETURNED_CARD;
use crate::contract::Contract;
use crate::deck::Deck;
use crate::game::Game;
use crate::hands::{Hand, Hands};
use crate::initial::Initial;
use crate::order::Order;
use crate::playing::Playing;
use crate::position::Position;
use crate::stack::Stack;
use crate::traits::PlayingOrder;
use derive_more::Constructor;
use inquire::{Confirm, Select};
use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::str::FromStr;
use strum::IntoEnumIterator;

pub enum PlayOrNext {
    NextGame(Game<Initial>),
    PlayGame(Game<Playing>),
    Interrupted,
}

#[derive(Constructor, Copy, Clone)]
pub struct Bidding {
    card_returned: Card,
    hands: Hands,
    initial: Initial,
}

impl Bidding {
    pub const fn order(&self) -> Order {
        self.initial.order()
    }

    pub const fn initial(self) -> Initial {
        self.initial
    }

    pub const fn stack(&self) -> Stack {
        self.initial.stack()
    }

    pub fn stack_mut(&mut self) -> &mut Stack {
        self.initial.stack_mut()
    }

    pub fn hand(&self, position: Position) -> Hand {
        self.hands[position]
    }

    pub fn hand_mut(&mut self, position: Position) -> &mut Hand {
        &mut self.hands[position]
    }

    pub fn gather(&self) -> Deck {
        self.hands.gather()
    }
}

impl PlayingOrder for Game<Bidding> {
    fn order(&self) -> Order {
        self.state().order()
    }
}

impl Game<Bidding> {
    pub fn playing_game_or_redistribute(mut self) -> PlayOrNext {
        let order = self.order();
        let players = self.players();
        let points = self.points();
        let mut rng = thread_rng();
        let card_returned = self.state().card_returned;
        let mut trump_color = card_returned.color();
        let mut taker: Option<Position> = None;
        println!("First bidding turn");
        for position in order {
            let take = if players[position].random() {
                rng.gen_bool(0.5)
            } else {
                'questionnaire: loop {
                    println!(
                        "{position} must decide if he is taking : {}",
                        self.state().hand(position)
                    );
                    let answer = Confirm::new("Do you take ? (ESC to cancel)")
                        .with_default(false)
                        .prompt_skippable();
                    match answer {
                        Ok(Some(value)) => {
                            break 'questionnaire value;
                        }
                        Ok(None) => {
                            println!("Interrupted.");
                            return PlayOrNext::Interrupted;
                        }
                        Err(_) => {
                            println!("Error with questionnaire, try again.");
                        }
                    };
                }
            };

            if take {
                taker = Some(position);
                trump_color = card_returned.color();
                break;
            }
            println!("{position} did not take at first glance");
        }
        if taker.is_none() {
            println!("Second bidding turn");
            'second_turn: for position in order {
                let chosen_color = if players[position].random() {
                    Contract::iter()
                        .choose(&mut rng)
                        .and_then(|contract| contract.color())
                } else {
                    'choose_contract: loop {
                        let card_returned_color = card_returned.color();
                        let contracts: Vec<String> = Contract::iter()
                            .filter(|c| c.to_string() != card_returned_color.to_string())
                            .map(|c| c.to_string())
                            .collect();
                        println!("Nobody took: {card_returned}, please choose a color for trumps");
                        let answer =
                            Select::new("Which color do you choose ? (ESC to cancel)", contracts)
                                .prompt_skippable();
                        match answer {
                            Ok(Some(maybe_chosen_color)) => {
                                if let Ok(chosen_color) = Color::from_str(&maybe_chosen_color) {
                                    break 'choose_contract Some(chosen_color);
                                }
                                continue 'second_turn;
                            }
                            Ok(None) => {
                                println!("Interrupted.");
                                return PlayOrNext::Interrupted;
                            }
                            Err(_) => {
                                println!("Error with questionnaire, try again.");
                            }
                        };
                    }
                };
                if let Some(chosen_color) = chosen_color {
                    taker = Some(position);
                    self.state_mut().hand_mut(position).take(card_returned);
                    trump_color = chosen_color;
                    break;
                }
            }
        }
        let Some(taker) = taker else {
            for (index, card) in self.state().gather().iter().enumerate() {
                self.state_mut()
                    .stack_mut()
                    .set(RETURNED_CARD + 1 + index, card);
            }

            return PlayOrNext::NextGame(Game::new(
                players,
                points,
                self.state().initial().next(self.state().stack()),
            ));
        };

        println!("{taker} for color {trump_color}, we give him {card_returned}");
        self.state_mut().hand_mut(taker).take(card_returned);

        let mut stack = self.state().stack();
        for position in order {
            if position == taker {
                println!("Giving {position} 2 more cards because taker");
                for card in stack.next_two_cards() {
                    self.state_mut().hand_mut(position).take(*card);
                }
            } else {
                println!("Giving {position} 3 more cards because others");
                for card in stack.next_three_cards() {
                    self.state_mut().hand_mut(position).take(*card);
                }
            }
        }
        PlayOrNext::PlayGame(Game::new(
            players,
            points,
            Playing::new(
                taker,
                self.state().hands,
                trump_color,
                self.state().initial(),
            ),
        ))
    }
}
