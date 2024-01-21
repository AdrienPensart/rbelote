use crate::card::{Card, Color, Value};
use crate::constants::{MAX_CARDS, RETURNED_CARD};
use crate::deck::Deck;
use derive_more::{Deref, DerefMut, Index, IntoIterator};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

#[derive(Deref, DerefMut)]
pub struct Iter(Box<dyn Iterator<Item = Card>>);

impl Iter {
    pub fn from_deck(deck: Deck) -> Self {
        Self(Box::new(deck.into_iter()))
    }
}

impl Iterator for Iter {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        self.0.next()
    }
}

impl Default for Iter {
    fn default() -> Self {
        Self(Box::new(Stack::default().into_iter()))
    }
}

#[derive(Clone, Copy, Debug, Index, IntoIterator)]
pub struct Stack([Card; MAX_CARDS]);

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for card in self.0 {
            writeln!(f, "{card}")?;
        }
        Ok(())
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::random()
    }
}

impl Stack {
    pub const fn ordered() -> Self {
        let cards = [
            Card::new(Color::Club, Value::As),
            Card::new(Color::Club, Value::King),
            Card::new(Color::Club, Value::Queen),
            Card::new(Color::Club, Value::Jack),
            Card::new(Color::Club, Value::_10),
            Card::new(Color::Club, Value::_9),
            Card::new(Color::Club, Value::_8),
            Card::new(Color::Club, Value::_7),
            Card::new(Color::Heart, Value::As),
            Card::new(Color::Heart, Value::King),
            Card::new(Color::Heart, Value::Queen),
            Card::new(Color::Heart, Value::Jack),
            Card::new(Color::Heart, Value::_10),
            Card::new(Color::Heart, Value::_9),
            Card::new(Color::Heart, Value::_8),
            Card::new(Color::Heart, Value::_7),
            Card::new(Color::Spade, Value::As),
            Card::new(Color::Spade, Value::King),
            Card::new(Color::Spade, Value::Queen),
            Card::new(Color::Spade, Value::Jack),
            Card::new(Color::Spade, Value::_10),
            Card::new(Color::Spade, Value::_9),
            Card::new(Color::Spade, Value::_8),
            Card::new(Color::Spade, Value::_7),
            Card::new(Color::Diamond, Value::As),
            Card::new(Color::Diamond, Value::King),
            Card::new(Color::Diamond, Value::Queen),
            Card::new(Color::Diamond, Value::Jack),
            Card::new(Color::Diamond, Value::_10),
            Card::new(Color::Diamond, Value::_9),
            Card::new(Color::Diamond, Value::_8),
            Card::new(Color::Diamond, Value::_7),
        ];
        Self(cards)
    }
    pub const fn returned_card(&self) -> Card {
        self.0[RETURNED_CARD]
    }
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let mut new_stack = Self::ordered();
        new_stack.0.shuffle(&mut rng);
        new_stack
    }
}
