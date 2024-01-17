use crate::team::Team;

use fixed_map::Key;
use std::fmt;

pub const MAX_PLAYERS: usize = 4;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Key, StaticVariantsArray, EnumIter, EnumCount)]
pub enum Position {
    North,
    South,
    West,
    East,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::North => write!(f, "North"),
            Self::South => write!(f, "South"),
            Self::West => write!(f, "West"),
            Self::East => write!(f, "East"),
        }
    }
}

impl Position {
    pub fn team(&self) -> Team {
        match self {
            Self::East | Self::West => Team::WestEast,
            Self::North | Self::South => Team::NorthSouth,
        }
    }
    pub fn next(&self) -> Self {
        match self {
            Self::East => Self::South,
            Self::West => Self::North,
            Self::North => Self::East,
            Self::South => Self::West,
        }
    }
}
