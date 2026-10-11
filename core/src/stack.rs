use crate::card::{Card, Color, Value};
use crate::constants::{MAX_CARDS, MIN_CUT};
use crate::errors::{BeloteErrorKind, ErrOnSome};
use derive_more::Index;
use rand::seq::SliceRandom;
use rand::{Rng, RngExt};
use std::fmt;
use tinyvec::ArrayVec;

#[derive(Default, Clone, Debug, Index, Copy)]
pub struct Stack(ArrayVec<[Option<Card>; MAX_CARDS]>);

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for card in self.0.into_iter().flatten() {
            writeln!(f, "{card}")?;
        }
        Ok(())
    }
}

impl Stack {
    pub fn new() -> Self {
        let mut cards = ArrayVec::new();
        cards.push(Card::new(Color::Club, Value::As));
        cards.push(Card::new(Color::Club, Value::King));
        cards.push(Card::new(Color::Club, Value::Queen));
        cards.push(Card::new(Color::Club, Value::Jack));
        cards.push(Card::new(Color::Club, Value::_10));
        cards.push(Card::new(Color::Club, Value::_9));
        cards.push(Card::new(Color::Club, Value::_8));
        cards.push(Card::new(Color::Club, Value::_7));
        cards.push(Card::new(Color::Heart, Value::As));
        cards.push(Card::new(Color::Heart, Value::King));
        cards.push(Card::new(Color::Heart, Value::Queen));
        cards.push(Card::new(Color::Heart, Value::Jack));
        cards.push(Card::new(Color::Heart, Value::_10));
        cards.push(Card::new(Color::Heart, Value::_9));
        cards.push(Card::new(Color::Heart, Value::_8));
        cards.push(Card::new(Color::Heart, Value::_7));
        cards.push(Card::new(Color::Spade, Value::As));
        cards.push(Card::new(Color::Spade, Value::King));
        cards.push(Card::new(Color::Spade, Value::Queen));
        cards.push(Card::new(Color::Spade, Value::Jack));
        cards.push(Card::new(Color::Spade, Value::_10));
        cards.push(Card::new(Color::Spade, Value::_9));
        cards.push(Card::new(Color::Spade, Value::_8));
        cards.push(Card::new(Color::Spade, Value::_7));
        cards.push(Card::new(Color::Diamond, Value::As));
        cards.push(Card::new(Color::Diamond, Value::King));
        cards.push(Card::new(Color::Diamond, Value::Queen));
        cards.push(Card::new(Color::Diamond, Value::Jack));
        cards.push(Card::new(Color::Diamond, Value::_10));
        cards.push(Card::new(Color::Diamond, Value::_9));
        cards.push(Card::new(Color::Diamond, Value::_8));
        cards.push(Card::new(Color::Diamond, Value::_7));
        Self(cards)
    }
    /// The 32 cards shuffled with the caller's generator.
    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let mut new_stack = Self::new();
        new_stack.0.shuffle(rng);
        new_stack
    }
    /// Lift some cards off the top and put them under, where the caller's generator says.
    ///
    /// RULE: a cut leaves at least [`MIN_CUT`] cards on each side.
    pub fn cut<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), BeloteErrorKind> {
        let len = self.0.len();
        if len < 2 * MIN_CUT {
            return Err(BeloteErrorKind::InvalidCase(format!(
                "a stack of {len} cards is too small to be cut"
            )));
        }
        self.cut_at(rng.random_range(MIN_CUT..=len - MIN_CUT))
    }
    /// Lift the top `depth` cards and put them under the rest.
    pub fn cut_at(&mut self, depth: usize) -> Result<(), BeloteErrorKind> {
        let len = self.0.len();
        if depth < MIN_CUT || depth + MIN_CUT > len {
            return Err(BeloteErrorKind::InvalidCase(format!(
                "a cut at {depth} cards does not leave {MIN_CUT} on both sides"
            )));
        }
        self.0.rotate_left(depth);
        Ok(())
    }
    pub fn append_card(&mut self, card: Card) -> Result<(), BeloteErrorKind> {
        self.0.try_push(Some(card)).err_on_some(|| {
            Err(BeloteErrorKind::InvalidCase(
                "Cannot append card to stack".to_string(),
            ))
        })
    }
    pub fn give_card(&mut self) -> Result<Card, BeloteErrorKind> {
        let Some(option_card) = self.0.pop() else {
            return Err(BeloteErrorKind::InvalidCase(
                "cannot give a card".to_string(),
            ));
        };
        let Some(card) = option_card else {
            return Err(BeloteErrorKind::InvalidCase(
                "cannot give a card".to_string(),
            ));
        };
        Ok(card)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;
    use crate::constants::{MAX_CARDS, MIN_CUT};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn a_cut_puts_the_top_under_without_losing_a_card() {
        let full = Stack::new();
        let mut cut = Stack::new();
        cut.cut_at(10).expect("ten cards is a fair cut");
        assert_eq!(cut.len(), MAX_CARDS);
        assert_eq!(cut[0], full[10]);
        assert_eq!(cut[MAX_CARDS - 10], full[0]);
    }

    #[test]
    fn a_cut_must_leave_three_cards_on_each_side() {
        let mut stack = Stack::new();
        assert!(stack.cut_at(MIN_CUT - 1).is_err());
        assert!(stack.cut_at(MAX_CARDS - MIN_CUT + 1).is_err());
        assert!(stack.cut_at(MIN_CUT).is_ok());
        assert!(stack.cut_at(MAX_CARDS - MIN_CUT).is_ok());
    }

    #[test]
    fn a_random_cut_always_moves_at_least_three_cards() {
        let mut rng = StdRng::seed_from_u64(1);
        let full = Stack::new();
        for _ in 0..200 {
            let mut stack = Stack::new();
            stack.cut(&mut rng).expect("a full stack can be cut");
            let depth = (0..MAX_CARDS)
                .find(|&depth| stack[0] == full[depth])
                .expect("the first card comes from the stack");
            assert!((MIN_CUT..=MAX_CARDS - MIN_CUT).contains(&depth), "{depth}");
        }
    }
}
