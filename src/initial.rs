use crate::deck::Deck;
use crate::distribution::Distribution;
use crate::game::Game;
use crate::hands::Hands;
use crate::order::Order;
use crate::players::Players;
use crate::points::Points;
use crate::stack::{Iter as StackIter, Stack};
use crate::traits::PlayingOrder;
use tracing::info;

pub struct Initial {
    number: u64,
    order: Order,
    stack_iter: StackIter,
}

impl Initial {
    pub fn new(order: Order) -> Self {
        Self {
            order,
            number: 0,
            stack_iter: Box::new(Stack::default().into_iter()),
        }
    }
    #[must_use]
    pub fn next(mut self, mut deck: Deck) -> Self {
        self.order.rotate();
        self.number += 1;
        deck.cut();
        self.stack_iter = Box::new(deck.into_iter());
        self
    }
    pub const fn order(&self) -> Order {
        self.order
    }
    pub const fn number(&self) -> u64 {
        self.number
    }
    pub fn stack_iter(&mut self) -> &mut StackIter {
        &mut self.stack_iter
    }
}

impl Game<Initial> {
    pub const fn number(&self) -> u64 {
        self.state().number()
    }

    pub fn default(players: Players, order: Order) -> Self {
        Self::new(players, Points::default(), Initial::new(order))
    }

    pub fn distribute(mut self) -> Game<Distribution> {
        let mut hands = Hands::default();
        for position in self.state().order() {
            for card in self.state_mut().stack_iter().take(3) {
                hands[position].take(card);
            }
        }
        for position in self.state().order() {
            for card in self.state_mut().stack_iter().take(2) {
                hands[position].take(card);
            }
            info!("{position} : {}", hands[position]);
        }
        Game::new(
            self.players(),
            self.points(),
            Distribution::new(hands, self.consume()),
        )
    }
}

impl PlayingOrder for Game<Initial> {
    fn order(&self) -> Order {
        self.state().order()
    }
}