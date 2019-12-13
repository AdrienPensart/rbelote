use crate::deck::*;
use crate::card::*;
use crate::turn::*;
use crate::traits::*;
use std::fmt;


#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, EnumIter)]
pub enum Contract {
    Pass = 0,
    _80 = 80,
    _90 = 90,
    _100 = 100,
    _110 = 110,
    _120 = 120,
    _130 = 130,
    _140 = 140,
    _150 = 150,
    _160 = 160,
    _170 = 170,
    _180 = 180,
    Capot = 250,
    CapotBelote = 270,
    Generale = 500,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Type {
    Color(Color),
    SansAtout,
    ToutAtout,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Interruption {
    Coinche,
    Surcoinche,
}

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pass  => write!(f, "Passe."),
            Self::Capot => write!(f, "Capot."),
            Self::CapotBelote => write!(f, "Capot beloté."),
            Self::Generale => write!(f, "Generale."),
            _ => write!(f, "{}", *self as usize),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Position {
    South,
    West,
    North,
    East,
}

impl Default for Position {
    fn default() -> Position { Self::South }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::South => write!(f, "South"),
            Self::West => write!(f, "West"),
            Self::North => write!(f, "North"),
            Self::East => write!(f, "East"),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Team {
    Defense,
    Attack,
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Defense => write!(f, "Defense"),
            Self::Attack  => write!(f, "Attack"),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Player {
    pub position: Position,
    pub contract: Option<Contract>,
    pub interruption: Option<Interruption>,
    pub team: Option<Team>,
    pub hand: Deck,
    pub owned: Deck,
    pub total: u16,
    random: bool,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(team) = &self.team {
            write!(f, "name: {}, team: {}, points: {}", &self.position, team, &self.total)
        } else {
            write!(f, "name: {}, points: {}", &self.position, &self.total)
        }
    }
}

impl Player
{
    pub fn new(position: Position, random: bool) -> Player {
        Player {
            position,
            random,
            ..Player::default()
        }
    }
    pub fn prepare(&mut self) {
        self.team = None;
    }
    pub fn last_turn(&self) -> bool {
        self.hand.is_empty()
    }
    pub fn before_last_turn(&self) -> bool {
        self.hand.len() == 1
    }
    pub fn points(&self) -> u16 {
        self.owned.points()
    }
    pub fn is_first_turn(&self) -> bool {
        self.hand.len() == 8
    }
    pub fn give_one(&mut self, index: usize) -> Card {
        self.hand.0.remove(index)
    }
    pub fn choices(&self, turn: &Turn) -> Vec<usize> {
        let mut atouts = Vec::new();
        let mut atouts_less = Vec::new();
        let mut atouts_more = Vec::new();
        let mut other_colors = Vec::new();
        let mut same_color  = Vec::new();
        match (turn.called(), turn.master_card()) {
            (Some(Card{atout: false, color: called_color, ..}), Some(Card{atout: false, ..})) => {
                for (i, card) in self.hand.0.iter().enumerate() {
                    match card {
                        Card{atout: true, ..} => {
                            atouts.push(i);
                        },
                        Card{atout: false, color: card_color, ..} => {
                            if card_color == called_color {
                                same_color.push(i);
                            } else {
                                other_colors.push(i);
                            }
                        }
                    }
                }
                if !same_color.is_empty() {
                    same_color
                } else if !atouts.is_empty() {
                    atouts
                } else {
                    other_colors
                }
            },
            (Some(Card{atout: false, color: called_color, ..}), Some(Card{atout: true, value: master_atout_value, ..})) => {
                for (i, card) in self.hand.0.iter().enumerate() {
                    match card {
                        Card{atout: true, value: card_atout_value, ..} => {
                            if card_atout_value > master_atout_value {
                                atouts_more.push(i);
                            } else {
                                atouts_less.push(i);
                            }
                        },
                        Card{atout: false, color: card_color, ..} => {
                            if card_color == called_color {
                                same_color.push(i);
                            } else {
                                other_colors.push(i);
                            }
                        }
                    }
                }
                if !same_color.is_empty() {
                    same_color
                } else if !atouts_more.is_empty() {
                    atouts_more
                } else if !atouts_less.is_empty() {
                    atouts_less
                } else {
                    other_colors
                }
            },
            (Some(Card{atout: true, ..}), Some(Card{atout: true, value: master_atout_value, ..})) => {
                for (i, card) in self.hand.0.iter().enumerate() {
                    if let Card{atout: true, value: card_atout_value, ..} = card {
                        atouts.push(i);
                        if card_atout_value > master_atout_value {
                            atouts_more.push(i);
                        } else {
                            atouts_less.push(i);
                            other_colors.push(i);
                        }
                    } else {
                        other_colors.push(i)
                    }
                }
                if !atouts_more.is_empty() {
                    atouts_more
                } else if !atouts_less.is_empty() {
                    atouts_less
                } else {
                    other_colors
                }
            },
            _ => (0..self.hand.len()).collect()
        }
    }
}
