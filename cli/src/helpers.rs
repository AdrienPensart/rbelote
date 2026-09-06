use crate::driver;
use crate::players::{Player, Players};
use rbelote_core::errors::BeloteErrorKind;
use rbelote_core::game::{Event, Game};
use rbelote_core::order::Order;

/// Plays `games` deals with four random players, as fast as the engine allows.
pub fn test_game(games: u64) -> Result<(), BeloteErrorKind> {
    let players = Players::new(
        Player::new(true),
        Player::new(true),
        Player::new(true),
        Player::new(true),
    );
    let mut game = Game::new(Order::random())?;
    let mut played = 0;
    while played < games {
        let request = game.request();
        let answer = driver::random(players, &request)?;
        match game.answer(answer)? {
            // a deal that nobody took still counts as one dealt hand
            Event::Redeal | Event::DealEnded(_) => played += 1,
            Event::Continue | Event::TrickWon(_) => {}
        }
    }
    Ok(())
}
