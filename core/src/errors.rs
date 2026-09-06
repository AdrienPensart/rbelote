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
    #[error("Unexpected answer : {0}")]
    UnexpectedAnswer(String),
    #[error("Illegal card : {0}")]
    IllegalCard(String),
    #[error("Illegal contract : {0}")]
    IllegalContract(String),
}

pub trait ErrOnSome {
    fn err_on_some<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<(), E>;
}

impl<T> ErrOnSome for Option<T> {
    fn err_on_some<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<(), E>,
    {
        match self {
            None => Ok(()),
            Some(_) => f(),
        }
    }
}
