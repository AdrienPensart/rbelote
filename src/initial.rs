use crate::deck::Deck;
use crate::distribution::Distribution;
use crate::game::Game;
use crate::hands::Hands;
use crate::order::Order;
use crate::players::Players;
use crate::points::Points;
use crate::stack::Iter as StackIter;
use derive_new::new;
use tracing::info;

#[derive(new)]
pub struct Initial {
    order: Order,
    #[new(default)]
    number: u64,
    #[new(default)]
    stack_iter: StackIter,
    #[new(default)]
    litige: u64,
}

impl Initial {
    #[must_use]
    pub fn next(mut self, mut deck: Deck) -> Self {
        deck.cut();
        self.order.rotate();
        self.number += 1;
        self.stack_iter = StackIter::from_deck(deck);
        self
    }

    pub fn add_litige(&mut self, litige: u64) {
        self.litige += litige;
    }
    pub fn reset_litige(&mut self) -> u64 {
        let old_litige = self.litige;
        self.litige = 0;
        old_litige
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
    pub fn default(players: Players, order: Order) -> Self {
        Self::new(players, Points::default(), Initial::new(order))
    }

    pub fn distribute(mut self) -> Game<Distribution> {
        let mut hands = Hands::default();
        for position in self.order() {
            for card in self.stack_iter().take(3) {
                hands[position].take(card);
            }
        }
        for position in self.order() {
            for card in self.stack_iter().take(2) {
                hands[position].take(card);
            }
            info!("{position} : {}", hands[position]);
        }
        Game::new(
            self.players(),
            self.points(),
            Distribution::new(hands, self.into()),
        )
    }
}
