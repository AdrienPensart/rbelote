//! The card faces, images embedded in the binary and uploaded to the GPU once.

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use rbelote_core::card::{Card, Color, Value};

/// Every face, in slot order: each color ace, 7, 8, 9, 10, jack, queen, king.
const FACES: [&[u8]; 32] = [
    include_bytes!("../assets/cards/heart_01.png"),
    include_bytes!("../assets/cards/heart_07.png"),
    include_bytes!("../assets/cards/heart_08.png"),
    include_bytes!("../assets/cards/heart_09.png"),
    include_bytes!("../assets/cards/heart_10.png"),
    include_bytes!("../assets/cards/heart_11.png"),
    include_bytes!("../assets/cards/heart_13.png"),
    include_bytes!("../assets/cards/heart_14.png"),
    include_bytes!("../assets/cards/spade_01.png"),
    include_bytes!("../assets/cards/spade_07.png"),
    include_bytes!("../assets/cards/spade_08.png"),
    include_bytes!("../assets/cards/spade_09.png"),
    include_bytes!("../assets/cards/spade_10.png"),
    include_bytes!("../assets/cards/spade_11.png"),
    include_bytes!("../assets/cards/spade_13.png"),
    include_bytes!("../assets/cards/spade_14.png"),
    include_bytes!("../assets/cards/diamond_01.png"),
    include_bytes!("../assets/cards/diamond_07.png"),
    include_bytes!("../assets/cards/diamond_08.png"),
    include_bytes!("../assets/cards/diamond_09.png"),
    include_bytes!("../assets/cards/diamond_10.png"),
    include_bytes!("../assets/cards/diamond_11.png"),
    include_bytes!("../assets/cards/diamond_13.png"),
    include_bytes!("../assets/cards/diamond_14.png"),
    include_bytes!("../assets/cards/club_01.png"),
    include_bytes!("../assets/cards/club_07.png"),
    include_bytes!("../assets/cards/club_08.png"),
    include_bytes!("../assets/cards/club_09.png"),
    include_bytes!("../assets/cards/club_10.png"),
    include_bytes!("../assets/cards/club_11.png"),
    include_bytes!("../assets/cards/club_13.png"),
    include_bytes!("../assets/cards/club_14.png"),
];

/// The slot of a card in [`FACES`].
const fn slot(card: Card) -> usize {
    color_slot(card.color()) * 8 + value_slot(card.value())
}

const fn color_slot(color: Color) -> usize {
    match color {
        Color::Heart => 0,
        Color::Spade => 1,
        Color::Diamond => 2,
        Color::Club => 3,
    }
}

const fn value_slot(value: Value) -> usize {
    match value {
        Value::As => 0,
        Value::_7 => 1,
        Value::_8 => 2,
        Value::_9 => 3,
        Value::_10 => 4,
        Value::Jack => 5,
        Value::Queen => 6,
        Value::King => 7,
    }
}

fn decode(ctx: &Context, name: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    Some(ctx.load_texture(name, color, TextureOptions::LINEAR))
}

pub struct Textures {
    faces: Vec<Option<TextureHandle>>,
}

impl Textures {
    /// Decode every face. One that fails to decode is simply drawn by hand.
    pub fn load(ctx: &Context) -> Self {
        let faces = FACES
            .iter()
            .enumerate()
            .map(|(index, bytes)| decode(ctx, &format!("card{index}"), bytes))
            .collect();
        Self { faces }
    }

    pub fn face(&self, card: Card) -> Option<&TextureHandle> {
        self.faces.get(slot(card)).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::{slot, FACES};
    use rbelote_core::card::{Card, Color, Value};
    use std::collections::BTreeSet;
    use strum::IntoEnumIterator;

    /// The 32 cards of the deck.
    fn deck() -> Vec<Card> {
        Color::iter()
            .flat_map(|color| Value::iter().filter_map(move |value| Card::new(color, value)))
            .collect()
    }

    #[test]
    fn every_card_has_its_own_face() {
        let slots: BTreeSet<usize> = deck().into_iter().map(slot).collect();
        assert_eq!(slots.len(), FACES.len());
        assert_eq!(slots.iter().next_back(), Some(&(FACES.len() - 1)));
    }

    #[test]
    fn every_face_decodes() {
        let textures = super::Textures::load(&egui::Context::default());
        let missing: Vec<String> = deck()
            .into_iter()
            .filter(|card| textures.face(*card).is_none())
            .map(|card| card.to_string())
            .collect();
        assert!(missing.is_empty(), "undecodable faces: {missing:?}");
    }

    #[test]
    fn slots_follow_the_file_names() {
        let card = |color, value| Card::new(color, value).map_or(usize::MAX, slot);
        assert_eq!(card(Color::Heart, Value::As), 0);
        assert_eq!(card(Color::Heart, Value::King), 7);
        assert_eq!(card(Color::Spade, Value::_7), 9);
        assert_eq!(card(Color::Club, Value::King), 31);
    }
}
