use crate::errors::BeloteErrorKind;
use std::fmt;
use std::str::FromStr;
use strum_macros::Display;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter, Display)]
pub enum Color {
    #[strum(serialize = "♥")]
    Heart,
    #[strum(serialize = "♠")]
    Spade,
    #[strum(serialize = "♦")]
    Diamond,
    #[strum(serialize = "♣")]
    Club,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter, Display)]
pub enum Contract {
    #[strum(serialize = "♥")]
    Heart,
    #[strum(serialize = "♠")]
    Spade,
    #[strum(serialize = "♦")]
    Diamond,
    #[strum(serialize = "♣")]
    Club,
    Pass,
}

impl Contract {
    pub fn color(&self) -> Option<Color> {
        match self {
            Self::Club => Some(Color::Club),
            Self::Diamond => Some(Color::Diamond),
            Self::Heart => Some(Color::Heart),
            Self::Spade => Some(Color::Spade),
            Self::Pass => None,
        }
    }
}

impl FromStr for Color {
    type Err = BeloteErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "♠" => Ok(Self::Spade),
            "♦" => Ok(Self::Diamond),
            "♣" => Ok(Self::Club),
            "♥" => Ok(Self::Heart),
            _ => Err(BeloteErrorKind::InvalidColor),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Value {
    _7,
    _8,
    _9,
    Jack,
    Queen,
    King,
    _10,
    As,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Jack => write!(f, "V"),
            Self::Queen => write!(f, "Q"),
            Self::King => write!(f, "K"),
            Self::_7 => write!(f, "7"),
            Self::_8 => write!(f, "8"),
            Self::_9 => write!(f, "9"),
            Self::_10 => write!(f, "10"),
            Self::As => write!(f, "11"),
        }
    }
}

#[derive(Copy, Ord, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Card {
    color: Color,
    value: Value,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.color, self.value)
    }
}

impl Card {
    pub fn new(color: Color, value: Value) -> Self {
        Self { color, value }
    }
    pub fn points(&self, trump_color: Color) -> u64 {
        match self.value {
            Value::Jack => {
                if self.color == trump_color {
                    20
                } else {
                    2
                }
            }
            Value::_9 => {
                if self.color == trump_color {
                    14
                } else {
                    0
                }
            }
            Value::As => 11,
            Value::_10 => 10,
            Value::King => 4,
            Value::Queen => 3,
            Value::_8 => 0,
            Value::_7 => 0,
        }
    }
    pub fn color(&self) -> Color {
        self.color
    }
    pub fn value(&self) -> Value {
        self.value
    }
    pub fn master(self, arg: Card, trump_color: Color) -> bool {
        match (self, arg) {
            (card1, card2) if card1.color == trump_color && card2.color != trump_color => true,
            (card1, card2) if card1.color != trump_color && card2.color == trump_color => false,
            (card1, card2) => {
                card1.color != card2.color || card1.power(trump_color) > card2.power(trump_color)
            }
        }
    }
    pub fn power(self, trump_color: Color) -> u8 {
        match self.value {
            Value::Jack => {
                if self.color == trump_color {
                    20
                } else {
                    3
                }
            }
            Value::_9 => {
                if self.color == trump_color {
                    14
                } else {
                    2
                }
            }
            Value::As => 11,
            Value::_10 => 10,
            Value::King => 5,
            Value::Queen => 4,
            Value::_8 => 1,
            Value::_7 => 0,
        }
    }
}

#[test]
fn card_tests() {
    let spade_7 = Card {
        color: Color::Spade,
        value: Value::_7,
    };
    let spade_10 = Card {
        color: Color::Spade,
        value: Value::_10,
    };
    let diamond_7 = Card {
        color: Color::Diamond,
        value: Value::_7,
    };
    let club_8 = Card {
        color: Color::Club,
        value: Value::_8,
    };
    assert!(!spade_10.master(diamond_7, Color::Diamond));
    assert!(spade_10.master(spade_7, Color::Club));
    assert!(club_8.master(diamond_7, Color::Heart));

    let heart_10 = Card {
        color: Color::Heart,
        value: Value::_10,
    };
    let heart_j = Card {
        color: Color::Heart,
        value: Value::Jack,
    };
    assert!(heart_j.master(heart_10, Color::Heart))
}
