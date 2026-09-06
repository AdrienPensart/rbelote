use crate::card::Card;
use crate::card::Color;
use crate::constants::{
    ALL_CARD_POINTS, BELOTE_POINTS, CAPOT_POINTS, LAST_TRICK_POINTS, MAX_CARDS_BY_PLAYER,
    MAX_PLAYERS,
};
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hand::Hand;
use crate::hands::Hands;
use crate::initial::Initial;
use crate::position::Position;
use crate::team::Team;
use crate::turn::Turn;
use derive_more::{Constructor, Deref, DerefMut};
use inquire::Select;
use rand::{rng, seq::IteratorRandom};
use tracing::{info, warn};

#[derive(Constructor, Deref, DerefMut)]
pub struct Playing {
    taker: Position,
    hands: Hands,
    trump_color: Color,
    #[deref]
    #[deref_mut]
    initial: Initial,
}

impl Playing {
    pub const fn into(self) -> Initial {
        self.initial
    }
    pub fn hand(&self, position: Position) -> &Hand {
        &self.hands[position]
    }
    pub fn hand_mut(&mut self, position: Position) -> &mut Hand {
        &mut self.hands[position]
    }
    pub const fn add_litige(&mut self, litige: u64) {
        self.initial.add_litige(litige);
    }
    pub const fn reset_litige(&mut self) -> u64 {
        self.initial.reset_litige()
    }
    pub const fn taker(&self) -> Position {
        self.taker
    }
    pub const fn hands(&self) -> Hands {
        self.hands
    }
    pub const fn trump_color(&self) -> Color {
        self.trump_color
    }
}

/// Everything a deal produced, before it is written to the score sheet.
#[derive(Debug)]
struct DealPoints {
    /// card points won by the taker's team, last trick included
    attack: u64,
    /// card points won by the defense, last trick included
    defense: u64,
    /// tricks won by the taker's team, out of `MAX_CARDS_BY_PLAYER`
    attack_tricks: u64,
    /// belote/rebelote, credited to whichever side holds it
    belote_attack: u64,
    belote_defense: u64,
    /// points held over from previous litiges
    litige: u64,
}

/// What each team scores for the deal, and what is held over to the next one.
#[derive(Debug, Eq, PartialEq)]
struct DealScore {
    attack: u64,
    defense: u64,
    litige: u64,
}

impl DealPoints {
    /// Belote/rebelote is scored by its holder whatever the outcome, so it is
    /// part of each side's total rather than a bonus added afterwards. Which
    /// side holds it therefore moves the threshold: the taker needs more than
    /// half of the 182 points in play, not more than half of the usual 162.
    const fn score(&self) -> DealScore {
        let attack_total = self.attack + self.belote_attack;
        let defense_total = self.defense + self.belote_defense;
        let defense_tricks = MAX_CARDS_BY_PLAYER as u64 - self.attack_tricks;

        if self.attack_tricks == 0 {
            // capot: the defense took every trick
            DealScore {
                attack: self.belote_attack,
                defense: CAPOT_POINTS + self.belote_defense + self.litige,
                litige: 0,
            }
        } else if defense_tricks == 0 {
            // capot: the taker took every trick
            DealScore {
                attack: CAPOT_POINTS + self.belote_attack + self.litige,
                defense: self.belote_defense,
                litige: 0,
            }
        } else if attack_total > defense_total {
            // reussite: contract fulfilled
            DealScore {
                attack: attack_total + self.litige,
                defense: defense_total,
                litige: 0,
            }
        } else if attack_total == defense_total {
            // litige: the taker's card points are held over to the next deal
            DealScore {
                attack: self.belote_attack,
                defense: defense_total,
                litige: self.litige + self.attack,
            }
        } else {
            // dedans: the defense scores every card point in play
            DealScore {
                attack: self.belote_attack,
                defense: ALL_CARD_POINTS + self.belote_defense + self.litige,
                litige: 0,
            }
        }
    }
}

pub enum NextGameOrInterrupt {
    NextGame(Game<Initial>),
    Interrupted,
}

impl Game<Playing> {
    pub fn play(mut self) -> Result<NextGameOrInterrupt, BeloteErrorKind> {
        let mut belote_rebelote: Option<Team> = None;
        for position in self.order() {
            if self.hand(position).belote_rebelote(self.trump_color()) {
                belote_rebelote = Some(position.team());
            }
        }
        let mut current_position = self.order()[0];
        let mut attack_points: u64 = 0;
        let mut defense_points: u64 = 0;
        let mut attack_tricks: u64 = 0;

        for turn_number in 0..MAX_CARDS_BY_PLAYER {
            let mut turn = Turn::new(turn_number as u64 + 1, self.order());
            loop {
                info!("{current_position} to play for {turn}");
                info!(
                    "Hand of {current_position} before playing : {}",
                    self.hand(current_position)
                );

                let choices = &self.players()[current_position].choices(
                    self.hand(current_position),
                    &current_position,
                    &turn,
                    self.trump_color(),
                )?;
                if choices.is_empty() {
                    return Err(BeloteErrorKind::InvalidCase(
                        "no choices available".to_string(),
                    ));
                }

                turn.called_color().map_or_else(
                    || info!("{current_position} is first to play, you can choose a color"),
                    |called_color| info!("{current_position} must play color {called_color}"),
                );

                let chosen_card = if self.players()[current_position].random() {
                    let mut rng = rng();
                    let Some(random_card): Option<Card> = choices.iter().choose(&mut rng).copied()
                    else {
                        return Err(BeloteErrorKind::InvalidCase(
                            "cannot find a random card choice".to_string(),
                        ));
                    };
                    random_card
                } else {
                    loop {
                        let page_size = choices.len();
                        let chosen_card = Select::new(
                            "Which card do you choose ? (ESC to cancel)",
                            choices.clone(),
                        )
                        .with_page_size(page_size)
                        .prompt_skippable();
                        match chosen_card {
                            Ok(Some(chosen_card)) => {
                                break chosen_card;
                            }
                            Ok(None) => {
                                info!("Interrupted.");
                                return Ok(NextGameOrInterrupt::Interrupted);
                            }
                            Err(_) => {
                                info!("Error with questionnaire, try again.");
                            }
                        }
                    }
                };

                let Some(_) = self.hand_mut(current_position).give(&chosen_card) else {
                    return Err(BeloteErrorKind::InvalidCase(
                        "cannot give chosen card".to_string(),
                    ));
                };
                info!(
                    "Hand of {current_position} after playing : {}",
                    self.hand(current_position)
                );
                let _ = turn.put(self.trump_color(), current_position, &chosen_card);
                if turn.finished() {
                    break;
                }
                current_position = current_position.next();
            }

            current_position = turn.master_position();
            info!("Fold master is player {}", turn.master_position());
            let master_team = turn.master_team();
            if self.taker().team() == master_team {
                attack_tricks += 1;
            }
            let Some(cards) = turn.take() else {
                return Err(BeloteErrorKind::InvalidCase(
                    "Cannot take turn cards".to_string(),
                ));
            };
            for card in cards {
                let points = card.points(self.trump_color());
                warn!("{card} : {points} points");
                if self.taker().team() == master_team {
                    attack_points += points;
                } else {
                    defense_points += points;
                }
                self.stack_mut().append_card(card)?;
            }

            if self.stack().len() != (turn_number + 1) * MAX_PLAYERS {
                return Err(BeloteErrorKind::InvalidCase(format!(
                    "bad deck length {} it should be {}",
                    self.stack().len(),
                    (turn_number + 1) * MAX_PLAYERS
                )));
            }
            info!("New attack points = {attack_points}");
            info!("New defense points = {defense_points}");
        }

        if current_position.team() == self.taker().team() {
            attack_points += LAST_TRICK_POINTS;
        } else {
            defense_points += LAST_TRICK_POINTS;
        }

        if attack_points + defense_points != ALL_CARD_POINTS {
            return Err(BeloteErrorKind::InvalidCase(format!(
                "bad points total : {attack_points} + {defense_points} \
                 should be {ALL_CARD_POINTS}"
            )));
        }

        let taker_team = self.taker().team();
        let (belote_attack, belote_defense) = belote_rebelote.map_or((0, 0), |team| {
            if team == taker_team {
                (BELOTE_POINTS, 0)
            } else {
                (0, BELOTE_POINTS)
            }
        });

        let deal = DealPoints {
            attack: attack_points,
            defense: defense_points,
            attack_tricks,
            belote_attack,
            belote_defense,
            litige: self.reset_litige(),
        };
        let score = deal.score();
        info!("{deal:?} => {score:?}");

        self.add_litige(score.litige);
        self.add_points(taker_team, score.attack);
        self.add_points(taker_team.other(), score.defense);
        let players = self.players();
        let points = self.points();
        let initial = self.into().into().next();
        Ok(NextGameOrInterrupt::NextGame(Game::new(
            players, points, initial,
        )))
    }
}

#[cfg(test)]
mod scoring_tests {
    use super::{DealPoints, DealScore};
    use crate::constants::BELOTE_POINTS;

    /// Card points always split 162 between the two teams.
    fn deal(attack: u64, attack_tricks: u64) -> DealPoints {
        DealPoints {
            attack,
            defense: 162 - attack,
            attack_tricks,
            belote_attack: 0,
            belote_defense: 0,
            litige: 0,
        }
    }

    fn score(attack: u64, defense: u64, litige: u64) -> DealScore {
        DealScore {
            attack,
            defense,
            litige,
        }
    }

    #[test]
    fn without_belote_the_taker_needs_82() {
        assert_eq!(deal(82, 4).score(), score(82, 80, 0));
        assert_eq!(deal(81, 4).score(), score(0, 81, 81)); // litige
        assert_eq!(deal(80, 4).score(), score(0, 162, 0)); // dedans
    }

    #[test]
    fn dedans_awards_the_162_points_actually_in_play() {
        // used to hand the defense 182 points out of a 162 point deal
        assert_eq!(deal(0, 1).score().defense, 162);
    }

    #[test]
    fn capot_counts_tricks_not_points() {
        assert_eq!(deal(162, 8).score(), score(252, 0, 0));
        assert_eq!(deal(0, 0).score(), score(0, 252, 0));
        // a taker winning two worthless tricks then losing the last one is
        // "dedans", not capot, even though they scored zero
        assert_eq!(deal(0, 2).score(), score(0, 162, 0));
    }

    #[test]
    fn belote_for_the_taker_lowers_the_threshold_to_72() {
        let with_belote = |attack, tricks| DealPoints {
            belote_attack: BELOTE_POINTS,
            ..deal(attack, tricks)
        };
        // 72 + 20 = 92 beats the defense's 90
        assert_eq!(with_belote(72, 4).score(), score(92, 90, 0));
        // 71 + 20 = 91 ties the defense's 91
        assert_eq!(with_belote(71, 4).score(), score(20, 91, 71));
        assert_eq!(with_belote(70, 4).score(), score(20, 162, 0)); // dedans, belote still counts
        // the defense is no longer credited the taker's own belote
        assert_eq!(with_belote(100, 4).score(), score(120, 62, 0));
        // a capot with belote is a capot, not a "reussite"
        assert_eq!(with_belote(162, 8).score(), score(272, 0, 0));
    }

    #[test]
    fn belote_for_the_defense_raises_the_threshold_to_92() {
        let with_belote = |attack, tricks| DealPoints {
            belote_defense: BELOTE_POINTS,
            ..deal(attack, tricks)
        };
        assert_eq!(with_belote(92, 4).score(), score(92, 90, 0));
        assert_eq!(with_belote(91, 4).score(), score(0, 91, 91)); // litige
        assert_eq!(with_belote(90, 4).score(), score(0, 182, 0)); // dedans
    }

    #[test]
    fn litige_accumulates_and_is_paid_to_the_next_winner() {
        assert_eq!(deal(81, 4).score().litige, 81);
        let carried = DealPoints {
            litige: 81,
            ..deal(81, 4)
        };
        assert_eq!(carried.score().litige, 162);
        // and it goes to whoever wins the following deal
        let next = DealPoints {
            litige: 81,
            ..deal(90, 4)
        };
        assert_eq!(next.score(), score(171, 72, 0));
    }
}
