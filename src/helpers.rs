use crate::bidding::PlayOrNext;
use crate::errors::BeloteErrorKind;
use crate::order::Order;
use crate::player::Player;
use crate::players::Players;

pub fn test_game(games: u64) -> Result<(), BeloteErrorKind> {
    use crate::game::{Game, PlayingResult};
    let players = Players::new(
        Player::new(true),
        Player::new(true),
        Player::new(true),
        Player::new(true),
    );
    let order = Order::random();
    let mut game = Game::default(players, order);
    for _ in 0..games {
        let distribution = game.distribute();
        let bidding = distribution.bidding()?;
        game = match bidding.playing_game_or_redistribute() {
            PlayOrNext::NextGame(next_game) => next_game,
            PlayOrNext::PlayGame(in_game) => match in_game.play()? {
                PlayingResult::NextGame(next_game) => next_game,
                PlayingResult::Interrupted => return Ok(()),
            },
            PlayOrNext::Interrupted => return Ok(()),
        };
    }
    Ok(())
}
