use crate::bidding::BiddingResult;
use crate::errors::BeloteErrorKind;
use crate::player::Player;
use crate::players::Players;
use rand::Rng;
use std::io;

pub fn gen_index<R: Rng + ?Sized>(rng: &mut R, ubound: usize) -> usize {
    if ubound <= (core::u32::MAX as usize) {
        rng.gen_range(0..ubound as u32) as usize
    } else {
        rng.gen_range(0..ubound)
    }
}

pub fn read_index() -> usize {
    let mut input = String::new();
    loop {
        if io::stdin().read_line(&mut input).is_ok() {
            return input.trim().parse::<usize>().unwrap();
        }
    }
}

pub fn wait_input() {
    use std::io::prelude::*;
    let mut stdin = io::stdin();
    let _ = stdin.read(&mut [0u8]).unwrap();
}

pub fn test_game() -> Result<(), BeloteErrorKind> {
    use crate::game::{Game, PlayingResult};
    let players = Players {
        north: Player::new(true),
        south: Player::new(true),
        east: Player::new(true),
        west: Player::new(true),
    };
    let mut game = Game::new(players);
    loop {
        let distribution = game.distribute();
        let bidding = distribution.bidding()?;
        game = match bidding.playing_game_or_redistribute() {
            BiddingResult::NextGame(next_game) => next_game,
            BiddingResult::PlayGame(in_game) => match in_game.play()? {
                PlayingResult::NextGame(next_game) => next_game,
                PlayingResult::Interrupted => return Ok(()),
            },
            BiddingResult::Interrupted => return Ok(()),
        };
    }
}
