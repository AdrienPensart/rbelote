use crate::bidding::Bidding;
use crate::deck::Deck;
use crate::distribution::FirstDistribution;
use crate::errors::BeloteErrorKind;
use crate::in_game::InGame;
use crate::player::{Player, Position, Team};
use fixed_map::Map;
use rand::{seq::IteratorRandom, thread_rng};
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct Game {
    pub number: u64,
    pub deck: Deck,
    pub dealer: Position,
    pub players: Map<Position, Player>,
    pub team_total_points: Map<Team, u64>,
}

impl Game {
    pub fn new(players: Map<Position, Player>) -> Result<Self, BeloteErrorKind> {
        let mut rng = thread_rng();
        let Some(dealer): Option<Position> = Position::iter().choose(&mut rng) else {
            return Err(BeloteErrorKind::InvalidCase(
                "no dealer could be chosen".to_string(),
            ));
        };

        let mut team_total_points = Map::new();
        team_total_points.insert(Team::NorthSouth, 0);
        team_total_points.insert(Team::WestEast, 0);
        Ok(Self {
            players,
            number: 0,
            deck: Deck::build_deck(),
            team_total_points,
            dealer,
        })
    }
    pub fn next_from_passed_bidding(bidding: Bidding) -> Self {
        Self {
            deck: bidding.deck,
            players: bidding.players,
            team_total_points: bidding.team_total_points,
            number: bidding.number,
            dealer: bidding.dealer.next(),
        }
    }
    pub fn next_game(mut in_game: InGame, mut stack: Deck) -> Self {
        for player in in_game.players.values_mut() {
            stack.append(player.hand.give_all());
        }
        Self {
            dealer: in_game.dealer.next(),
            deck: stack,
            number: in_game.number,
            players: in_game.players,
            team_total_points: in_game.team_total_points,
        }
    }
    pub fn distribute(mut self) -> FirstDistribution {
        self.number += 1;
        FirstDistribution::new(self)
    }
    pub fn team_total_points(&self) -> Map<Team, u64> {
        self.team_total_points
    }
}
