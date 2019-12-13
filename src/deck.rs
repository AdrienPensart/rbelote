use std::fmt;
use rand::thread_rng;
use rand::seq::SliceRandom;
use itertools::Itertools;
use strum::IntoEnumIterator;
use crate::card::*;
use crate::traits::*;

#[derive(Default, Clone, Debug)]
pub struct Deck (pub Vec<Card>);
pub const MAX_CARDS : usize = 32;

impl<'a> fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for card in self.0.iter() {
            write!(f, "\t{}", card)?
        }
        Ok(())
    }
}

impl Points for Deck {
    fn points(&self) -> u16 {
        self.0.iter().map(Points::points).sum()
    }
}

impl Deck {
    pub fn build_deck() -> Deck {
        let mut d : Vec<Card> = Color::iter().cartesian_product(Value::iter()).map(|(c, v)| Card{atout: false, color: c, value: v}).collect();
        let mut rng = thread_rng();
        d.shuffle(&mut rng);
        assert!(d.len() == MAX_CARDS);
        Deck(d)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn give(&mut self, size: usize) -> Deck {
        Deck(self.0.drain(0..size).collect())
    }
    pub fn give_one(&mut self) -> Option<Card> {
        if !self.is_empty() {
            Some(self.0.remove(0))
        } else {
            None
        }
    }
    pub fn give_all(&mut self) -> Deck {
        Deck(self.0.drain(..).collect())
    }
    pub fn append(&mut self, deck: Deck) {
        self.0.append(&mut deck.0.clone());
    }
    pub fn push(&mut self, card: Card){
        self.0.push(card);
    }
    pub fn sort(&mut self) {
        self.0.sort();
    }
}

#[test]
fn deck_tests() {
    let stack = Deck::build_deck();

    assert!(stack.len() == MAX_CARDS);

    let empty = Deck::default();
    assert!(empty.is_empty());
}

