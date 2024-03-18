use crate::bidding::Bidding;
use crate::errors::BeloteErrorKind;
use crate::game::Game;
use crate::hands::Hands;
use crate::initial::Initial;
use derive_more::{Constructor, Deref, DerefMut};
use tracing::info;

#[derive(Constructor, Deref, DerefMut)]
pub struct Distribution {
    hands: Hands,
    #[deref]
    #[deref_mut]
    initial: Initial,
}

impl Game<Distribution> {
    pub fn bidding(mut self) -> Result<Game<Bidding>, BeloteErrorKind> {
        let card_returned = self.stack_mut().give_card()?;
        info!("Card returned: {card_returned}");
        Ok(Game::new(
            self.players(),
            self.points(),
            Bidding::new(card_returned, self.hands, self.into().initial),
        ))
    }
}
