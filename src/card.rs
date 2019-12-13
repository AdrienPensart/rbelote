use std::fmt;
use crate::traits::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Color {
    Heart,
    Spade,
    Diamond,
    Club,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Spade   => write!(f, "♠"),
            Self::Diamond => write!(f, "♦"),
            Self::Club  => write!(f, "♣"),
            Self::Heart   => write!(f, "♥"),
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
    pub atout: bool,
    pub color: Color,
    pub value: Value,
}

impl Points for Card {
    fn points(&self) -> u16 {
        match self.value {
            Value::Jack => if self.atout { 20 } else { 3 },
            Value::_9   => if self.atout { 14 } else { 0 },
            Value::As   => 11,
            Value::_10  => 10,
            Value::King => 5,
            Value::Queen => 4,
            Value::_8 => 0,
            Value::_7 => 0,
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.atout, self.color, self.value)
    }
}

impl Card {
    pub fn master(self, arg: Card) -> bool {
        match (&self, &arg) {
            (Card{atout: true, ..}, Card{atout: false, ..}) => true,
            (Card{atout: false, ..}, Card{atout: true, ..}) => false,
            (card1, card2) => card1.color != card2.color || card1.power() > card2.power()
        }
    }
    pub fn power(self) -> u8 {
        match self.value {
            Value::Jack => if self.atout { 20 } else { 3 },
            Value::_9   => if self.atout { 14 } else { 2 },
            Value::As   => 11,
            Value::_10  => 10,
            Value::King => 5,
            Value::Queen => 4,
            Value::_8 => 1,
            Value::_7 => 0,
        }
    }
}

#[test]
fn card_tests() {
    let spade_7 = Card{atout: false, color: Color::Spade, value: Value::_7};
    let spade_10 = Card{atout: false, color: Color::Spade, value: Value::_10};
    let diamond_7 = Card{atout: true, color: Color::Diamond, value: Value::_7};
    let club_8 = Card{atout: true, color: Color::Club, value: Value::_8};
    assert!(!spade_10.master(diamond_7));
    assert!(spade_10.master(spade_7));
    assert!(club_8.master(diamond_7));

    let heart_10 = Card{atout: true, color: Color::Heart, value: Value::_10};
    let heart_j = Card{atout: true, color: Color::Heart, value: Value::Jack};
    assert!(heart_j.master(heart_10))
}
