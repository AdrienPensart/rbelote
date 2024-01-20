use crate::card::Color;
use crate::order::Order;

pub trait PlayingOrder {
    fn order(&self) -> Order;
}

pub trait BeloteRebelote {
    fn belote_rebelote(&self, trump_color: Color) -> bool;
}
