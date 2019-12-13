use std::fmt;
use crate::deck::Deck;
use crate::card::Card;

#[derive(Debug, Default)]
pub struct Turn {
    pub master_index: Option<usize>,
    cards: Deck,
}

impl Turn {
    pub fn take(&mut self) -> Deck {
        Deck(self.cards.0.drain(..).collect())
    }
    pub fn put(&mut self, card: Card) {
        self.cards.push(card);
    }
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    pub fn master_card(&self) -> Option<&Card> {
        if let Some(index) = self.master_index {
            Some(&self.cards.0[index])
        } else {
            None
        }
    }
    pub fn called(&self) -> Option<&Card> {
        if !self.is_empty() {
            Some(&self.cards.0[0])
        } else {
            None
        }
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Turn cards: {}", &self.cards)?;
        if let Some(master) = self.master_card() {
            write!(f, "\nMaster: {}", &master)?;
        }
        Ok(())
    }
}
