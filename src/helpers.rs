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
    loop {
        let mut game = Game::new(true, true);
        game.distribute();
        assert!(game.bidding().is_ok());
        if game.passed() {
            continue
        }
        while !game.finished() {
            assert!(game.play().is_ok());
        }
        assert!(game.count_points().is_ok());
        break
    }
}
