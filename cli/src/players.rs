use rbelote_core::position::Position;
use std::fmt;
use std::ops::Index;

/// Who sits at a seat. The engine does not care: it only ever asks the driver
/// for a decision, and the driver looks here to know who should make it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    random: bool,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "random: {}", self.random)
    }
}

impl Player {
    pub const fn new(random: bool) -> Self {
        Self { random }
    }
    pub const fn random(&self) -> bool {
        self.random
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Players {
    north: Player,
    east: Player,
    south: Player,
    west: Player,
}

impl Players {
    pub const fn new(north: Player, east: Player, south: Player, west: Player) -> Self {
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

    /// How eager a random player is to take, so that a table of four of them
    /// takes about once per deal.
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
