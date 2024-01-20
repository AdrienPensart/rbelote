use crate::bidding::Bidding;
use crate::deck::Deck;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hands::{Hand, Hands};
use crate::initial::Initial;
use crate::order::Order;
use crate::position::Position;
use crate::stack::Stack;
use crate::traits::PlayingOrder;
use derive_more::Constructor;

#[derive(Debug, Clone, Copy, Constructor)]
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
    pub fn next(self, stack: Stack) -> Initial {
        self.initial.next(stack)
    }
    pub const fn number(&self) -> u64 {
        self.initial.number()
    }
    pub const fn stack(&self) -> Stack {
        self.initial.stack()
    }
    pub fn stack_mut(&mut self) -> &mut Stack {
        self.initial.stack_mut()
    }
}

impl PlayingOrder for Game<Distribution> {
    fn order(&self) -> Order {
        self.state().initial.order()
    }
}

impl Game<Distribution> {
    pub fn bidding(self) -> Result<Game<Bidding>, BeloteErrorKind> {
        let card_returned = self.state().stack().returned_card();
        println!("Card returned: {card_returned}");
        Ok(Game::new(
            self.players(),
            self.points(),
            Bidding::new(card_returned, self.state().hands, self.state().initial),
        ))
    }
}
