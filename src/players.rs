use crate::player::Player;
use crate::position::Position;
use std::ops::Index;

#[derive(Debug, Default)]
pub struct Players {
    pub north: Player,
    pub south: Player,
    pub east: Player,
    pub west: Player,
}

impl Index<Position> for Players {
    type Output = Player;

    fn index(&self, position: Position) -> &Self::Output {
        match position {
            Position::North => &self.north,
            Position::South => &self.south,
            Position::East => &self.east,
            Position::West => &self.west,
        }
    }
}
