use crate::card::{Card, Color, Value};
use crate::constants::MAX_CARDS;
use crate::errors::{BeloteErrorKind, ErrOnSome};
use derive_more::{Index, IntoIterator};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::fmt;
use tinyvec::ArrayVec;

#[derive(Default, Clone, Debug, Index, Copy)]
pub struct Stack(ArrayVec<[Option<Card>; MAX_CARDS]>);

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for card in self.0.into_iter().flatten() {
            writeln!(f, "{card}")?;
        }
        Ok(())
    }
}

impl Stack {
    pub fn new() -> Self {
        let mut cards = ArrayVec::new();
        cards.push(Card::new(Color::Club, Value::As));
        cards.push(Card::new(Color::Club, Value::King));
        cards.push(Card::new(Color::Club, Value::Queen));
        cards.push(Card::new(Color::Club, Value::Jack));
        cards.push(Card::new(Color::Club, Value::_10));
        cards.push(Card::new(Color::Club, Value::_9));
        cards.push(Card::new(Color::Club, Value::_8));
        cards.push(Card::new(Color::Club, Value::_7));
        cards.push(Card::new(Color::Heart, Value::As));
        cards.push(Card::new(Color::Heart, Value::King));
        cards.push(Card::new(Color::Heart, Value::Queen));
        cards.push(Card::new(Color::Heart, Value::Jack));
        cards.push(Card::new(Color::Heart, Value::_10));
        cards.push(Card::new(Color::Heart, Value::_9));
        cards.push(Card::new(Color::Heart, Value::_8));
        cards.push(Card::new(Color::Heart, Value::_7));
        cards.push(Card::new(Color::Spade, Value::As));
        cards.push(Card::new(Color::Spade, Value::King));
        cards.push(Card::new(Color::Spade, Value::Queen));
        cards.push(Card::new(Color::Spade, Value::Jack));
        cards.push(Card::new(Color::Spade, Value::_10));
        cards.push(Card::new(Color::Spade, Value::_9));
        cards.push(Card::new(Color::Spade, Value::_8));
        cards.push(Card::new(Color::Spade, Value::_7));
        cards.push(Card::new(Color::Diamond, Value::As));
        cards.push(Card::new(Color::Diamond, Value::King));
        cards.push(Card::new(Color::Diamond, Value::Queen));
        cards.push(Card::new(Color::Diamond, Value::Jack));
        cards.push(Card::new(Color::Diamond, Value::_10));
        cards.push(Card::new(Color::Diamond, Value::_9));
        cards.push(Card::new(Color::Diamond, Value::_8));
        cards.push(Card::new(Color::Diamond, Value::_7));
        Self(cards)
    }
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let mut new_stack = Self::new();
        new_stack.0.shuffle(&mut rng);
        new_stack
    }
    pub fn cut(&mut self) {
        let mut rng = thread_rng();
        let len = self.0.len();
        self.0.rotate_left(rng.gen_range(0..len));
    }
    pub fn append_card(&mut self, card: Card) -> Result<(), BeloteErrorKind> {
        self.0.try_push(Some(card)).err_on_some(|| {
            Err(BeloteErrorKind::InvalidCase(
                "Cannot append card to stack".to_string(),
            ))
        })
    }
    pub fn give_card(&mut self) -> Result<Card, BeloteErrorKind> {
        let Some(option_card) = self.0.pop() else {
            return Err(BeloteErrorKind::InvalidCase(
                "cannot give a card".to_string(),
            ));
        };
        let Some(card) = option_card else {
            return Err(BeloteErrorKind::InvalidCase(
                "cannot give a card".to_string(),
            ));
        };
        Ok(card)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
