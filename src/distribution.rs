use crate::bidding::Bidding;
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hands::{Hand, Hands};
use crate::initial::Initial;
use crate::order::Order;
use crate::position::Position;
use crate::stack::Iter as StackIter;
use derive_more::Constructor;
use tracing::info;

#[derive(Constructor)]
pub struct Distribution {
    hands: Hands,
    initial: Box<Initial>,
}

impl Distribution {
    pub const fn order(&self) -> Order {
        self.initial.order()
    }
    pub fn gather(self) -> Deck {
        self.hands.gather()
    }
    pub fn hand(&self, position: Position) -> &Hand {
        &self.hands[position]
    }
    pub fn next(self, deck: Deck) -> Box<Initial> {
        self.initial.next(deck)
    }
    pub const fn number(&self) -> u64 {
        self.initial.number()
    }
    pub fn stack_iter(&mut self) -> &mut StackIter {
        self.initial.stack_iter()
    }
}

impl Game<Distribution> {
    pub fn bidding(mut self) -> Result<Game<Bidding>, BeloteErrorKind> {
        let Some(card_returned) = self.stack_iter().next() else {
            return Err(BeloteErrorKind::InvalidCase("No card returned".to_string()));
        };
        info!("Card returned: {card_returned}");
        Ok(Game::new(
            self.players(),
            self.points(),
            Box::new(Bidding::new(
                card_returned,
                self.hands,
                self.consume().initial,
            )),
        ))
    }
}
