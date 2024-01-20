use crate::card::{Card, Color, Value};
use crate::constants::{MAX_CARDS, RETURNED_CARD};
// use crate::errors::BeloteErrorKind;
use derive_more::Index;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

#[derive(Clone, Copy, Debug, Index)]
pub struct Stack {
    index: usize,
    #[index]
    cards: [Card; MAX_CARDS],
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for card in self.cards {
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
        Self { index: 0, cards }
    }

    pub fn set(&mut self, index: usize, card: &Card) {
        self.cards[index] = *card;
    }

    pub const fn returned_card(&self) -> Card {
        self.cards[RETURNED_CARD]
    }

    pub fn next_card(&mut self) -> Card {
        let card = self.cards[self.index];
        self.index += 1;
        card
    }

    pub fn next_two_cards(&mut self) -> &[Card] {
        let two_cards = &self.cards[self.index..self.index + 2];
        self.index += 2;
        two_cards
    }

    pub fn next_three_cards(&mut self) -> &[Card] {
        let three_cards = &self.cards[self.index..self.index + 3];
        self.index += 3;
        three_cards
    }

    pub fn random() -> Self {
        let mut rng = thread_rng();
        let mut new_stack = Self::ordered();
        new_stack.cards.shuffle(&mut rng);
        new_stack
    }
}
