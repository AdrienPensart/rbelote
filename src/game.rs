use crate::players::Players;
use crate::points::Points;
use crate::team::Team;

use derive_more::{Constructor, Deref, DerefMut};

#[derive(Debug, Constructor, Deref, DerefMut)]
pub struct Game<State> {
    players: Players,
    points: Points,
    #[deref]
    #[deref_mut]
    state: State,
}

impl<State> Game<State> {
    pub fn into(self) -> State {
        self.state
    }
    pub const fn is_full_random(&self) -> bool {
        self.players.full_random()
    }
    pub const fn points(&self) -> Points {
        self.points
    }
    pub fn add_points(&mut self, team: Team, points: u64) {
        self.points[team] += points;
    }
    pub const fn players(&self) -> Players {
        self.players
    }
}
