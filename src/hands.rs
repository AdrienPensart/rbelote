use crate::deck::Deck;
use crate::position::Position;
use std::ops::{Index, IndexMut};

#[derive(Debug, Default)]
pub struct Hands {
    pub north: Deck,
    pub south: Deck,
    pub east: Deck,
    pub west: Deck,
}

impl Index<Position> for Hands {
    type Output = Deck;

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
