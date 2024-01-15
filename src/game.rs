use crate::card::{Card, Color, Contract};
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::helpers::read_index;
use crate::player::{Player, Position, Team};
use crate::turn::Turn;
use fixed_map::Map;
use inquire::{Confirm, Select};
use rand::seq::SliceRandom;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::fmt;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct Game {
    number: u64,
    deck: Deck,
    dealer: Position,
    players: Map<Position, Player>,
    taker: Option<Position>,
    belote_rebelote: Option<Team>,
    trump_color: Option<Color>,
    team_deck: Map<Team, Deck>,
    team_points: Map<Team, u64>,
    team_total_points: Map<Team, u64>,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (position, points) in self.team_points.iter() {
            writeln!(f, "{} : {} points", position, points)?;
        }
        Ok(())
    }
}

impl Game {
    pub fn new(players: Map<Position, Player>) -> Result<Game, BeloteErrorKind> {
        let mut rng = rand::thread_rng();

        let Some(dealer): Option<Position> = Position::iter().choose(&mut rng) else {
            return Err(BeloteErrorKind::InvalidCase(
                "no dealer could be chosen".to_string(),
            ));
        };

        let mut team_deck = Map::new();
        team_deck.insert(Team::NorthSouth, Deck::default());
        team_deck.insert(Team::WestEast, Deck::default());

        let mut team_points = Map::new();
        team_points.insert(Team::NorthSouth, 0);
        team_points.insert(Team::WestEast, 0);

        let mut team_total_points = Map::new();
        team_total_points.insert(Team::NorthSouth, 0);
        team_total_points.insert(Team::WestEast, 0);

        Ok(Game {
            number: 0,
            dealer,
            deck: Deck::build_deck(),
            players,
            team_deck,
            team_points,
            team_total_points,
            taker: None,
            trump_color: None,
            belote_rebelote: None,
        })
    }
    pub fn number(&self) -> u64 {
        self.number
    }
    fn next_dealer(&mut self) -> Position {
        let next_dealer = self.dealer.next();
        self.dealer = next_dealer;
        next_dealer
    }
    pub fn distribute(&mut self) -> Card {
        self.number += 1;
        self.taker = None;
        self.trump_color = None;
        self.belote_rebelote = None;
        let mut decks: Vec<Deck> = Vec::new();
        for player in self.players.values_mut() {
            self.deck.append(player.hand.give_all());
        }
        for team_deck in self.team_deck.values_mut() {
            let hand = team_deck.give_all();
            decks.push(hand);
        }

        let mut rng = thread_rng();
        decks.shuffle(&mut rng);
        for mut d in decks {
            self.deck.append(d.give_all());
        }

        assert!(self.deck.len() == 32, "bad deck lenth {}", self.deck.len());

        for player in self.players.values_mut() {
            player.hand.append(self.deck.give(3));
        }
        for player in self.players.values_mut() {
            player.hand.append(self.deck.give(2));
        }
        for (position, player) in self.players.iter() {
            println!("{position} : {}", player.hand);
        }
        self.deck.give_one().unwrap()
    }
    pub fn bidding(&mut self, card_returned: Card) -> bool {
        let mut rng = rand::thread_rng();
        let mut trump_color = card_returned.color();
        println!("First bidding turn, card returned: {card_returned}");
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
                            self.deck.push(card_returned);
                            return false;
                        }
                        Err(_) => {
                            println!("Error with questionnaire, try again.");
                        }
                    };
                }
            };

            if take {
                println!("Pushing returned card {card_returned} in {position} hand");
                self.taker = Some(position);
                player.hand.push(card_returned);
                trump_color = card_returned.color();
                break;
            } else {
                println!("{position} did not take at first glance");
            }
        }
        if self.taker.is_none() {
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
                        let contracts: Vec<String> = Contract::iter()
                            .filter(|c| c.to_string() != card_returned.color().to_string())
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
                                self.deck.push(card_returned);
                                return false;
                            }
                            Err(_) => {
                                println!("Error with questionnaire, try again.");
                            }
                        };
                    }
                };
                if let Some(chosen_color) = chosen_color {
                    self.taker = Some(position);
                    player.hand.push(card_returned);
                    trump_color = chosen_color;
                    break;
                }
            }
        }
        let Some(taker) = self.taker else {
            self.deck.push(card_returned);
            return false;
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
            if player.belote_rebelote(trump_color) {
                self.belote_rebelote = Some(position.team());
            }
            println!("{position} : {player}");
        }
        self.trump_color = Some(trump_color);
        true
    }
    pub fn play(&mut self) -> Result<(), BeloteErrorKind> {
        let Some(trump_color) = self.trump_color else {
            return Err(BeloteErrorKind::InvalidColor);
        };
        let Some(taker) = self.taker else {
            return Err(BeloteErrorKind::NoTaker);
        };

        let starter = self.next_dealer();
        let mut current_position = starter;
        for turn_number in 0..8 {
            let mut turn = Turn::new(turn_number + 1);
            loop {
                println!("{current_position} to play for {turn_number}");
                let Some(current_player) = self.players.get_mut(current_position) else {
                    return Err(BeloteErrorKind::InvalidCase(
                        "current position not defined".to_string(),
                    ));
                };
                assert!(current_player.len() == 8 - turn_number as usize);

                println!("{turn}");
                println!("Hand of {current_position} : {}", current_player.hand);
                println!("Choices :");

                let choices = &current_player.choices(&current_position, &turn, trump_color)?;
                if choices.is_empty() {
                    return Err(BeloteErrorKind::InvalidCase(
                        "no choices available".to_string(),
                    ));
                }
                for i in choices {
                    println!("\t{0: <2} : {1}", i, current_player.hand.0[*i]);
                }

                if let Some(master_card) = turn.master_card() {
                    println!("{current_position} must play color {}", master_card.color())
                } else {
                    println!("{current_position} is first to play, you can choose a color")
                }

                let index = if current_player.random {
                    let mut rng = rand::thread_rng();
                    let Some(choice): Option<usize> = choices.iter().choose(&mut rng).copied()
                    else {
                        return Err(BeloteErrorKind::InvalidCase(
                            "invalid index for choice".to_string(),
                        ));
                    };
                    choice
                } else {
                    loop {
                        let choice_index = read_index();
                        if choices.contains(&choice_index) {
                            break choice_index;
                        } else {
                            println!("Error, please retry")
                        }
                    }
                };

                let given_card = current_player.give_one(index);
                turn.put(current_position, given_card);
                match turn.master_card() {
                    None => {
                        println!(
                            "First card is {given_card}, so player {current_position} becomes master"
                        );
                        turn.set_master_position(current_position);
                    }
                    Some(master_card) => {
                        if master_card.master(given_card, trump_color) {
                            println!(
                                "Master card is {master_card}, so player {current_position} stays master"
                            );
                        } else {
                            println!(
                                "Master card is {given_card}, so player {current_position} becomes master"
                            );
                            turn.set_master_position(current_position);
                        }
                    }
                }
                if turn.finished() {
                    break;
                }
                current_position = current_position.next();
            }

            match turn.master_position() {
                None => {
                    return Err(BeloteErrorKind::InvalidCase(
                        "no master position at the end of turn".to_string(),
                    ))
                }
                Some(master_position) => {
                    let Some(team_deck) = self.team_deck.get_mut(master_position.team()) else {
                        return Err(BeloteErrorKind::InvalidCase(
                            "no team deck for master team".to_string(),
                        ));
                    };
                    let cards = turn.take();
                    println!("Fold master is player {}", master_position);
                    team_deck.append(cards);
                }
            }
        }

        if let Some(last_master_team_points) = self.team_points.get_mut(current_position.team()) {
            *last_master_team_points += 10;
        }

        let Some(attack_team_points) = self.team_points.get_mut(taker.team()) else {
            return Err(BeloteErrorKind::InvalidCase(
                "no attack team points".to_string(),
            ));
        };

        let (attack_points, defense_points): (u64, u64) = match attack_team_points {
            0..=80 => (0, 160),
            81 => (81, 81),
            81..=159 => (*attack_team_points, 160 - *attack_team_points),
            160 => (250, 0),
            _ => {
                return Err(BeloteErrorKind::InvalidCase(
                    "bad points number".to_string(),
                ))
            }
        };

        for (team, total_points) in self.team_total_points.iter_mut() {
            if self.belote_rebelote == Some(team) {
                *total_points += 20;
            }
            if team == taker.team() {
                *total_points += attack_points;
            } else {
                *total_points += defense_points;
            }
        }

        // reset points
        for team_points in self.team_points.values_mut() {
            *team_points = 0;
        }
        Ok(())
    }
    pub fn team_points(&self) -> Map<Team, u64> {
        self.team_points
    }
}
