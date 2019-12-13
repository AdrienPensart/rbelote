extern crate itertools;
extern crate rand;
#[macro_use] extern crate strum_macros;
#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;

pub mod helpers;
pub mod traits;
pub mod errors;
pub mod card;
pub mod deck;
pub mod player;
pub mod turn;
pub mod game;

use std::error;
use std::thread;
use structopt::StructOpt;

lazy_static! {
    static ref DEFAULT_CONCURRENCY: String = num_cpus::get().to_string();
}

#[derive(StructOpt, Debug)]
#[structopt(name = "RCoinche", about = "Coinche simulation", version = "1.0", author = "Adrien P. <crunchengine@gmail.com>")]
struct Opt {
    /// Random playing mode
    #[structopt(short = "r", long = "random")]
    random: bool,

    /// Auto playing mode when possible
    #[structopt(short = "a", long = "auto")]
    auto: bool,

    /// Test mode
    #[structopt(short = "t", long = "test")]
    test: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[structopt(short, default_value = DEFAULT_CONCURRENCY.as_str())]
    concurrency: usize
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = Opt::from_args();
    if opt.test {
        let mut children = vec![];
        for _ in 0..opt.concurrency {
            children.push(thread::spawn(move || {
                helpers::test_game();
            }));
        }
        for child in children {
            let _ = child.join();
        }
    } else {
        let mut game = game::Game::new(opt.random, opt.auto);
        loop {
            game.distribute();
            game.bidding()?;
            if game.passed() {
                println!("Everyone passed !");
                continue
            }
            while !game.finished() {
                game.play()?;
            }
            game.count_points()?;
            break
        }
        println!("GAME ENDED");
        println!("{}", &game);
    }
    Ok(())
}
