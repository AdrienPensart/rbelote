use crate::card::Color;
use crate::hands::{Hand, Hands};
use crate::initial::Initial;
use crate::order::Order;
use crate::position::Position;
use derive_more::Constructor;

#[derive(Constructor)]
pub struct Playing {
    taker: Position,
    hands: Hands,
    trump_color: Color,
    initial: Initial,
}

impl Playing {
    pub fn hand(&self, position: Position) -> &Hand {
        &self.hands[position]
    }

    pub const fn initial(self) -> Initial {
        self.initial
    }

    pub const fn taker(&self) -> Position {
        self.taker
    }

    pub const fn order(&self) -> Order {
        self.initial.order()
    }

    pub const fn hands(&self) -> Hands {
        self.hands
    }

    pub fn hand_mut(&mut self, position: Position) -> &mut Hand {
        &mut self.hands[position]
    }

    pub const fn trump_color(&self) -> Color {
        self.trump_color
    }
}
