use failure::Context;
#[derive(Debug)]
pub struct BeloteError {
    inner: Context<BeloteErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum BeloteErrorKind {
    #[fail(display = "Invalid mode")]
    InvalidMode,
    #[fail(display = "No contract")]
    NoContract,
    #[fail(display = "Invalid contract")]
    InvalidContract,
    #[fail(display = "Invalid case")]
    InvalidCase,
    #[fail(display = "Invalid color")]
    InvalidColor,
    #[fail(display = "No taker or auctions not finished")]
    NoTaker,
    #[fail(display = "A player shoud belongs to a team")]
    NoTeam,
}
