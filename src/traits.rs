use crate::order::Order;

pub trait PlayingOrder {
    fn order(&self) -> Order;
}
