//! Belote rules engine.
//!
//! The engine performs no input/output: it never reads a terminal, never
//! blocks, and never decides what a player wants. Instead it exposes a
//! [`Game`](game::Game) that answers "what do you need next?" with a
//! [`Request`](game::Request), and is driven forward with
//! [`Answer`](game::Answer)s. A terminal, a browser or an AI can all drive it
//! the same way.

pub mod bot;
pub mod card;
pub mod constants;
pub mod contract;
pub mod errors;
pub mod game;
pub mod hand;
pub mod hands;
pub mod order;
pub mod points;
pub mod position;
pub mod rules;
pub mod score;
pub mod stack;
pub mod team;
pub mod turn;
