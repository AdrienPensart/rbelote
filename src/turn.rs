use crate::card::{Card, Color};
use crate::deck::Deck;
use crate::player::{Position, Team};
use fixed_map::Map;
use std::fmt;

#[derive(Debug, Default)]
pub struct Turn {
    number: u64,
    color: Option<Color>,
    cards: Map<Position, Card>,
    master_position: Option<Position>,
}

impl Turn {
    pub fn new(number: u64) -> Turn {
        Self {
            number,
            ..Default::default()
        }
    }
    pub fn take(self) -> Deck {
        let cards = self.cards.values().copied().collect();
        Deck(cards)
    }
    pub fn put(&mut self, position: Position, card: Card) {
        if self.cards.is_empty() {
            self.color = Some(card.color());
            self.master_position = Some(position);
        }
        self.cards.insert(position, card);
    }
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn is_first(&self) -> bool {
        self.color.is_none() && self.master_position.is_none() && !self.cards.is_empty()
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    pub fn finished(&self) -> bool {
        self.len() == 4
    }
    pub fn set_master_position(&mut self, master_position: Position) {
        self.master_position = Some(master_position);
    }
    pub fn master_position(&self) -> Option<Position> {
        self.master_position
    }
    pub fn master_card(&self) -> Option<&Card> {
        if let Some(master_player) = self.master_position {
            self.cards.get(master_player)
        } else {
            None
        }
    }
    pub fn master_team(&self) -> Option<Team> {
        self.master_position
            .map(|master_player| master_player.team())
    }
    pub fn called(&self) -> Option<Color> {
        self.color
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Turn: {}", self.number)?;
        for (position, card) in self.cards.iter() {
            writeln!(f, "\t{position}: {card}")?;
        }
        if let (Some(card), Some(master_position)) = (self.master_card(), self.master_position) {
            write!(f, "\nMaster card: {card} ({})", master_position)?;
        }
        Ok(())
    }
}
