use crate::bidding::Bidding;
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::game::{Game, Initial};
use crate::hands::Hands;
use crate::order::Order;
use crate::position::Position;
use crate::traits::PlayingOrder;

pub struct Distribution {
    hands: Hands,
    initial: Initial,
}

impl Distribution {
    pub fn new(initial: Initial, hands: Hands) -> Self {
        Self { hands, initial }
    }
    pub fn order(&self) -> Order {
        self.initial.order()
    }
    pub fn hand(&mut self, position: Position) -> &mut Deck {
        &mut self.hands[position]
    }
    pub fn next(self) -> Initial {
        self.initial.next()
    }
    pub fn number(&self) -> u64 {
        self.initial.number()
    }
}

impl PlayingOrder for Game<Distribution> {
    fn order(&self) -> Order {
        self.state.initial.order()
    }
}

impl Game<Distribution> {
    pub fn bidding(mut self) -> Result<Game<Bidding>, BeloteErrorKind> {
        let Some(card_returned) = self.deck.give_one() else {
            return Err(BeloteErrorKind::InvalidCase(
                "cannot get a returned card".to_string(),
            ));
        };
        println!("Card returned: {card_returned}");
        Ok(Game {
            points: self.points,
            deck: self.deck,
            players: self.players,
            state: Bidding {
                distribution: self.state,
                card_returned,
            },
        })
    }
}
