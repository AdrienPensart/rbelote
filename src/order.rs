use crate::helpers::gen_index;
use crate::position::{Position, MAX_PLAYERS};
use derive_more::{Index, IntoIterator};
use rand::thread_rng;
use strum::{EnumCount, StaticVariantsArray};

#[derive(Debug, IntoIterator, Index, Clone, Copy)]
pub struct Order([Position; MAX_PLAYERS]);

impl Default for Order {
    fn default() -> Self {
        let mut rng = thread_rng();
        let mut variants = [
            Position::North,
            Position::North,
            Position::North,
            Position::North,
        ];
        variants.copy_from_slice(Position::ALL_VARIANTS);
        let random_index = gen_index(&mut rng, Position::COUNT);
        variants.rotate_right(random_index);
        Order(variants)
    }
}

impl Order {
    pub fn rotate(&mut self) {
        self.0.rotate_right(1)
    }
}
