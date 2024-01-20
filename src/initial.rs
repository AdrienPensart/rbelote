use crate::order::Order;
use crate::stack::Stack;

#[derive(Debug, Clone, Copy)]
pub struct Initial {
    number: u64,
    order: Order,
    stack: Stack,
}

impl Initial {
    pub fn new(order: Order) -> Self {
        Self {
            order,
            number: 0,
            stack: Stack::random(),
        }
    }
    #[must_use]
    pub fn next(mut self, stack: Stack) -> Self {
        self.order.rotate();
        self.number += 1;
        self.stack = stack;

        println!("New stack:\n{stack}");

        self
    }
    pub const fn order(&self) -> Order {
        self.order
    }
    pub const fn number(&self) -> u64 {
        self.number
    }
    pub const fn stack(&self) -> Stack {
        self.stack
    }
    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}
