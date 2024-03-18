use crate::hand::Hand;
use crate::position::Position;
use std::ops::{Index, IndexMut};

#[derive(Default, Debug, Clone, Copy)]
pub struct Hands {
    north: Hand,
    south: Hand,
    east: Hand,
    west: Hand,
}

impl Index<Position> for Hands {
    type Output = Hand;

    fn index(&self, position: Position) -> &Self::Output {
        match position {
            Position::North => &self.north,
            Position::South => &self.south,
            Position::East => &self.east,
            Position::West => &self.west,
        }
    }
}

impl IndexMut<Position> for Hands {
    fn index_mut(&mut self, position: Position) -> &mut Self::Output {
        match position {
            Position::North => &mut self.north,
            Position::South => &mut self.south,
            Position::East => &mut self.east,
            Position::West => &mut self.west,
        }
    }
}
