use crate::card::{Card, Color};
use crate::contract::Contract;
use crate::deck::Deck;
use crate::distribution::FirstDistribution;
use crate::game::Game;
use crate::in_game::InGame;
use crate::player::{Player, Position, Team};
use fixed_map::Map;
use inquire::{Confirm, Select};
use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct Bidding {
    pub number: u64,
    pub dealer: Position,
    pub card_returned: Card,
    pub deck: Deck,
    pub players: Map<Position, Player>,
    pub team_total_points: Map<Team, u64>,
}

impl Bidding {
    pub fn new(first_distribution: FirstDistribution, card_returned: Card) -> Self {
        Self {
            card_returned,
            team_total_points: first_distribution.team_total_points,
            number: first_distribution.number,
            dealer: first_distribution.dealer,
            deck: first_distribution.deck,
            players: first_distribution.players,
        }
    }

    pub fn start_game_or_next(mut self) -> (Option<Game>, Option<InGame>) {
        let mut rng = thread_rng();
        let mut trump_color = self.card_returned.color();
        let mut taker: Option<Position> = None;
        println!("First bidding turn");
        for (position, player) in self.players.iter_mut() {
            let take = if player.random {
                rng.gen_bool(0.5)
            } else {
                'questionnaire: loop {
                    println!("{position} must decide if he is taking : {}", player.hand);
                    let answer = Confirm::new("Do you take ? (ESC to cancel)")
                        .with_default(false)
                        .prompt_skippable();
                    match answer {
                        Ok(Some(value)) => {
                            break 'questionnaire value;
                        }
                        Ok(None) => {
                            println!("Interrupted.");
                            self.deck.push(self.card_returned);
                            return (None, None);
                        }
                        Err(_) => {
                            println!("Error with questionnaire, try again.");
                        }
                    };
                }
            };

            if take {
                println!(
                    "Pushing returned card {} in {position} hand",
                    self.card_returned
                );
                taker = Some(position);
                player.hand.push(self.card_returned);
                trump_color = self.card_returned.color();
                break;
            } else {
                println!("{position} did not take at first glance");
            }
        }
        if taker.is_none() {
            println!("Second bidding turn");
            'second_turn: for (position, player) in self.players.iter_mut() {
                let chosen_color = if player.random {
                    if let Some(chosen_contract) = Contract::iter().choose(&mut rng) {
                        chosen_contract.color()
                    } else {
                        None
                    }
                } else {
                    'choose_contract: loop {
                        let card_returned_color = self.card_returned.color();
                        let contracts: Vec<String> = Contract::iter()
                            .filter(|c| c.to_string() != card_returned_color.to_string())
                            .map(|c| c.to_string())
                            .collect();
                        println!(
                            "Nobody took: {}, please choose a color for trumps",
                            self.card_returned
                        );
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
                                return (None, None);
                            }
                            Err(_) => {
                                println!("Error with questionnaire, try again.");
                            }
                        };
                    }
                };
                if let Some(chosen_color) = chosen_color {
                    taker = Some(position);
                    player.hand.push(self.card_returned);
                    trump_color = chosen_color;
                    break;
                }
            }
        }
        let Some(taker) = taker else {
            self.deck.push(self.card_returned);
            for player in self.players.values_mut() {
                self.deck.append(player.hand.give_all());
            }
            return (Some(Game::next_from_passed_bidding(self)), None);
        };

        println!("Taker is {taker}");
        for (position, player) in self.players.iter_mut() {
            if position == taker {
                println!(
                    "Giving {position} 2 more cards because taker ({})",
                    player.len()
                );
                player.hand.append(self.deck.give(2));
            } else {
                println!(
                    "Giving {position} 3 more cards because others ({})",
                    player.len()
                );
                player.hand.append(self.deck.give(3));
            }
            println!("{position} : {player}");
        }
        (
            None,
            Some(InGame::new_from_bidding(self, taker, trump_color)),
        )
    }
}
