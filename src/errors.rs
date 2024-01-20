use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum BeloteErrorKind {
    #[error("Invalid case : {0}")]
    InvalidCase(String),
    #[error("Invalid color")]
    InvalidColor(String),
    #[error("Invalid card")]
    InvalidCard(String),
    #[error("Invalid value")]
    InvalidValue(String),
    #[error("No taker or auctions not finished")]
    NoTaker,
}
