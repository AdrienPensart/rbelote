extern crate itertools;
extern crate rand;
#[macro_use]
extern crate strum_macros;
use crate::bidding::BiddingResult;
use crate::game::{Game, PlayingResult};
use crate::player::Player;
use crate::players::Players;
use crate::team::Team;
use clap::Parser;
use inquire::Confirm;
use std::error;
use std::num::NonZeroUsize;
use std::thread;
use strum::IntoEnumIterator;

pub mod bidding;
pub mod card;
pub mod contract;
pub mod deck;
pub mod distribution;
pub mod errors;
pub mod game;
pub mod hands;
pub mod helpers;
pub mod order;
pub mod player;
pub mod players;
pub mod points;
pub mod position;
pub mod team;
pub mod traits;
pub mod turn;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Opts {
    /// Is North human ?
    #[arg(long = "north", default_value_t = false)]
    human_north: bool,

    /// Is East human ?
    #[arg(long = "east", default_value_t = false)]
    human_east: bool,

    /// Is South human ?
    #[arg(long = "south", default_value_t = false)]
    human_south: bool,

    /// Is West human ?
    #[arg(long = "west", default_value_t = false)]
    human_west: bool,

    /// Test mode
    #[arg(short = 't', long = "test", default_value_t = false)]
    test: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[arg(short = 'c', long = "concurrency", default_value_t = thread::available_parallelism().unwrap())]
    concurrency: NonZeroUsize,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opts = Opts::parse();
    if opts.test {
        let mut children = vec![];
        for _ in 0..opts.concurrency.get() {
            children.push(thread::spawn(move || {
                helpers::test_game().unwrap();
            }));
        }
        for child in children {
            let _ = child.join();
        }
    } else {
        let players = Players {
            north: Player::new(!opts.human_north),
            south: Player::new(!opts.human_south),
            east: Player::new(!opts.human_east),
            west: Player::new(!opts.human_west),
        };

        let mut game = Game::new(players);
        'current_game: loop {
            let distribution = game.distribute();
            let bidding = distribution.bidding()?;
            game = match bidding.playing_game_or_redistribute() {
                BiddingResult::NextGame(next_game) => next_game,
                BiddingResult::PlayGame(in_game) => match in_game.play()? {
                    PlayingResult::NextGame(next_game) => next_game,
                    PlayingResult::Interrupted => break 'current_game,
                },
                BiddingResult::Interrupted => break 'current_game,
            };

            for team in Team::iter() {
                println!(
                    "Game number {}, team {} = {} points",
                    game.number(),
                    team,
                    game.points()[team],
                );
            }

            loop {
                let answer = Confirm::new("Continue to play ? (ESC to cancel)")
                    .with_default(true)
                    .prompt_skippable();

                match answer {
                    Ok(Some(true)) => {
                        continue 'current_game;
                    }
                    Ok(Some(false)) => {
                        break 'current_game;
                    }
                    Ok(None) => {
                        println!("Interrupted.");
                        break 'current_game;
                    }
                    Err(_) => {
                        println!("Error with questionnaire, try again.");
                    }
                };
            }
        }
        println!("GAME ENDED");
    }
    Ok(())
}
