use crate::card::Color;

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
