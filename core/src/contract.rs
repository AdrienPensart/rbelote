use crate::card::Color;
use crate::errors::BeloteErrorKind;
use core::fmt::Display;
use derive_more::Debug;
use std::fmt;
use std::str::FromStr;
use strum_macros::EnumIter;

const PASS: &str = "Pass";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Contract {
    Heart,
    Spade,
    Diamond,
    Club,
    Pass,
}

impl Contract {
    pub const fn color(&self) -> Option<Color> {
        match self {
            Self::Club => Some(Color::Club),
            Self::Diamond => Some(Color::Diamond),
            Self::Heart => Some(Color::Heart),
            Self::Spade => Some(Color::Spade),
            Self::Pass => None,
        }
    }
}

impl Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color() {
            None => write!(f, "{PASS}"),
            Some(color) => {
                write!(f, "{color}")
            }
        }
    }
}

impl FromStr for Contract {
    type Err = BeloteErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == PASS {
            Ok(Self::Pass)
        } else {
            Ok(match Color::from_str(s)? {
                Color::Club => Self::Club,
                Color::Diamond => Self::Diamond,
                Color::Heart => Self::Heart,
                Color::Spade => Self::Spade,
            })
        }
    }
}
