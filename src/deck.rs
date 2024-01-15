use crate::card::*;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;
use strum::IntoEnumIterator;

#[derive(Default, Clone, Debug)]
pub struct Deck(pub Vec<Card>);
pub const MAX_CARDS: usize = 32;

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        for card in self.0.iter() {
            writeln!(f, "\t{}", card)?
        }
        Ok(())
    }
}

impl Deck {
    // fn points(&self, trump_color: Color) -> u16 {
    //     self.0.iter().map(|c| c.points(trump_color)).sum()
    // }
    pub fn build_deck() -> Deck {
        let mut d: Vec<Card> = Color::iter()
            .cartesian_product(Value::iter())
            .map(|(c, v)| Card::new(c, v))
            .collect();
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
    pub fn push(&mut self, card: Card) {
        self.0.push(card);
    }
    pub fn remove(&mut self, index: usize) -> Card {
        self.0.remove(index)
    }
    pub fn sort(&mut self) {
        self.0.sort();
    }
    pub fn has_color(&self, color: Color) -> bool {
        self.0.iter().any(|c| c.color() == color)
    }
    pub fn belote_rebelote(&self, trump_color: Color) -> bool {
        let mut queen = false;
        let mut king = false;
        for card in self.0.iter() {
            if card.color() == trump_color {
                if card.value() == Value::Queen {
                    queen = true;
                } else if card.value() == Value::King {
                    king = true;
                }
            }
        }
        queen && king
    }
}

#[test]
fn deck_tests() {
    let stack = Deck::build_deck();

    assert!(stack.len() == MAX_CARDS);

    let empty = Deck::default();
    assert!(empty.is_empty());
}
