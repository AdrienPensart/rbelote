use crate::player::Player;
use crate::position::Position;
use std::ops::Index;

#[derive(Debug, Copy, Clone)]
pub struct Players {
    north: Player,
    east: Player,
    south: Player,
    west: Player,
}

impl Players {
    pub const fn new(north: Player, south: Player, east: Player, west: Player) -> Self {
        Self {
            north,
            east,
            south,
            west,
        }
    }

    pub const fn full_random(&self) -> bool {
        self.north.random() && self.south.random() && self.east.random() && self.west.random()
    }

    pub fn randomization(&self) -> f64 {
        let random_players = f64::from(self.north.random())
            + f64::from(self.east.random())
            + f64::from(self.south.random())
            + f64::from(self.west.random());
        if random_players.is_normal() {
            1.0 / random_players
        } else {
            0.0
        }
    }
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
