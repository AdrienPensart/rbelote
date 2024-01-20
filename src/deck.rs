use crate::card::Card;
use crate::hands::Hand;
use crate::stack::Iter as StackIter;
use derive_more::IntoIterator;
use rand::{thread_rng, Rng};
use std::fmt;

#[derive(Default, Clone, Debug, IntoIterator)]
pub struct Deck(Vec<Card>);

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        for card in &self.0 {
            writeln!(f, "\t{card}")?;
        }
        Ok(())
    }
}

impl Deck {
    pub fn append_stack_iter(&mut self, stack_iter: &mut StackIter) {
        self.0.extend(stack_iter);
    }
    pub fn append_card(&mut self, card: &Card) {
        self.0.push(*card);
    }
    pub fn append_hand(&mut self, hand: Hand) {
        self.0.extend(hand);
    }
    pub fn append_deck(&mut self, deck: Self) {
        self.0.extend(deck);
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn cut(&mut self) {
        let mut rng = thread_rng();
        let len = self.len();
        self.0.rotate_left(rng.gen_range(0..len));
    }
}
