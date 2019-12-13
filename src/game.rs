use failure::Error;
use std::fmt;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;
use crate::deck::*;
use crate::player::*;
use crate::errors::*;
use crate::helpers::*;
use crate::turn::*;

#[derive(Default, Debug)]
pub struct Game {
    deck: Deck,
    players: Vec<Player>,
    random: bool,
    auto: bool,
    defense_cards: usize,
    attack_cards: usize,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for p in &self.players {
            writeln!(f, "\t{}", p)?;
        }
        Ok(())
    }
}

impl Game
{
    pub fn new(random: bool, auto: bool) -> Game {
        Game {
            random,
            auto,
            deck: Deck::build_deck(),
            players: vec![
                Player::new(Position::South, random),
                Player::new(Position::West, random),
                Player::new(Position::North, random),
                Player::new(Position::East, random),
            ],
            ..Game::default()
        }
    }
    pub fn passed(&self) -> bool {
        self.players.iter().all(|p| p.contract == Some(Contract::Pass))
    }
    pub fn finished(&self) -> bool {
        self.players.iter().all(|p| p.hand.is_empty())
    }
    pub fn distribute(&mut self) {
        let mut decks : Vec<Deck> = Vec::new();
        for p in self.players.iter_mut() {
            let hand = p.hand.give_all();
            decks.push(hand);
            let deck = p.owned.give_all();
            decks.push(deck);
            p.prepare();
        }

        let mut rng = thread_rng();
        decks.shuffle(&mut rng);
        for mut d in decks {
            self.deck.append(d.give_all());
        }

        self.defense_cards = 0;
        self.attack_cards = 0;

        for p in self.players.iter_mut() {
            p.hand.append(self.deck.give(8));
        }
    }
    pub fn bidding(&mut self) -> Result<(), Error> {
        let mut contracts: Vec<Contract> = Contract::iter().collect();

        for p in self.players.iter_mut() {
            if self.auto && contracts.len() == 1 {
                p.contract = Some(Contract::Pass);
                println!("Auto pass");
                continue
            }

            p.contract = if self.random {
                Some(contracts[rand::thread_rng().gen_range(0, contracts.len())])
            } else {
                loop {
                    println!("{} must play : {}", &p, &p.hand);
                    println!("Choose a contract, possibilities :");
                    for (i, c) in contracts.iter().enumerate() {
                        println!("\t{} : press {}", c, i);
                    }
                    let contract_index = read_index();
                    if contract_index < contracts.len() {
                        break Some(contracts[contract_index])
                    } else {
                        println!("Error, please retry");
                    }
                }
            };

            contracts = match p.contract {
                Some(Contract::Pass) => {
                    println!("Pass");
                    p.contract = Some(Contract::Pass);
                    contracts
                },
                Some(contract) => {
                    println!("Chosen contract: {}", contract);
                    p.contract = Some(contract);
                    Contract::iter().filter(|other_contract| other_contract == &Contract::Pass || *other_contract as usize > contract as usize).collect()
                },
                _ => {
                    println!("A contract must be available for everyone!");
                    return Err(BeloteErrorKind::InvalidCase.into())
                }
            };
        }
        Ok(())
    }
    pub fn play (&mut self) -> Result<(), Error> {
        let mut turn = Turn::default();
        let mut master_player: usize = 0;
        for (i, p) in self.players.iter_mut().enumerate() {
            println!("{}", &turn);
            println!("Hand of {} : {}", &p, &p.hand);
            println!("Choices :");
            let choices = &p.choices(&turn);
            if choices.is_empty() {
                println!("No choices available, invalid case.");
                return Err(BeloteErrorKind::InvalidCase.into())
            }
            for &i in choices {
                println!("\t{0: <2} : {1}", &i, p.hand.0[i]);
            }

            if let Some(master) = turn.master_card() {
                println!("{} must play color {}", &p.position, &master)
            } else {
                println!("{} is first to play:", &p.position)
            }

            let index = if self.auto && choices.len() == 1 {
                choices[0]
            } else if self.random {
                choices[rand::thread_rng().gen_range(0, choices.len())]
            } else {
                loop {
                    let choice_index = read_index();
                    if choices.contains(&choice_index) {
                        break choice_index
                    } else {
                        println!("Error, please retry")
                    }
                }
            };

            let card = p.give_one(index);
            turn.put(card);
            if let Some(master) = turn.master_card() {
                if master.master(card) {
                    println!("Master card is {}, so player {} stays master", master, master_player);
                } else {
                    println!("Master card is {}, so player {} becomes master", card, i);
                    master_player = i;
                    turn.master_index = Some(turn.len()-1);
                }
            } else {
                println!("First card is {}, so player {} becomes master", card, i);
                master_player = i;
                turn.master_index = Some(turn.len()-1);
            }
        }

        let cards = turn.take();
        println!("Winner is player {}", self.players[master_player]);
        match self.players[master_player].team {
            Some(Team::Attack) => self.attack_cards += cards.len(),
            Some(Team::Defense) => self.defense_cards += cards.len(),
            _ => return Err(BeloteErrorKind::NoTeam.into())
        }
        self.players[master_player].owned.append(cards);
        self.players.rotate_left(master_player);
        Ok(())
    }
    pub fn count_points(&mut self) -> Result<(), Error> {
        if self.passed() {
            return Err(BeloteErrorKind::NoTaker.into());
        }
        Ok(())
    }
}
