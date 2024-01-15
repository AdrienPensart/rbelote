use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum BeloteErrorKind {
    #[error("Invalid case : {0}")]
    InvalidCase(String),
    #[error("Invalid color")]
    InvalidColor,
    #[error("No taker or auctions not finished")]
    NoTaker,
}
