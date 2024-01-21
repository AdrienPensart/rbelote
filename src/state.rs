// use crate::bidding::Bidding;
// use crate::distribution::Distribution;
// use crate::initial::Initial;
// use crate::players::Players;
// use crate::playing::Playing;
// use crate::points::Points;

// pub struct Game {
//     players: Players,
//     points: Points,
// }

// pub enum State {
//     Initial(Initial),
//     Distribution(Distribution),
//     Bidding(Bidding),
//     Playing(Playing),
//     Interrupted,
// }

// pub enum PlayOrNext {
//     NextGame(Game<Initial>),
//     PlayGame(Game<Playing>),
//     Interrupted,
// }

// pub trait State {
//     fn next(self) {}
// }

// initial =>
//     => interrupt
//     => distribution
//         => bidding
//             => play
//             => distribution
//             => interrupt
