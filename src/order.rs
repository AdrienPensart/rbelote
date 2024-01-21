use crate::constants;
use crate::position::Position;
use derive_more::{Index, IntoIterator};
use rand::{thread_rng, Rng};
use std::fmt;
use strum::{EnumCount, StaticVariantsArray};

#[derive(Debug, IntoIterator, Index, Clone, Copy)]
pub struct Order([Position; constants::MAX_PLAYERS]);

impl Default for Order {
    fn default() -> Self {
        let variants = [
            Position::North,
            Position::East,
            Position::South,
            Position::West,
        ];
        Self(variants)
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for position in self.into_iter() {
            write!(f, "{position} => ")?;
        }
        Ok(())
    }
}

impl Order {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let mut variants = [
            Position::North,
            Position::North,
            Position::North,
            Position::North,
        ];
        variants.copy_from_slice(Position::ALL_VARIANTS);
        let random_index = rng.gen_range(0..=Position::COUNT);
        variants.rotate_left(random_index);
        Self(variants)
    }
    pub const fn first(&self) -> Position {
        self.0[0]
    }
    pub fn rotate(mut self) {
        self.0.rotate_left(1);
    }
}

#[test]
fn order_tests() {
    let order1 = Order::default();
    let mut order_iter1 = order1.into_iter();
    assert!(order_iter1.next() == Some(Position::North));
    assert!(order_iter1.next() == Some(Position::East));
    assert!(order_iter1.next() == Some(Position::South));
    assert!(order_iter1.next() == Some(Position::West));
    assert!(order_iter1.next().is_none());

    let order2 = Order::default();
    order2.rotate();
    let mut order_iter2 = order2.into_iter();
    assert!(order_iter2.next() == Some(Position::East));
    assert!(order_iter2.next() == Some(Position::South));
    assert!(order_iter2.next() == Some(Position::West));
    assert!(order_iter2.next() == Some(Position::North));
    assert!(order_iter2.next().is_none());
}
