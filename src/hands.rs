use crate::card::{Card, Color, Value};
use crate::deck::Deck;
use crate::position::Position;
use crate::traits::BeloteRebelote;
use derive_more::IntoIterator;
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Hand {
    card0: Option<Card>,
    card1: Option<Card>,
    card2: Option<Card>,
    card3: Option<Card>,
    card4: Option<Card>,
    card5: Option<Card>,
    card6: Option<Card>,
    card7: Option<Card>,
}

impl Hand {
    pub const fn count(&self) -> u64 {
        let mut length = 0;
        if self.card0.is_some() {
            length += 1;
        }
        if self.card1.is_some() {
            length += 1;
        }
        if self.card2.is_some() {
            length += 1;
        }
        if self.card3.is_some() {
            length += 1;
        }
        if self.card4.is_some() {
            length += 1;
        }
        if self.card5.is_some() {
            length += 1;
        }
        if self.card6.is_some() {
            length += 1;
        }
        if self.card7.is_some() {
            length += 1;
        }
        length
    }
    pub fn take(&mut self, card: Card) -> bool {
        if self.card0.is_none() {
            self.card0 = Some(card);
            return true;
        }
        if self.card1.is_none() {
            self.card1 = Some(card);
            return true;
        }
        if self.card2.is_none() {
            self.card2 = Some(card);
            return true;
        }
        if self.card3.is_none() {
            self.card3 = Some(card);
            return true;
        }
        if self.card4.is_none() {
            self.card4 = Some(card);
            return true;
        }
        if self.card5.is_none() {
            self.card5 = Some(card);
            return true;
        }
        if self.card6.is_none() {
            self.card6 = Some(card);
            return true;
        }
        if self.card7.is_none() {
            self.card7 = Some(card);
            return true;
        }
        false
    }
    pub fn give(&mut self, card: &Card) -> bool {
        if let Some(current_card) = self.card0 {
            if current_card == *card {
                self.card0 = None;
                return true;
            }
        }
        if let Some(current_card) = self.card1 {
            if current_card == *card {
                self.card1 = None;
                return true;
            }
        }
        if let Some(current_card) = self.card2 {
            if current_card == *card {
                self.card2 = None;
                return true;
            }
        }
        if let Some(current_card) = self.card3 {
            if current_card == *card {
                self.card3 = None;
                return true;
            }
        }
        if let Some(current_card) = self.card4 {
            if current_card == *card {
                self.card4 = None;
                return true;
            }
        }
        if let Some(current_card) = self.card5 {
            if current_card == *card {
                self.card5 = None;
                return true;
            }
        }
        if let Some(current_card) = self.card6 {
            if current_card == *card {
                self.card6 = None;
                return true;
            }
        }
        if let Some(current_card) = self.card7 {
            if current_card == *card {
                self.card7 = None;
                return true;
            }
        }
        false
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        if let Some(card) = self.card0 {
            writeln!(f, "\t{card}")?;
        }
        if let Some(card) = self.card1 {
            writeln!(f, "\t{card}")?;
        }
        if let Some(card) = self.card2 {
            writeln!(f, "\t{card}")?;
        }
        if let Some(card) = self.card3 {
            writeln!(f, "\t{card}")?;
        }
        if let Some(card) = self.card4 {
            writeln!(f, "\t{card}",)?;
        }
        if let Some(card) = self.card5 {
            writeln!(f, "\t{card}")?;
        }
        if let Some(card) = self.card6 {
            writeln!(f, "\t{card}")?;
        }
        if let Some(card) = self.card7 {
            writeln!(f, "\t{card}")?;
        }
        Ok(())
    }
}

impl IntoIterator for Hand {
    type Item = Card;
    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Option<Card>>>;

    fn into_iter(self) -> Self::IntoIter {
        let cards = vec![
            self.card0, self.card1, self.card2, self.card3, self.card4, self.card5, self.card6,
            self.card7,
        ]
        .into_iter()
        .flatten();
        cards.into_iter()
    }
}

impl BeloteRebelote for Hand {
    fn belote_rebelote(&self, trump_color: Color) -> bool {
        let mut queen = false;
        let mut king = false;
        for card in self.into_iter() {
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

#[derive(Default, Debug, Clone, Copy)]
pub struct Hands {
    pub north: Hand,
    pub south: Hand,
    pub east: Hand,
    pub west: Hand,
}

impl Hands {
    pub fn gather(self) -> Deck {
        let mut deck = Deck::default();
        deck.append_hand(self.north);
        deck.append_hand(self.south);
        deck.append_hand(self.east);
        deck.append_hand(self.west);
        deck
    }
}

impl Index<Position> for Hands {
    type Output = Hand;

    fn index(&self, position: Position) -> &Self::Output {
        match position {
            Position::North => &self.north,
            Position::South => &self.south,
            Position::East => &self.east,
            Position::West => &self.west,
        }
    }
}

impl IndexMut<Position> for Hands {
    fn index_mut(&mut self, position: Position) -> &mut Self::Output {
        match position {
            Position::North => &mut self.north,
            Position::South => &mut self.south,
            Position::East => &mut self.east,
            Position::West => &mut self.west,
        }
    }
}

#[test]
fn hand_tests() {
    let card = Card::new(Color::Club, Value::As);
    let mut hand = Hand {
        card5: Some(card),
        ..Hand::default()
    };
    assert!(hand.give(&card));
    assert!(hand == Hand::default());
}
