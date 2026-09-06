//! How a finished deal is turned into points on the score sheet.

use crate::constants::{ALL_CARD_POINTS, CAPOT_POINTS, MAX_CARDS_BY_PLAYER};

/// Everything a deal produced, before it is written to the score sheet.
#[derive(Debug)]
pub struct DealPoints {
    /// card points won by the taker's team, last trick included
    pub attack: u64,
    /// card points won by the defense, last trick included
    pub defense: u64,
    /// tricks won by the taker's team, out of `MAX_CARDS_BY_PLAYER`
    pub attack_tricks: u64,
    /// belote/rebelote, credited to whichever side holds it
    pub belote_attack: u64,
    pub belote_defense: u64,
    /// points held over from previous litiges
    pub litige: u64,
}

/// What each team scores for the deal, and what is held over to the next one.
#[derive(Debug, Eq, PartialEq)]
pub struct DealScore {
    pub attack: u64,
    pub defense: u64,
    pub litige: u64,
}

impl DealPoints {
    /// Belote/rebelote is scored by its holder whatever the outcome, so it is
    /// part of each side's total rather than a bonus added afterwards. Which
    /// side holds it therefore moves the threshold: the taker needs more than
    /// half of the 182 points in play, not more than half of the usual 162.
    pub const fn score(&self) -> DealScore {
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
