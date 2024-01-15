use crate::bidding::Bidding;
use crate::card::Color;
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::helpers::read_index;
use crate::player::{Player, Position, Team};
use crate::turn::Turn;
use fixed_map::Map;
use rand::seq::IteratorRandom;

#[derive(Debug)]
pub struct InGame {
    pub dealer: Position,
    pub number: u64,
    pub players: Map<Position, Player>,
    pub taker: Position,
    pub belote_rebelote: Option<Team>,
    pub trump_color: Color,
    pub team_total_points: Map<Team, u64>,
}

impl InGame {
    pub fn new_from_bidding(bidding: Bidding, taker: Position, trump_color: Color) -> Self {
        let mut belote_rebelote: Option<Team> = None;
        for (position, player) in bidding.players.iter() {
            if player.belote_rebelote(trump_color) {
                belote_rebelote = Some(position.team());
            }
        }
        Self {
            taker,
            trump_color,
            belote_rebelote,
            players: bidding.players,
            team_total_points: bidding.team_total_points,
            number: bidding.number,
            dealer: bidding.dealer,
        }
    }

    pub fn play(mut self) -> Result<Game, BeloteErrorKind> {
        let starter = self.dealer.next();
        let mut current_position = starter;
        let mut attack_points: u64 = 0;
        let mut stack = Deck::default();

        for turn_number in 0..8 {
            let mut turn = Turn::new(turn_number + 1);
            loop {
                println!("{current_position} to play for {turn_number}");
                let Some(current_player) = self.players.get_mut(current_position) else {
                    return Err(BeloteErrorKind::InvalidCase(
                        "current position not defined".to_string(),
                    ));
                };
                println!("{turn}");
                println!("Hand of {current_position} : {}", current_player.hand);
                println!("Choices :");

                let choices =
                    &current_player.choices(&current_position, &turn, self.trump_color)?;
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
                        if master_card.master(given_card, self.trump_color) {
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
                    let cards = turn.take();
                    let turn_points = cards.points(self.trump_color);
                    println!("Fold master is player {}", master_position);
                    stack.append(cards);
                    if self.taker.team() == master_position.team() {
                        attack_points += turn_points;
                    }
                }
            }

            println!("Stack size: {}", stack.len());
        }

        if current_position.team() == self.taker.team() {
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

        for (team, total_points) in self.team_total_points.iter_mut() {
            if self.belote_rebelote == Some(team) {
                *total_points += 20;
            }
            if team == self.taker.team() {
                *total_points += final_attack_points;
            } else {
                *total_points += final_defense_points;
            }
        }
        Ok(Game::next_game(self, stack))
    }
}
