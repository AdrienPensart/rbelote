use crate::bidding::Bidding;
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::player::{Player, Position, Team};
use fixed_map::Map;

#[derive(Debug)]
pub struct FirstDistribution {
    pub number: u64,
    pub dealer: Position,
    pub deck: Deck,
    pub players: Map<Position, Player>,
    pub team_total_points: Map<Team, u64>,
}

impl FirstDistribution {
    pub fn new(game: Game) -> Self {
        let mut players = game.players;
        let mut deck = game.deck;

        let mut team_deck = Map::new();
        team_deck.insert(Team::NorthSouth, Deck::default());
        team_deck.insert(Team::WestEast, Deck::default());

        for player in players.values_mut() {
            player.hand.append(deck.give(3));
        }
        for player in players.values_mut() {
            player.hand.append(deck.give(2));
        }
        for (position, player) in players.iter() {
            println!("{position} : {}", player.hand);
        }
        Self {
            team_total_points: game.team_total_points,
            dealer: game.dealer,
            deck,
            players,
            number: game.number,
        }
    }

    pub fn create_bidding(mut self) -> Result<Bidding, BeloteErrorKind> {
        let Some(card_returned) = self.deck.give_one() else {
            return Err(BeloteErrorKind::InvalidCase(
                "cannot get a returned card".to_string(),
            ));
        };
        println!("Card returned: {card_returned}");
        Ok(Bidding::new(self, card_returned))
    }
}
