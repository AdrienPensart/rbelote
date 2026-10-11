//! A bot that plays by a handful of rules of thumb.
//!
//! It is not clever, but it is not random either: it takes on the strength of
//! its hand with the proposed trump, gives points to a partner who is master,
//! takes a trick with the cheapest card that can, and dumps its least valuable
//! card otherwise. Everything it looks at is public to the seat it plays for:
//! its own hand and the cards on the table.

use crate::card::{Card, Color, Value};
use crate::contract::Contract;
use crate::errors::BeloteErrorKind;
use crate::game::{Answer, Game, Request};
use crate::hand::Hand;
use crate::position::Position;
use crate::turn::Turn;
use rand::seq::IteratorRandom;
use rand::{rng, RngExt};

/// Strength a hand must reach to take on the first bidding turn, when the
/// returned card comes with the trump.
pub const TAKE_THRESHOLD: u64 = 54;
/// Strength a hand must reach to name a trump on the second turn.
pub const NAME_THRESHOLD: u64 = 50;

/// The reply a sensible, if unimaginative, player would give.
pub fn answer(game: &Game, request: &Request) -> Result<Answer, BeloteErrorKind> {
    let position = request.position();
    let hand = game.hand(position);
    match request {
        Request::Take { card_returned, .. } => Ok(Answer::Take(
            strength(hand, Some(*card_returned), card_returned.color()) >= TAKE_THRESHOLD,
        )),
        Request::Trump {
            card_returned,
            choices,
            ..
        } => Ok(Answer::Trump(name(hand, *card_returned, choices))),
        Request::Play { choices, .. } => {
            let turn = game
                .turn()
                .ok_or_else(|| BeloteErrorKind::InvalidCase("no turn to play".to_string()))?;
            let trump = game.trump_color().ok_or(BeloteErrorKind::NoTaker)?;
            play(choices, position, turn, trump)
                .map(Answer::Play)
                .ok_or_else(|| BeloteErrorKind::InvalidCase("nothing to play".to_string()))
        }
    }
}

/// A legal answer drawn at random, for tests and for the dumbest of bots.
/// Takes about once per deal at a table of four of them.
pub fn random_answer(request: &Request) -> Result<Answer, BeloteErrorKind> {
    match request {
        Request::Take { .. } => Ok(Answer::Take(rng().random_bool(0.25))),
        Request::Trump { choices, .. } => {
            if !rng().random_bool(0.25) {
                return Ok(Answer::Trump(Contract::Pass));
            }
            Ok(choices
                .iter()
                .filter(|contract| contract.color().is_some())
                .choose(&mut rng())
                .map_or(Answer::Trump(Contract::Pass), |contract| {
                    Answer::Trump(*contract)
                }))
        }
        Request::Play { choices, .. } => choices
            .iter()
            .choose(&mut rng())
            .map(|card| Answer::Play(*card))
            .ok_or_else(|| BeloteErrorKind::InvalidCase("no legal card".to_string())),
    }
}

// ---- weighing a hand -------------------------------------------------------

/// How much a hand is worth with `trump` as trump: its card points, plus a
/// bonus for every trump held and every ace, which win tricks on their own.
pub fn strength(hand: &Hand, extra: Option<Card>, trump: Color) -> u64 {
    hand.into_iter()
        .flatten()
        .chain(extra)
        .map(|card| {
            let bonus = if card.color() == trump {
                5
            } else if card.value() == Value::As {
                3
            } else {
                0
            };
            card.points(trump) + bonus
        })
        .sum()
}

/// Second turn: the strongest biddable color, if it is strong enough.
fn name(hand: &Hand, card_returned: Card, choices: &[Contract]) -> Contract {
    choices
        .iter()
        .filter_map(|contract| contract.color().map(|color| (*contract, color)))
        .map(|(contract, color)| (strength(hand, Some(card_returned), color), contract))
        .filter(|(strength, _)| *strength >= NAME_THRESHOLD)
        .max_by_key(|(strength, _)| *strength)
        .map_or(Contract::Pass, |(_, contract)| contract)
}

// ---- playing a trick -------------------------------------------------------

/// Whether `card` would take the trick from `master`, the card now winning it.
pub fn beats(card: Card, master: Card, trump: Color) -> bool {
    if card.color() == trump && master.color() != trump {
        return true;
    }
    card.color() == master.color() && card.power(trump) > master.power(trump)
}

/// The card to lead with, when the trick is empty: a plain ace, else the
/// jack of trump, else the least valuable card.
fn lead(choices: &[Card], trump: Color) -> Option<Card> {
    choices
        .iter()
        .find(|card| card.value() == Value::As && card.color() != trump)
        .or_else(|| {
            choices
                .iter()
                .find(|card| card.value() == Value::Jack && card.color() == trump)
        })
        .copied()
        .or_else(|| cheapest(choices, trump))
}

/// The card worth the least: fewest points, then lowest rank, trumps last.
fn cheapest(choices: &[Card], trump: Color) -> Option<Card> {
    choices
        .iter()
        .min_by_key(|card| (card.color() == trump, card.points(trump), card.power(trump)))
        .copied()
}

/// The card worth the most to give to a partner who is master.
fn richest(choices: &[Card], trump: Color) -> Option<Card> {
    choices
        .iter()
        .max_by_key(|card| (card.color() != trump, card.points(trump), card.power(trump)))
        .copied()
}

fn play(choices: &[Card], position: Position, turn: &Turn, trump: Color) -> Option<Card> {
    let Some(master) = turn.master_card() else {
        return lead(choices, trump);
    };
    if turn.master_team() == position.team() {
        // A partner is master: feed the trick, or follow as low as the rules allow.
        let all_trumps = choices.iter().all(|card| card.color() == trump);
        return if all_trumps {
            cheapest(choices, trump)
        } else {
            richest(choices, trump)
        };
    }
    // An opponent is master: take the trick with the cheapest card that can.
    let winner = choices
        .iter()
        .filter(|card| beats(**card, master, trump))
        .min_by_key(|card| (card.points(trump), card.power(trump)))
        .copied();
    winner.or_else(|| cheapest(choices, trump))
}

#[cfg(test)]
mod tests {
    use super::{answer, beats, random_answer, strength};
    use crate::card::{Card, Color, Value};
    use crate::game::{Event, Game, Request};
    use crate::hand::Hand;
    use crate::order::Order;

    fn card(color: Color, value: Value) -> Card {
        let Some(card) = Card::new(color, value) else {
            panic!("cannot build card");
        };
        card
    }

    fn hand_of(cards: &[Card]) -> Hand {
        let mut hand = Hand::default();
        for card in cards {
            assert!(hand.take(*card).is_ok());
        }
        hand
    }

    #[test]
    fn a_trump_beats_a_plain_card_and_a_higher_card_beats_a_lower_one() {
        let trump = Color::Spade;
        assert!(beats(
            card(Color::Spade, Value::_7),
            card(Color::Heart, Value::As),
            trump
        ));
        assert!(!beats(
            card(Color::Heart, Value::As),
            card(Color::Spade, Value::_7),
            trump
        ));
        assert!(beats(
            card(Color::Heart, Value::As),
            card(Color::Heart, Value::_10),
            trump
        ));
        // another plain color never takes the trick
        assert!(!beats(
            card(Color::Club, Value::As),
            card(Color::Heart, Value::_7),
            trump
        ));
        assert!(beats(
            card(Color::Spade, Value::Jack),
            card(Color::Spade, Value::_9),
            trump
        ));
    }

    #[test]
    fn a_hand_full_of_trumps_is_strong() {
        let strong = hand_of(&[
            card(Color::Spade, Value::Jack),
            card(Color::Spade, Value::_9),
            card(Color::Spade, Value::As),
            card(Color::Heart, Value::_7),
            card(Color::Club, Value::_8),
        ]);
        let weak = hand_of(&[
            card(Color::Heart, Value::_7),
            card(Color::Heart, Value::_8),
            card(Color::Club, Value::_7),
            card(Color::Club, Value::_8),
            card(Color::Diamond, Value::_7),
        ]);
        assert!(strength(&strong, None, Color::Spade) >= super::TAKE_THRESHOLD);
        assert!(strength(&weak, None, Color::Spade) < super::NAME_THRESHOLD);
        assert!(strength(&strong, None, Color::Spade) > strength(&strong, None, Color::Heart));
    }

    /// Four bots play many deals without ever being refused, and take often
    /// enough that the table is not forever dealing again.
    #[test]
    fn bots_play_whole_games() {
        let Ok(mut game) = Game::new(Order::default()) else {
            panic!("cannot start");
        };
        let mut scored = 0;
        let mut redeals = 0;
        let mut guard = 0;
        while scored < 200 {
            guard += 1;
            assert!(guard < 200_000, "the bots never finish");
            let request = game.request();
            let Ok(answer) = answer(&game, &request) else {
                panic!("the bot has no answer to {request:?}");
            };
            match game.answer(answer) {
                Ok(Event::DealEnded(_)) => scored += 1,
                Ok(Event::Redeal) => redeals += 1,
                Ok(Event::Continue | Event::TrickWon(_)) => {}
                Err(error) => panic!("the engine refused {answer:?}: {error}"),
            }
        }
        assert!(
            redeals < scored,
            "{redeals} redeals for {scored} deals: the bots are too shy"
        );
    }

    #[test]
    fn random_answers_are_legal() {
        let Ok(mut game) = Game::new(Order::default()) else {
            panic!("cannot start");
        };
        for _ in 0..2_000 {
            let request = game.request();
            let Ok(answer) = random_answer(&request) else {
                panic!("no random answer to {request:?}");
            };
            if let Request::Play { choices, .. } = &request {
                assert!(
                    matches!(answer, crate::game::Answer::Play(card) if choices.contains(&card))
                );
            }
            assert!(game.answer(answer).is_ok());
        }
    }
}
