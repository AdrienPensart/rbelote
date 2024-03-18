// use crate::card::Card;
// use crate::deck::Deck;
// use crate::order::Order;
// use crate::player::Player;
// use crate::players::Players;
// #[allow(clippy::wildcard_imports)]
// use belote::*;
// use typestate::typestate;

// #[allow(clippy::module_inception)]
// #[typestate]
// mod belote {
//     use crate::card::Card;
//     use crate::deck::Deck;
//     use crate::order::Order;
//     use crate::players::Players;
//     use crate::points::Points;
//     use crate::stack::Iter as StackIter;
//     use arrayvec::ArrayVec;
//     use derive_more::{Constructor, Deref, DerefMut, Into, IntoIterator};
//     use derive_new::new;

//     #[derive(Constructor)]
//     #[automaton]
//     pub struct Belote;

//     #[derive(new, IntoIterator)]
//     #[state]
//     pub struct Initial {
//         players: Players,
//         order: Order,
//         #[new(default)]
//         points: Points,
//         #[new(default)]
//         number: u64,
//         #[new(default)]
//         #[into_iterator]
//         stack_iter: StackIter,
//         stack: ArrayVec<Card, 32>,
//         #[new(default)]
//         litige: u64,
//     }

//     impl Initial {
//         pub fn next(self) -> Self {
//             Self {
//                 litige: self.litige,
//                 number: self.number + 1,
//                 order: self.order.rotate(),
//                 players: self.players,
//                 points: self.points,
//                 stack_iter: StackIter::from_deck(
//                     Deck::new(self.into_iter().collect::<Vec<Card>>()).cut(),
//                 ),
//             }
//         }
//     }
//     #[derive(Constructor, Deref, DerefMut, IntoIterator, Into)]
//     #[state]
//     pub struct Distribution {
//         #[into_iterator]
//         #[into]
//         initial: Initial,
//     }

//     #[derive(Constructor, Deref, DerefMut, IntoIterator, Into)]
//     #[state]
//     pub struct Bidding {
//         #[into]
//         #[deref]
//         #[deref_mut]
//         #[into_iterator]
//         initial: Initial,
//     }

//     #[derive(Constructor, Deref, DerefMut, IntoIterator, Into)]
//     #[state]
//     pub struct Playing {
//         #[into]
//         #[deref]
//         #[deref_mut]
//         #[into_iterator]
//         initial: Initial,
//     }

//     pub trait Initial {
//         fn start() -> Initial;
//         fn to_distribution(self) -> Distribution;
//         fn stop(self);
//     }

//     pub trait Distribution {
//         fn to_bidding(self) -> Bidding;
//     }

//     pub trait Bidding {
//         fn to_initial(self) -> Initial;
//         fn to_play(self) -> Playing;
//         fn stop(self);
//     }

//     pub trait Playing {
//         fn to_initial(self) -> Initial;
//         fn stop(self);
//     }

//     #[test]
//     fn belote_tests() {
//         let initial = Belote::<Initial>::start();
//         let distribution = initial.to_distribution();
//         let bidding = distribution.to_bidding();
//         let _ = bidding.to_play();
//     }
// }

// impl InitialState for Belote<Initial> {
//     fn start() -> Belote<Initial> {
//         let players = Players::new(
//             Player::new(true),
//             Player::new(true),
//             Player::new(true),
//             Player::new(true),
//         );
//         Self {
//             state: Initial::new(players, Order::random()),
//         }
//     }
//     fn to_distribution(self) -> Belote<Distribution> {
//         Belote {
//             state: Distribution::new(self.state),
//         }
//     }
//     fn stop(self) {}
// }

// impl DistributionState for Belote<Distribution> {
//     fn to_bidding(self) -> Belote<Bidding> {
//         Belote {
//             state: Bidding::new(self.state.into()),
//         }
//     }
// }

// impl BiddingState for Belote<Bidding> {
//     fn to_play(self) -> Belote<Playing> {
//         Belote {
//             state: Playing::new(self.state.into()),
//         }
//     }
//     fn to_initial(self) -> Belote<Initial> {
//         // let deck = Deck::new(self.state.into_iter().collect::<Vec<Card>>());
//         Belote {
//             state: self.state.into(),
//         }
//     }
//     fn stop(self) {}
// }

// impl PlayingState for Belote<Playing> {
//     fn stop(self) {}
//     fn to_initial(self) -> Belote<Initial> {
//         // let deck = Deck::new(self.state.into_iter().collect::<Vec<Card>>());
//         Belote {
//             state: self.state.into(),
//         }
//     }
// }
