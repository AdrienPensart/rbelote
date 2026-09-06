use crate::driver::Decision;
use crate::players::{Player, Players};
use clap::Parser;
use color_eyre::eyre::Result;
use inquire::Confirm;
use rbelote_core::game::{Event, Game};
use rbelote_core::order::Order;
use rbelote_core::team::Team;
use std::error;
use std::num::NonZeroUsize;
use std::thread;
use strum::IntoEnumIterator;
use tracing::{error, info};

pub mod driver;
pub mod helpers;
pub mod players;

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
        return Ok(());
    }

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

    let mut game = Game::new(order)?;
    'current_game: for _ in 0..opts.games {
        // one deal: keep answering whatever the engine asks for
        loop {
            let request = game.request();
            let answer = match driver::decide(&game, players, &request)? {
                Decision::Answer(answer) => answer,
                Decision::Interrupted => {
                    info!("Interrupted.");
                    break 'current_game;
                }
            };
            match game.answer(answer)? {
                Event::Continue => {}
                Event::TrickWon(position) => info!("{position} takes the fold"),
                Event::Redeal => {
                    info!("Nobody took, dealing again");
                    break;
                }
                Event::DealEnded(outcome) => {
                    info!(
                        "{} took at {}, attack {} / defense {}",
                        outcome.taker,
                        driver::colorize(outcome.trump_color),
                        outcome.score.attack,
                        outcome.score.defense
                    );
                    break;
                }
            }
        }

        for team in Team::iter() {
            info!(
                "Game number {}, team {} = {} points",
                game.number(),
                team,
                game.points()[team],
            );
        }

        if players.full_random() {
            continue;
        }
        loop {
            match Confirm::new("Continue to play ? (ESC to cancel)")
                .with_default(true)
                .prompt_skippable()
            {
                Ok(Some(true)) => continue 'current_game,
                Ok(Some(false)) => break 'current_game,
                Ok(None) => {
                    info!("Interrupted.");
                    break 'current_game;
                }
                Err(_) => error!("Error with questionnaire, try again."),
            }
        }
    }
    info!("GAME ENDED");
    Ok(())
}
