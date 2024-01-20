use crate::bidding::Bidding;
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hands::{Hand, Hands};
use crate::initial::Initial;
use crate::order::Order;
use crate::position::Position;
use crate::stack::Iter as StackIter;
use crate::traits::PlayingOrder;
use derive_more::Constructor;
use tracing::info;

#[derive(Constructor)]
pub struct Distribution {
    hands: Hands,
    initial: Initial,
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
    pub fn next(self, deck: Deck) -> Initial {
        self.initial.next(deck)
    }
    pub const fn number(&self) -> u64 {
        self.initial.number()
    }
    pub fn stack_iter(&mut self) -> &mut StackIter {
        self.initial.stack_iter()
    }
}

impl PlayingOrder for Game<Distribution> {
    fn order(&self) -> Order {
        self.state().initial.order()
    }
}

impl Game<Distribution> {
    pub fn bidding(mut self) -> Result<Game<Bidding>, BeloteErrorKind> {
        let Some(card_returned) = self.state_mut().stack_iter().next() else {
            return Err(BeloteErrorKind::InvalidCase("No card returned".to_string()));
        };
        info!("Card returned: {card_returned}");
        Ok(Game::new(
            self.players(),
            self.points(),
            Bidding::new(card_returned, self.state().hands, self.consume().initial),
        ))
    }
}
