use crate::distribution::Distribution;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hands::Hands;
use crate::order::Order;
use crate::players::Players;
use crate::points::Points;
use crate::stack::Stack;
use derive_new::new;
use tracing::info;

#[derive(new)]
pub struct Initial {
    order: Order,
    #[new(default)]
    number: u64,
    #[new(value = "Stack::random()")]
    stack: Stack,
    #[new(default)]
    litige: u64,
}

impl Initial {
    #[must_use]
    pub fn next(mut self) -> Self {
        self.order.rotate();
        self.number += 1;
        self.stack.cut();
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
    pub const fn stack(&self) -> &Stack {
        &self.stack
    }
    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}

impl Game<Initial> {
    pub fn default(players: Players, order: Order) -> Self {
        Self::new(players, Points::default(), Initial::new(order))
    }

    pub fn distribute(mut self) -> Result<Game<Distribution>, BeloteErrorKind> {
        let mut hands = Hands::default();
        for position in self.order() {
            for _ in 0..3 {
                hands[position].take(self.stack.give_card()?)?;
            }
        }
        for position in self.order() {
            for _ in 0..2 {
                hands[position].take(self.stack.give_card()?)?;
            }
            info!("{position} : {}", hands[position]);
        }
        Ok(Game::new(
            self.players(),
            self.points(),
            Distribution::new(hands, self.into()),
        ))
    }
}
