use strum_macros::{EnumCount, EnumIter, VariantArray};

use crate::team::Team;
use std::fmt;

// KEEP THIS ORDER
#[derive(Eq, PartialEq, Clone, Copy, Debug, VariantArray, EnumIter, EnumCount)]
pub enum Position {
    North,
    East,
    South,
    West,
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
    pub const fn team(&self) -> Team {
        match self {
            Self::East | Self::West => Team::WestEast,
            Self::North | Self::South => Team::NorthSouth,
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::East => Self::South,
            Self::West => Self::North,
            Self::North => Self::East,
            Self::South => Self::West,
        }
    }
}
