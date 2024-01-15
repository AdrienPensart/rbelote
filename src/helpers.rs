use crate::player::{Player, Position};
use fixed_map::Map;
use std::io;

#[allow(clippy::redundant_closure)]
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

pub fn test_game() {
    use crate::game::Game;
    let mut players = Map::new();
    players.insert(Position::North, Player::new(true));
    players.insert(Position::East, Player::new(true));
    players.insert(Position::South, Player::new(true));
    players.insert(Position::West, Player::new(true));
    loop {
        let mut game = Game::new(players.clone()).unwrap();
        let card = game.distribute();
        if game.bidding(card) {
            game.play().unwrap();
        }
        // assert!(game.count_points().is_ok());
    }
}
