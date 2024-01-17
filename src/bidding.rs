use crate::card::{Card, Color};
use crate::contract::Contract;
use crate::distribution::Distribution;
use crate::game::{Game, Initial, Playing};
use crate::order::Order;
use crate::position::Position;
use crate::traits::PlayingOrder;
use inquire::{Confirm, Select};
use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::str::FromStr;
use strum::IntoEnumIterator;

pub enum BiddingResult {
    NextGame(Game<Initial>),
    PlayGame(Game<Playing>),
    Interrupted,
}

pub struct Bidding {
    pub card_returned: Card,
    pub distribution: Distribution,
}

impl PlayingOrder for Game<Bidding> {
    fn order(&self) -> Order {
        self.state.distribution.order()
    }
}

impl Game<Bidding> {
    pub fn playing_game_or_redistribute(mut self) -> BiddingResult {
        let card_returned = self.state.card_returned;
        let mut rng = thread_rng();
        let mut trump_color = card_returned.color();
        let mut taker: Option<Position> = None;
        println!("First bidding turn");
        for position in self.order() {
            let take = if self.players[position].random {
                rng.gen_bool(0.5)
            } else {
                'questionnaire: loop {
                    println!(
                        "{position} must decide if he is taking : {}",
                        self.state.distribution.hand(position)
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
                            self.deck.push(card_returned);
                            return BiddingResult::Interrupted;
                        }
                        Err(_) => {
                            println!("Error with questionnaire, try again.");
                        }
                    };
                }
            };

            if take {
                println!("Pushing returned card {card_returned} in {position} hand");
                taker = Some(position);
                self.state.distribution.hand(position).push(card_returned);
                trump_color = card_returned.color();
                break;
            } else {
                println!("{position} did not take at first glance");
            }
        }
        if taker.is_none() {
            println!("Second bidding turn");
            'second_turn: for position in self.order() {
                let chosen_color = if self.players[position].random {
                    if let Some(chosen_contract) = Contract::iter().choose(&mut rng) {
                        chosen_contract.color()
                    } else {
                        None
                    }
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
                                return BiddingResult::Interrupted;
                            }
                            Err(_) => {
                                println!("Error with questionnaire, try again.");
                            }
                        };
                    }
                };
                if let Some(chosen_color) = chosen_color {
                    taker = Some(position);
                    self.state.distribution.hand(position).push(card_returned);
                    trump_color = chosen_color;
                    break;
                }
            }
        }
        let Some(taker) = taker else {
            self.deck.push(card_returned);
            for position in self.order() {
                self.deck
                    .append(self.state.distribution.hand(position).give_all());
            }
            return BiddingResult::NextGame(Game {
                deck: self.deck,
                players: self.players,
                state: self.state.distribution.next(),
                points: self.points,
            });
        };

        println!("Taker is {taker}");
        for position in self.order() {
            if position == taker {
                println!("Giving {position} 2 more cards because taker");
                self.state
                    .distribution
                    .hand(position)
                    .append(self.deck.give(2));
            } else {
                println!("Giving {position} 3 more cards because others");
                self.state
                    .distribution
                    .hand(position)
                    .append(self.deck.give(3));
            }
        }
        BiddingResult::PlayGame(Game {
            deck: self.deck,
            players: self.players,
            state: Playing {
                distribution: self.state.distribution,
                taker,
                trump_color,
            },
            points: self.points,
        })
    }
}
