use crate::team::Team;
use std::ops::{Index, IndexMut};

#[derive(Debug, Default)]
pub struct Points {
    north_south: u64,
    west_east: u64,
}

impl Index<Team> for Points {
    type Output = u64;

    fn index(&self, team: Team) -> &Self::Output {
        match team {
            Team::NorthSouth => &self.north_south,
            Team::WestEast => &self.west_east,
        }
    }
}

impl IndexMut<Team> for Points {
    fn index_mut(&mut self, team: Team) -> &mut Self::Output {
        match team {
            Team::NorthSouth => &mut self.north_south,
            Team::WestEast => &mut self.west_east,
        }
    }
}
