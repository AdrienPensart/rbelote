use crate::players::Players;
use crate::points::Points;
use crate::team::Team;

use derive_more::Constructor;

#[derive(Debug, Constructor)]
pub struct Game<State> {
    players: Players,
    points: Points,
    state: State,
}

impl<State> Game<State> {
    pub fn consume(self) -> State {
        self.state
    }
    pub const fn state(&self) -> &State {
        &self.state
    }
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
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
