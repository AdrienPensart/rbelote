use crate::card::{Card, Color, Value};
use crate::constants::MAX_CARDS_BY_PLAYER;
use crate::errors::{BeloteErrorKind, ErrOnSome};
use derive_more::{Index, IntoIterator};
use std::fmt;
use tinyvec::ArrayVec;

#[derive(Default, Clone, Debug, Index, Copy, IntoIterator)]
pub struct Hand(ArrayVec<[Option<Card>; MAX_CARDS_BY_PLAYER]>);

impl Hand {
    pub fn belote_rebelote(&self, trump_color: Color) -> bool {
        let mut queen = false;
        let mut king = false;
        for card in self.into_iter().flatten() {
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
    pub fn take(&mut self, card: Card) -> Result<(), BeloteErrorKind> {
        self.0.try_push(Some(card)).err_on_some(|| {
            Err(BeloteErrorKind::InvalidCase(
                "Cannot append card to hand".to_string(),
            ))
        })
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn give(&mut self, card: &Card) -> Option<Card> {
        self.0
            .iter()
            .position(|c| *c == Some(*card))
            .and_then(|i| self.0.remove(i))
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        for card in self.0.into_iter().flatten() {
            writeln!(f, "\t{card}")?;
        }
        Ok(())
    }
}

// impl IntoIterator for Hand {
//     type Item = Card;
//     type IntoIter = std::iter::Flatten<std::vec::IntoIter<Option<Card>>>;

//     fn into_iter(self) -> Self::IntoIter {
//         let cards = vec![
//             self.card0, self.card1, self.card2, self.card3, self.card4, self.card5, self.card6,
//             self.card7,
//         ]
//         .into_iter()
//         .flatten();
//         cards.into_iter()
//     }
// }
