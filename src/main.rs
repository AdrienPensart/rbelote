extern crate itertools;
extern crate rand;
#[macro_use]
extern crate strum_macros;
use crate::bidding::PlayOrNext;
use crate::game::Game;
use crate::order::Order;
use crate::player::Player;
use crate::players::Players;
use crate::playing::NextGameOrInterrupt;
use crate::team::Team;
use clap::Parser;
use color_eyre::eyre::Result;
use inquire::Confirm;
use std::error;
use std::num::NonZeroUsize;
use std::thread;
use strum::IntoEnumIterator;
use tracing::{error, info};

pub mod bidding;
pub mod card;
pub mod constants;
pub mod contract;
pub mod deck;
pub mod distribution;
pub mod errors;
pub mod game;
pub mod hands;
pub mod helpers;
pub mod initial;
pub mod order;
pub mod player;
pub mod players;
pub mod playing;
pub mod points;
pub mod position;
pub mod stack;
pub mod state;
pub mod team;
pub mod turn;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Opts {
    /// Number of games to play
    #[arg(long = "games", default_value_t = 1)]
    games: u64,

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

    /// Random order ?
    #[arg(long = "random-order", default_value_t = false)]
    random_order: bool,

    /// Test mode
    #[arg(short = 't', long = "test", default_value_t = false)]
    test: bool,

    /// Tracing ?
    #[arg(long = "trace", default_value_t = false)]
    tracing: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[arg(short = 'c', long = "concurrency", default_value_t = thread::available_parallelism().unwrap())]
    concurrency: NonZeroUsize,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    color_eyre::install()?;
    let opts = Opts::parse();
    if opts.tracing {
        tracing_subscriber::fmt::init();
    }
    if opts.test {
        let mut children = vec![];
        for _ in 0..opts.concurrency.get() {
            let games = opts.games;
            children.push(thread::spawn(move || {
                if let Err(e) = helpers::test_game(games) {
                    error!("{e}");
                }
            }));
        }
        for child in children {
            let _ = child.join();
        }
    } else {
        let players = Players::new(
            Player::new(!opts.human_north),
            Player::new(!opts.human_south),
            Player::new(!opts.human_east),
            Player::new(!opts.human_west),
        );

        let order = if opts.random_order {
            Order::random()
        } else {
            Order::default()
        };

        let mut game = Game::default(players, order);
        'current_game: for _ in 0..opts.games {
            let distribution = game.distribute();
            let bidding = distribution.bidding()?;
            game = match bidding.playing_game_or_redistribute() {
                PlayOrNext::NextGame(next_game) => next_game,
                PlayOrNext::PlayGame(in_game) => match in_game.play()? {
                    NextGameOrInterrupt::NextGame(next_game) => next_game,
                    NextGameOrInterrupt::Interrupted => break 'current_game,
                },
                PlayOrNext::Interrupted => break 'current_game,
            };

            for team in Team::iter() {
                info!(
                    "Game number {}, team {} = {} points",
                    game.number(),
                    team,
                    game.points()[team],
                );
            }

            if game.is_full_random() {
                continue;
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
                        info!("Interrupted.");
                        break 'current_game;
                    }
                    Err(_) => {
                        error!("Error with questionnaire, try again.");
                    }
                };
            }
        }
        info!("GAME ENDED");
    }
    Ok(())
}
