use crate::card::Card;
use crate::hands::Hand;
use derive_more::{Deref, Index, Into, IntoIterator};
use std::fmt;

#[derive(Default, Clone, Debug, IntoIterator, Index, Deref, Into)]
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
    pub fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }
    // pub fn len(&self) -> usize {
    //     self.0.len()
    // }
    // pub fn is_empty(&self) -> bool {
    //     self.0.is_empty()
    // }
    pub fn append_hand(&mut self, hand: Hand) {
        self.0.extend(hand);
    }
}
