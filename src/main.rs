extern crate itertools;
extern crate rand;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate lazy_static;

use crate::player::{Player, Position};
use fixed_map::Map;
use inquire::Confirm;

pub mod card;
pub mod deck;
pub mod errors;
pub mod game;
pub mod helpers;
pub mod player;
pub mod turn;

use clap::Parser;
use std::error;
use std::thread;

lazy_static! {
    static ref DEFAULT_CONCURRENCY: String = num_cpus::get().to_string();
}

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
    #[arg(short = 't', long = "test")]
    test: bool,

    /// Infinite mode
    #[arg(long = "infinite")]
    infinite: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[arg(short = 'c', long = "concurrency", default_value = DEFAULT_CONCURRENCY.as_str())]
    concurrency: usize,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opts = Opts::parse();
    if opts.test {
        let mut children = vec![];
        for _ in 0..opts.concurrency {
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
        'game: loop {
            let trump = game.distribute();
            if !game.bidding(trump) {
                println!("Everyone passed !");
            } else {
                game.play()?;
            }

            for (team, points) in game.team_points() {
                println!(
                    "Game number {}, team {} = {} points",
                    game.number(),
                    team,
                    points
                );
            }

            if opts.infinite {
                continue;
            }

            loop {
                let answer = Confirm::new("Continue to play ? (ESC to cancel)")
                    .with_default(true)
                    .prompt_skippable();

                match answer {
                    Ok(Some(true)) => {
                        continue 'game;
                    }
                    Ok(Some(false)) => {
                        break 'game;
                    }
                    Ok(None) => {
                        println!("Interrupted.");
                        break 'game;
                    }
                    Err(_) => {
                        println!("Error with questionnaire, try again.");
                    }
                };
            }
        }
        println!("GAME ENDED");
        println!("{}", game);
    }
    Ok(())
}
