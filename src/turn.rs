use crate::card::{Card, Color};
use crate::order::Order;
use crate::position::Position;
use crate::team::Team;
use std::fmt;

#[derive(Debug)]
pub struct Turn {
    number: u64,
    order: Order,
    called_color: Option<Color>,
    north: Option<Card>,
    east: Option<Card>,
    south: Option<Card>,
    west: Option<Card>,
    master_position: Position,
}

impl Turn {
    pub const fn new(number: u64, order: Order) -> Self {
        Self {
            number,
            order,
            master_position: order.first(),
            called_color: None,
            east: None,
            north: None,
            south: None,
            west: None,
        }
    }
    pub fn take(self) -> Option<[Card; 4]> {
        let north = self.north?;
        let east = self.east?;
        let south = self.south?;
        let west = self.west?;
        Some([north, east, south, west])
    }
    pub fn put(&mut self, trump_color: Color, position: Position, card: &Card) -> bool {
        println!(
            "Turn put card {card} for position {position}, master card was {:?}",
            self.master_card()
        );
        match self.master_card() {
            None => {
                println!("First card is {card}, so player {position} becomes master");
                self.called_color = Some(card.color());
                self.master_position = position;
            }
            Some(master_card) => {
                if master_card.master(*card, trump_color) {
                    println!(
                        "Master card is {master_card}, so player {} stays master",
                        self.master_position
                    );
                } else {
                    println!("Master card is {card}, so player {position} becomes master");
                    self.master_position = position;
                }
            }
        }
        match position {
            Position::North => {
                if self.north.is_some() {
                    return false;
                }
                self.north = Some(*card);
            }
            Position::East => {
                if self.east.is_some() {
                    return false;
                }
                self.east = Some(*card);
            }
            Position::South => {
                if self.south.is_some() {
                    return false;
                }
                self.south = Some(*card);
            }
            Position::West => {
                if self.west.is_some() {
                    return false;
                }
                self.west = Some(*card);
            }
        }
        true
    }
    pub const fn is_first(&self) -> bool {
        self.called_color.is_none()
    }
    pub const fn finished(&self) -> bool {
        self.north.is_some() && self.south.is_some() && self.east.is_some() && self.west.is_some()
    }
    pub const fn master_position(&self) -> Position {
        self.master_position
    }
    pub const fn master_card(&self) -> Option<Card> {
        match self.master_position {
            Position::North => self.north,
            Position::East => self.east,
            Position::South => self.south,
            Position::West => self.west,
        }
    }
    pub fn master_color(&self) -> Option<Color> {
        self.master_card().map(|card| card.color())
    }
    pub const fn master_team(&self) -> Team {
        self.master_position.team()
    }
    pub const fn called_color(&self) -> Option<Color> {
        self.called_color
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Turn: {}", self.number)?;
        for position in self.order {
            let card = match position {
                Position::North => self.north,
                Position::East => self.east,
                Position::South => self.south,
                Position::West => self.west,
            };
            if let Some(card) = card {
                writeln!(f, "\t{position}: {card}")?;
            }
        }
        if let Some(card) = self.master_card() {
            write!(f, "\nMaster card: {card} ({})", self.master_position)?;
        }
        writeln!(f)
    }
}
