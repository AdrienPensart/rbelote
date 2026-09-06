use crate::errors::BeloteErrorKind;
use colored::Colorize;
use core::fmt::Display;
use std::fmt;
use std::str::FromStr;
use strum_macros::{Display, EnumIter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter, Hash)]
pub enum Color {
    Heart,
    Spade,
    Diamond,
    Club,
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::Club => "♣".blue(),
            Self::Diamond => "♦".red(),
            Self::Heart => "♥".red(),
            Self::Spade => "♠".blue(),
        };
        write!(f, "{repr}")
    }
}

/// `Display` renders colors through `colored`, which wraps them in ANSI escapes
/// whenever it detects a terminal. Drop those so parsing stays the inverse of
/// displaying, whether or not styling was applied.
fn strip_ansi(s: &str) -> String {
    let mut plain = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            in_escape = c != 'm';
        } else if c == '\u{1b}' {
            in_escape = true;
        } else {
            plain.push(c);
        }
    }
    plain
}

impl FromStr for Color {
    type Err = BeloteErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match strip_ansi(s).trim() {
            "♠" => Ok(Self::Spade),
            "♦" => Ok(Self::Diamond),
            "♣" => Ok(Self::Club),
            "♥" => Ok(Self::Heart),
            _ => Err(BeloteErrorKind::InvalidColor(s.to_string())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter, Display, Hash)]
pub enum Value {
    #[strum(serialize = "7")]
    _7,
    #[strum(serialize = "8")]
    _8,
    #[strum(serialize = "9")]
    _9,
    #[strum(serialize = "J")]
    Jack,
    #[strum(serialize = "Q")]
    Queen,
    #[strum(serialize = "K")]
    King,
    #[strum(serialize = "10")]
    _10,
    #[strum(serialize = "11")]
    As,
}

impl FromStr for Value {
    type Err = BeloteErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "J" => Ok(Self::Jack),
            "Q" => Ok(Self::Queen),
            "K" => Ok(Self::King),
            "7" => Ok(Self::_7),
            "8" => Ok(Self::_8),
            "9" => Ok(Self::_9),
            "10" => Ok(Self::_10),
            "11" => Ok(Self::As),
            _ => Err(BeloteErrorKind::InvalidValue(s.to_string())),
        }
    }
}

#[derive(Copy, Ord, Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Card {
    color: Color,
    value: Value,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.color, self.value)
    }
}

impl Card {
    pub const fn new(color: Color, value: Value) -> Option<Self> {
        Some(Self { color, value })
    }
    pub fn points(&self, trump_color: Color) -> u64 {
        match self.value {
            Value::As => 11,
            Value::_10 => 10,
            Value::King => 4,
            Value::Queen => 3,
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
            Value::_7 | Value::_8 => 0,
        }
    }
    pub const fn color(&self) -> Color {
        self.color
    }
    pub const fn value(&self) -> Value {
        self.value
    }
    pub fn master(self, arg: Self, trump_color: Color) -> bool {
        match (self, arg) {
            (card1, card2) if card1.color == trump_color && card2.color != trump_color => true,
            (card1, card2) if card1.color != trump_color && card2.color == trump_color => false,
            (card1, card2) => {
                card1.color != card2.color || card1.power(trump_color) > card2.power(trump_color)
            }
        }
    }
    // power is not a number of points, just a rating
    pub fn power(self, trump_color: Color) -> u8 {
        match self.value {
            Value::As => 11,
            Value::_10 => 10,
            Value::King => 5,
            Value::Queen => 4,
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
            Value::_8 => 1,
            Value::_7 => 0,
        }
    }
}

impl FromStr for Card {
    type Err = BeloteErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let plain = strip_ansi(s);
        let mut chars = plain.trim().chars();
        let Some(maybe_color) = chars.next() else {
            return Err(BeloteErrorKind::InvalidCard(s.to_string()));
        };
        let color = Color::from_str(maybe_color.to_string().as_str())?;
        let value = Value::from_str(chars.as_str().trim())?;
        Ok(Self { color, value })
    }
}

#[test]
fn card_tests() {
    assert!(Card::from_str("♦J").is_ok());

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
    assert!(heart_j.master(heart_10, Color::Heart));
}

#[test]
fn card_display_from_str_roundtrip() {
    use strum::IntoEnumIterator;

    // Both with and without the ANSI styling `colored` applies on a terminal.
    for styled in [false, true] {
        colored::control::set_override(styled);
        for color in Color::iter() {
            assert_eq!(Color::from_str(&color.to_string()), Ok(color));
            for value in Value::iter() {
                let Some(card) = Card::new(color, value) else {
                    panic!("cannot build card");
                };
                assert_eq!(
                    Card::from_str(&card.to_string()),
                    Ok(card),
                    "roundtrip failed for {card:?} (styled: {styled})"
                );
            }
        }
    }
    colored::control::unset_override();
}
