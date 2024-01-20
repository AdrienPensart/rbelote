use fixed_map::Key;
use std::fmt;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Key, EnumIter)]
pub enum Team {
    NorthSouth,
    WestEast,
}

impl Team {
    pub const fn other(&self) -> &Self {
        match self {
            Self::NorthSouth => &Self::WestEast,
            Self::WestEast => &Self::NorthSouth,
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::NorthSouth => write!(f, "North / South"),
            Self::WestEast => write!(f, "West / East"),
        }
    }
}
