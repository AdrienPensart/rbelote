use crate::card::Color;
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::position::Position;
use crate::turn::Turn;
use std::fmt;

#[derive(Clone, Debug, Default)]
pub struct Player {
    pub random: bool,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "random: {}", self.random)
    }
}

impl Player {
    pub fn new(random: bool) -> Player {
        Player { random }
    }
    pub fn choices(
        &self,
        hand: &Deck,
        position: &Position,
        turn: &Turn,
        trump_color: Color,
    ) -> Result<Vec<usize>, BeloteErrorKind> {
        println!("{position} : trump color is {trump_color}");
        let choices = match (turn.called(), turn.master_card(), turn.master_team()) {
            (None, None, None) => (0..hand.len()).collect::<Vec<usize>>(),
            (Some(called_color), Some(master_card), Some(master_team)) => {
                let mut trumps = Vec::new();
                let mut trumps_less = Vec::new();
                let mut trumps_more = Vec::new();
                let mut other_colors = Vec::new();
                let mut same_colors = Vec::new();

                for (i, card) in hand.0.iter().enumerate() {
                    if card.color() == trump_color {
                        trumps.push(i);
                        if card.points(trump_color) > master_card.points(trump_color) {
                            trumps_more.push(i);
                        } else {
                            trumps_less.push(i);
                        }
                    } else if card.color() == called_color {
                        same_colors.push(i);
                    } else {
                        other_colors.push(i);
                    }
                }

                println!("trumps: {:?}", trumps);
                println!("trumps_less: {:?}", trumps_less);
                println!("trumps_more: {:?}", trumps_more);
                println!("other_colors: {:?}", other_colors);
                println!("same_colors: {:?}", same_colors);

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
                } else if master_team == position.team() {
                    println!(
                        "{position} : my team ({master_team}) is master, I can defausse or cut"
                    );
                    if !trumps_more.is_empty() {
                        other_colors.extend(trumps_more);
                    } else {
                        other_colors.extend(trumps_less);
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
