use crate::card::{Card, Color};
use crate::errors::BeloteErrorKind;
use crate::hands::Hand;
use crate::position::Position;
use crate::turn::Turn;
use std::fmt;

#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    random: bool,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "random: {}", self.random)
    }
}

impl Player {
    pub const fn new(random: bool) -> Self {
        Self { random }
    }
    pub const fn random(&self) -> bool {
        self.random
    }
    pub fn choices(
        &self,
        hand: &Hand,
        position: &Position,
        turn: &Turn,
        trump_color: Color,
    ) -> Result<Vec<Card>, BeloteErrorKind> {
        println!("{position} : trump color is {trump_color}");
        println!(
            "called color {:?} : master card {:?}",
            turn.called_color(),
            turn.master_card()
        );
        let choices = match (turn.called_color(), turn.master_card()) {
            (None, None) => hand.into_iter().collect(),
            (Some(called_color), Some(master_card)) => {
                let mut trumps = Vec::new();
                let mut trumps_less = Vec::new();
                let mut trumps_more = Vec::new();
                let mut other_colors = Vec::new();
                let mut same_colors = Vec::new();

                for card in hand.into_iter() {
                    if card.color() == trump_color {
                        trumps.push(card);
                        if card.power(trump_color) > master_card.power(trump_color) {
                            trumps_more.push(card);
                        } else {
                            trumps_less.push(card);
                        }
                    } else if card.color() == called_color {
                        same_colors.push(card);
                    } else {
                        other_colors.push(card);
                    }
                }

                // println!("trumps: {:?}", trumps);
                // println!("trumps_less: {:?}", trumps_less);
                // println!("trumps_more: {:?}", trumps_more);
                // println!("other_colors: {:?}", other_colors);
                // println!("same_colors: {:?}", same_colors);

                if called_color == trump_color {
                    if !trumps_more.is_empty() {
                        println!("{position} : trump color asked, I must go up");
                        trumps_more
                    } else if !trumps_less.is_empty() {
                        println!("{position} : trump color asked, but can't go up, going down");
                        trumps_less
                    } else {
                        println!("{position} : no trump left I must piss");
                        other_colors
                    }
                } else if !same_colors.is_empty() {
                    println!("{position} : I have asked color");
                    same_colors
                } else if turn.master_team() == position.team() {
                    println!(
                        "{position} : my team ({}) is master, I can defausse or cut",
                        turn.master_team()
                    );
                    if trumps_more.is_empty() {
                        other_colors.extend(trumps_less);
                    } else {
                        other_colors.extend(trumps_more);
                    }
                    other_colors
                } else if !trumps.is_empty() {
                    println!("{position} : I can't give asked color, I must cut with a trump");
                    trumps
                } else {
                    println!("{position} : no trumps left, I must piss");
                    other_colors
                }
            }
            _ => {
                return Err(BeloteErrorKind::InvalidCase(format!(
                    "no choices for player {position}"
                )))
            }
        };
        Ok(choices)
    }
}
