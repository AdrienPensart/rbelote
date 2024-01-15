extern crate itertools;
extern crate rand;
#[macro_use]
extern crate strum_macros;
use crate::player::{Player, Position};
use clap::Parser;
use fixed_map::Map;
use inquire::Confirm;
use std::error;
use std::num::NonZeroUsize;
use std::thread;

pub mod bidding;
pub mod card;
pub mod contract;
pub mod deck;
pub mod distribution;
pub mod errors;
pub mod game;
pub mod helpers;
pub mod in_game;
pub mod player;
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
                helpers::test_game();
            }));
        }
        for child in children {
            let _ = child.join();
        }
    } else {
        let mut players = Map::new();
        players.insert(Position::North, Player::new(!opts.human_north));
        players.insert(Position::East, Player::new(!opts.human_east));
        players.insert(Position::South, Player::new(!opts.human_south));
        players.insert(Position::West, Player::new(!opts.human_west));

        let mut game = game::Game::new(players)?;
        'current_game: loop {
            let distribution = game.distribute();
            let bidding = distribution.create_bidding()?;
            game = match bidding.start_game_or_next() {
                (Some(old_game), None) => old_game,
                (None, Some(in_game)) => in_game.play()?,
                _ => {
                    println!("A game can only be played or redistributed");
                    break 'current_game;
                }
            };

            for (team, points) in game.team_total_points() {
                println!(
                    "Game number {}, team {} = {} points",
                    game.number, team, points
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
