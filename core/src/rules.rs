//! Which cards and which contracts the rules allow, for a given situation.
//!
//! These are the only two places where "what may I do now?" is decided, and
//! they are pure functions: a front-end uses them to grey out illegal moves,
//! and an AI uses them as its search space.

use crate::card::{Card, Color};
use crate::constants::MAX_CARDS_BY_PLAYER;
use crate::contract::Contract;
use crate::errors::BeloteErrorKind;
use crate::hand::Hand;
use crate::position::Position;
use crate::turn::Turn;
use strum::IntoEnumIterator;
use tracing::info;

/// Second bidding turn: any color may be named as trump except the one of the
/// returned card. `Contract::Pass` is kept, it is how a player declines.
pub fn biddable_contracts(card_returned: Card) -> impl Iterator<Item = Contract> {
    Contract::iter().filter(move |contract| contract.color() != Some(card_returned.color()))
}

/// Every card `position` is allowed to play, given what is already on the table.
pub fn choices(
    hand: &Hand,
    position: &Position,
    turn: &Turn,
    trump_color: Color,
) -> Result<Vec<Card>, BeloteErrorKind> {
        if hand.len() as u64 != MAX_CARDS_BY_PLAYER as u64 - turn.number() + 1 {
            return Err(BeloteErrorKind::InvalidCase(format!(
                "Bad number of cards for player {position} : {} it should be {}",
                hand.len(),
                MAX_CARDS_BY_PLAYER as u64 - turn.number() + 1
            )));
        }
        info!("{position} : trump color is {trump_color}");
        info!(
            "called color {:?} : master card {:?}",
            turn.called_color(),
            turn.master_card()
        );
        let choices = match (turn.called_color(), turn.master_card()) {
            (None, None) => hand.into_iter().flatten().collect(),
            (Some(called_color), Some(master_card)) => {
                let mut trumps = Vec::new();
                let mut trumps_less = Vec::new();
                let mut trumps_more = Vec::new();
                let mut other_colors = Vec::new();
                let mut same_colors = Vec::new();

                let master_is_trump = master_card.color() == trump_color;
                for card in hand.into_iter().flatten() {
                    if card.color() == trump_color {
                        trumps.push(card);
                        if master_is_trump
                            && card.power(trump_color) > master_card.power(trump_color)
                        {
                            trumps_more.push(card);
                        } else {
                            trumps_less.push(card);
                        }
                    } else if card.color() == called_color {
                        same_colors.push(card);
                    } else {
                        other_colors.push(card);
                    }
                }

                // info!("trumps: {:?}", trumps);
                // info!("trumps_less: {:?}", trumps_less);
                // info!("trumps_more: {:?}", trumps_more);
                // info!("other_colors: {:?}", other_colors);
                // info!("same_colors: {:?}", same_colors);

                if called_color == trump_color {
                    if !trumps_more.is_empty() {
                        info!("{position} : trump color asked, I must go up");
                        trumps_more
                    } else if !trumps_less.is_empty() {
                        info!("{position} : trump color asked, but can't go up, going down");
                        trumps_less
                    } else {
                        info!("{position} : no trump left I must piss");
                        other_colors
                    }
                } else if !same_colors.is_empty() {
                    info!("{position} : I have asked color");
                    same_colors
                } else if turn.master_team() == position.team() {
                    info!(
                        "{position} : my team ({}) is master, I can play anything",
                        turn.master_team()
                    );
                    hand.into_iter().flatten().collect()
                } else if !trumps.is_empty() {
                    if trumps_more.is_empty() {
                        // either nobody cut yet and any trump wins the fold, or an
                        // opponent cut above all of mine and I must "sous-couper"
                        info!("{position} : I can't give asked color, I must cut with a trump");
                        trumps
                    } else {
                        info!("{position} : an opponent cut, I must go up");
                        trumps_more
                    }
                } else {
                    info!("{position} : no trumps left, I must piss");
                    other_colors
                }
            }
            _ => {
                return Err(BeloteErrorKind::InvalidCase(format!(
                    "no choices for player {position}"
                )))
            }
        };
    Ok(choices)
}

#[cfg(test)]
mod choices_tests {
    use crate::card::{Card, Color, Value};
    use crate::hand::Hand;
    use crate::order::Order;
    use crate::position::Position;
    use crate::turn::Turn;

    const TRUMP: Color = Color::Spade;

    fn card(color: Color, value: Value) -> Card {
        let Some(card) = Card::new(color, value) else {
            panic!("cannot build card");
        };
        card
    }

    fn hand_of(cards: &[Card]) -> Hand {
        let mut hand = Hand::default();
        for card in cards {
            assert!(hand.take(*card).is_ok(), "cannot build hand");
        }
        hand
    }

    /// Cards already on the table, in playing order starting from North.
    fn turn_with(played: &[(Position, Card)]) -> Turn {
        // hands of 3 cards are consistent with the 6th turn: 8 - 6 + 1
        let mut turn = Turn::new(6, Order::default());
        for (position, card) in played {
            assert!(turn.put(TRUMP, *position, card), "cannot put card");
        }
        turn
    }

    fn choices(hand: &Hand, position: Position, turn: &Turn) -> Vec<Card> {
        let Ok(mut choices) = super::choices(hand, &position, turn, TRUMP) else {
            panic!("no choices");
        };
        choices.sort_unstable();
        choices
    }

    fn sorted(cards: &[Card]) -> Vec<Card> {
        let mut cards = cards.to_vec();
        cards.sort_unstable();
        cards
    }

    #[test]
    fn the_asked_color_must_be_supplied() {
        let hand = hand_of(&[
            card(Color::Heart, Value::_7),
            card(Color::Heart, Value::King),
            card(TRUMP, Value::Jack),
        ]);
        let turn = turn_with(&[(Position::North, card(Color::Heart, Value::As))]);
        assert_eq!(
            choices(&hand, Position::East, &turn),
            sorted(&[
                card(Color::Heart, Value::_7),
                card(Color::Heart, Value::King)
            ])
        );
    }

    #[test]
    fn trump_asked_means_going_up() {
        let hand = hand_of(&[
            card(TRUMP, Value::_7),
            card(TRUMP, Value::Jack),
            card(Color::Diamond, Value::King),
        ]);
        let turn = turn_with(&[(Position::North, card(TRUMP, Value::_9))]);
        assert_eq!(
            choices(&hand, Position::East, &turn),
            sorted(&[card(TRUMP, Value::Jack)])
        );
    }

    #[test]
    fn a_partner_master_frees_the_whole_hand() {
        // North is master, South plays: he may discard OR cut, with any card
        let hand = hand_of(&[
            card(TRUMP, Value::_7),
            card(TRUMP, Value::Jack),
            card(Color::Diamond, Value::King),
        ]);
        let turn = turn_with(&[
            (Position::North, card(Color::Heart, Value::As)),
            (Position::East, card(Color::Heart, Value::_7)),
        ]);
        // used to withhold the 7 of trump, keeping only the cards beating an ace
        assert_eq!(
            choices(&hand, Position::South, &turn),
            sorted(&[
                card(TRUMP, Value::_7),
                card(TRUMP, Value::Jack),
                card(Color::Diamond, Value::King)
            ])
        );
    }

    #[test]
    fn an_opponent_cut_must_be_gone_over() {
        // East cut with the 9 of trump, South holds the jack: surcoupe is compulsory
        let hand = hand_of(&[
            card(TRUMP, Value::_7),
            card(TRUMP, Value::Jack),
            card(Color::Diamond, Value::King),
        ]);
        let turn = turn_with(&[
            (Position::North, card(Color::Heart, Value::As)),
            (Position::East, card(TRUMP, Value::_9)),
        ]);
        assert_eq!(
            choices(&hand, Position::South, &turn),
            sorted(&[card(TRUMP, Value::Jack)])
        );
    }

    #[test]
    fn an_unbeatable_cut_still_forces_a_trump() {
        // East cut with the jack: South cannot go over, but must "sous-couper"
        // rather than discard his diamond.
        let hand = hand_of(&[
            card(TRUMP, Value::_7),
            card(TRUMP, Value::_8),
            card(Color::Diamond, Value::King),
        ]);
        let turn = turn_with(&[
            (Position::North, card(Color::Heart, Value::As)),
            (Position::East, card(TRUMP, Value::Jack)),
        ]);
        assert_eq!(
            choices(&hand, Position::South, &turn),
            sorted(&[card(TRUMP, Value::_7), card(TRUMP, Value::_8)])
        );
    }

    #[test]
    fn cutting_a_plain_color_may_be_done_with_any_trump() {
        // nobody cut yet: the ace of hearts is master but it is not a trump, so
        // every trump takes the fold and none of them is compulsory
        let hand = hand_of(&[
            card(TRUMP, Value::_7),
            card(TRUMP, Value::Jack),
            card(Color::Diamond, Value::King),
        ]);
        let turn = turn_with(&[(Position::North, card(Color::Heart, Value::As))]);
        assert_eq!(
            choices(&hand, Position::East, &turn),
            sorted(&[card(TRUMP, Value::_7), card(TRUMP, Value::Jack)])
        );
    }

    #[test]
    fn without_the_asked_color_nor_trump_anything_goes() {
        let hand = hand_of(&[
            card(Color::Diamond, Value::King),
            card(Color::Club, Value::_7),
        ]);
        let mut turn = Turn::new(7, Order::default());
        assert!(turn.put(TRUMP, Position::North, &card(Color::Heart, Value::As)));
        assert_eq!(
            choices(&hand, Position::East, &turn),
            sorted(&[
                card(Color::Diamond, Value::King),
                card(Color::Club, Value::_7)
            ])
        );
    }
}

#[cfg(test)]
mod bidding_tests {
    use super::biddable_contracts;
    use crate::card::{Card, Color, Value};
    use crate::contract::Contract;
    use strum::IntoEnumIterator;

    #[test]
    fn second_turn_excludes_the_returned_card_color() {
        for color in Color::iter() {
            for value in Value::iter() {
                let Some(card_returned) = Card::new(color, value) else {
                    panic!("cannot build card");
                };
                let biddable: Vec<Contract> = biddable_contracts(card_returned).collect();

                assert!(
                    !biddable.iter().any(|c| c.color() == Some(color)),
                    "{color} was biddable although it is the returned card color"
                );
                // the three other colors, plus Pass
                assert_eq!(biddable.len(), 4);
                assert!(biddable.contains(&Contract::Pass));
            }
        }
    }
}
