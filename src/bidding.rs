use crate::card::{Card, Color};
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
use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::str::FromStr;
use strum::IntoEnumIterator;
use tracing::info;

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
        let mut rng = thread_rng();
        let card_returned = self.card_returned;
        let mut trump_color = card_returned.color();
        let mut taker: Option<Position> = None;
        info!("First bidding turn");
        for position in order {
            let take = if players[position].random() {
                rng.gen_bool(players.randomization())
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
                    };
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
                    if rng.gen_bool(players.randomization()) {
                        Contract::iter()
                            .choose(&mut rng)
                            .and_then(|contract| contract.color())
                    } else {
                        None
                    }
                } else {
                    'choose_contract: loop {
                        let card_returned_color = card_returned.color();
                        let contracts: Vec<String> = Contract::iter()
                            .filter(|c| c.to_string() != card_returned_color.to_string())
                            .map(|c| c.to_string())
                            .collect();
                        info!("Nobody took: {card_returned}, please choose a color for trumps");
                        let answer =
                            Select::new("Which color do you choose ? (ESC to cancel)", contracts)
                                .prompt_skippable();
                        match answer {
                            Ok(Some(maybe_chosen_color)) => {
                                if let Ok(chosen_color) = Color::from_str(&maybe_chosen_color) {
                                    break 'choose_contract Some(chosen_color);
                                }
                                continue 'second_turn;
                            }
                            Ok(None) => {
                                info!("Interrupted.");
                                return Ok(PlayOrNext::Interrupted);
                            }
                            Err(_) => {
                                info!("Error with questionnaire, try again.");
                            }
                        };
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
