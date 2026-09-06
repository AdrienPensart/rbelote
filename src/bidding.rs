use crate::card::Card;
use crate::contract::Contract;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hand::Hand;
use crate::hands::Hands;
use crate::initial::Initial;
use crate::playing::Playing;
use crate::position::Position;
use derive_more::{Constructor, Deref, DerefMut};
use inquire::{Confirm, Select};
use rand::{rng, seq::IteratorRandom, RngExt};
use strum::IntoEnumIterator;
use tracing::info;

/// Second bidding turn: any color may be named as trump except the one of the
/// returned card. `Contract::Pass` is kept, it is how a player declines.
fn biddable_contracts(card_returned: Card) -> impl Iterator<Item = Contract> {
    Contract::iter().filter(move |contract| contract.color() != Some(card_returned.color()))
}

pub enum PlayOrNext {
    NextGame(Game<Initial>),
    PlayGame(Game<Playing>),
    Interrupted,
}

#[derive(Constructor, Deref, DerefMut)]
pub struct Bidding {
    card_returned: Card,
    hands: Hands,
    #[deref]
    #[deref_mut]
    initial: Initial,
}

impl Bidding {
    pub const fn into(self) -> Initial {
        self.initial
    }

    pub fn hand(&self, position: Position) -> Hand {
        self.hands[position]
    }

    pub fn hand_mut(&mut self, position: Position) -> &mut Hand {
        &mut self.hands[position]
    }

    pub fn complete_hand(
        &mut self,
        position: Position,
        count: usize,
    ) -> Result<(), BeloteErrorKind> {
        for _ in 0..count {
            let card = self.stack_mut().give_card()?;
            self.hand_mut(position).take(card)?;
        }
        Ok(())
    }
}

impl Game<Bidding> {
    pub fn playing_game_or_redistribute(mut self) -> Result<PlayOrNext, BeloteErrorKind> {
        let order = self.order();
        let players = self.players();
        let points = self.points();
        let card_returned = self.card_returned;
        let mut trump_color = card_returned.color();
        let mut taker: Option<Position> = None;
        info!("First bidding turn");
        for position in order {
            let take = if players[position].random() {
                rng().random_bool(players.randomization())
            } else {
                'questionnaire: loop {
                    info!(
                        "{position} must decide if he is taking : {}",
                        self.hand(position)
                    );
                    let answer = Confirm::new("Do you take ? (ESC to cancel)")
                        .with_default(false)
                        .prompt_skippable();
                    match answer {
                        Ok(Some(value)) => {
                            break 'questionnaire value;
                        }
                        Ok(None) => {
                            info!("Interrupted.");
                            return Ok(PlayOrNext::Interrupted);
                        }
                        Err(_) => {
                            info!("Error with questionnaire, try again.");
                        }
                    }
                }
            };

            if take {
                taker = Some(position);
                trump_color = card_returned.color();
                break;
            }
            info!("{position} did not take at first glance");
        }
        info!("randomization: {}", players.randomization());
        if taker.is_none() {
            info!("Second bidding turn");
            'second_turn: for position in order {
                let chosen_color = if players[position].random() {
                    if rng().random_bool(players.randomization()) {
                        biddable_contracts(card_returned)
                            .filter_map(|contract| contract.color())
                            .choose(&mut rng())
                    } else {
                        None
                    }
                } else {
                    'choose_contract: loop {
                        let contracts: Vec<Contract> = biddable_contracts(card_returned).collect();
                        info!("Nobody took: {card_returned}, please choose a color for trumps");
                        let answer =
                            Select::new("Which color do you choose ? (ESC to cancel)", contracts)
                                .prompt_skippable();
                        match answer {
                            Ok(Some(chosen_contract)) => match chosen_contract.color() {
                                Some(chosen_color) => break 'choose_contract Some(chosen_color),
                                None => continue 'second_turn,
                            },
                            Ok(None) => {
                                info!("Interrupted.");
                                return Ok(PlayOrNext::Interrupted);
                            }
                            Err(_) => {
                                info!("Error with questionnaire, try again.");
                            }
                        }
                    }
                };
                if let Some(chosen_color) = chosen_color {
                    taker = Some(position);
                    trump_color = chosen_color;
                    break;
                }
            }
        }
        let Some(taker) = taker else {
            self.stack_mut().append_card(card_returned)?;
            for position in order {
                for card in self.hand_mut(position).into_iter().flatten() {
                    self.stack_mut().append_card(card)?;
                }
            }
            // deck.append_stack(self.stack_mut());
            let initial = self.into().into().next();
            return Ok(PlayOrNext::NextGame(Game::new(players, points, initial)));
        };

        info!("{taker} for color {trump_color}, we give him {card_returned}");
        let mut bidding = self.into();
        bidding.hand_mut(taker).take(card_returned)?;

        for position in order {
            if position == taker {
                info!("Giving {position} 2 more cards because taker");
                bidding.complete_hand(position, 2)?;
            } else {
                info!("Giving {position} 3 more cards because others");
                bidding.complete_hand(position, 3)?;
            }
        }

        Ok(PlayOrNext::PlayGame(Game::new(
            players,
            points,
            Playing::new(taker, bidding.hands, trump_color, bidding.into()),
        )))
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
