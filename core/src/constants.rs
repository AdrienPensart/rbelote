pub const MAX_CARDS: usize = 32;
pub const MAX_PLAYERS: usize = 4;
pub const MAX_CARDS_BY_PLAYER: usize = 8;
/// A cut leaves at least this many cards on each side.
pub const MIN_CUT: usize = 3;

/// All the card points of a deal: 152 in the cards themselves, plus the
/// 10 points of the last trick ("dix de der").
pub const ALL_CARD_POINTS: u64 = 162;
/// Awarded to the team winning the last trick.
pub const LAST_TRICK_POINTS: u64 = 10;
/// Awarded to a team winning every single trick.
pub const CAPOT_POINTS: u64 = 252;
/// King + Queen of trump held by the same player.
pub const BELOTE_POINTS: u64 = 20;
