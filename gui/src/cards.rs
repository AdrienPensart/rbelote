//! Painting cards: the embedded images when they are loaded, shapes and text otherwise.

use crate::textures::Textures;
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, StrokeKind,
    Vec2,
};
use rbelote_core::card::{Card, Color, Value};

// The images come from a tarot deck, whose cards are tall: 250 by 481 pixels.
pub const HAND_CARD: Vec2 = vec2(82.0, 144.0);
pub const TRICK_CARD: Vec2 = vec2(70.0, 124.0);
pub const SMALL_CARD: Vec2 = vec2(53.0, 94.0);

const RADIUS: CornerRadius = CornerRadius::same(6);
const FACE: Color32 = Color32::from_rgb(250, 247, 238);
const EDGE: Color32 = Color32::from_rgb(70, 70, 70);
const GOLD: Color32 = Color32::from_rgb(214, 170, 40);

/// The ink a suit is printed with on a card face.
pub const fn rgb(color: Color) -> Color32 {
    match color {
        Color::Heart | Color::Diamond => Color32::from_rgb(200, 30, 50),
        Color::Spade | Color::Club => Color32::from_rgb(30, 30, 30),
    }
}

/// The same suit, painted on the dark felt where black would not show.
pub const fn felt_rgb(color: Color) -> Color32 {
    match color {
        Color::Heart | Color::Diamond => Color32::from_rgb(255, 110, 110),
        Color::Spade | Color::Club => Color32::from_rgb(235, 235, 225),
    }
}

/// What is printed in the corner of a card: the ace is an A, not an 11.
pub fn symbol(value: Value) -> String {
    match value {
        Value::As => "A".to_string(),
        Value::_7
        | Value::_8
        | Value::_9
        | Value::_10
        | Value::Jack
        | Value::Queen
        | Value::King => value.to_string(),
    }
}

/// A card named the way a player would say it: the suit symbol then the value.
pub fn name(card: Card) -> String {
    format!("{}{}", card.color(), symbol(card.value()))
}

/// A suit symbol of height `size`, drawn from circles and polygons.
pub fn paint_suit(painter: &Painter, center: Pos2, size: f32, color: Color, ink: Color32) {
    let at = |x: f32, y: f32| center + vec2(x * size, y * size);
    let polygon = |points: Vec<Pos2>| {
        painter.add(egui::Shape::convex_polygon(points, ink, Stroke::NONE));
    };
    let stem = || polygon(vec![at(-0.14, 0.5), at(0.14, 0.5), at(0.0, 0.1)]);
    match color {
        Color::Diamond => polygon(vec![
            at(0.0, -0.5),
            at(0.38, 0.0),
            at(0.0, 0.5),
            at(-0.38, 0.0),
        ]),
        Color::Heart => {
            painter.circle_filled(at(-0.22, -0.18), 0.27 * size, ink);
            painter.circle_filled(at(0.22, -0.18), 0.27 * size, ink);
            polygon(vec![at(-0.47, -0.08), at(0.47, -0.08), at(0.0, 0.5)]);
        }
        Color::Spade => {
            polygon(vec![at(0.0, -0.5), at(0.47, 0.1), at(-0.47, 0.1)]);
            painter.circle_filled(at(-0.23, 0.1), 0.25 * size, ink);
            painter.circle_filled(at(0.23, 0.1), 0.25 * size, ink);
            stem();
        }
        Color::Club => {
            painter.circle_filled(at(0.0, -0.24), 0.23 * size, ink);
            painter.circle_filled(at(-0.24, 0.08), 0.23 * size, ink);
            painter.circle_filled(at(0.24, 0.08), 0.23 * size, ink);
            stem();
        }
    }
}

/// A card, face up: its image when there is one, else a drawing.
pub fn paint_card(painter: &Painter, rect: Rect, card: Card, textures: &Textures) {
    if let Some(texture) = textures.face(card) {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        // The images carry their own border and rounded corners.
        painter.image(texture.id(), rect, uv, Color32::WHITE);
        return;
    }
    painter.rect(
        rect,
        RADIUS,
        FACE,
        Stroke::new(1.5, EDGE),
        StrokeKind::Inside,
    );
    let color = card.color();
    let ink = rgb(color);
    let corner = rect.height() * 0.19;
    let font = FontId::proportional(corner);
    let symbol = symbol(card.value());
    // Index in both corners, the lower one upside down would need a rotated galley;
    // mirroring the position is enough to read the card from either side.
    painter.text(
        rect.min + vec2(5.0, 3.0),
        Align2::LEFT_TOP,
        &symbol,
        font.clone(),
        ink,
    );
    let offset = vec2(corner.mul_add(0.42, 5.0), corner.mul_add(1.45, 3.0));
    paint_suit(painter, rect.min + offset, corner * 0.6, color, ink);
    painter.text(
        rect.max - vec2(5.0, 3.0),
        Align2::RIGHT_BOTTOM,
        &symbol,
        font,
        ink,
    );
    paint_suit(painter, rect.max - offset, corner * 0.6, color, ink);

    let center = rect.center();
    match card.value() {
        Value::Jack | Value::Queen | Value::King => {
            let big = FontId::proportional(rect.height() * 0.4);
            painter.text(
                center - vec2(0.0, rect.height() * 0.04),
                Align2::CENTER_CENTER,
                symbol,
                big,
                ink,
            );
            paint_suit(
                painter,
                center + vec2(0.0, rect.height() * 0.24),
                rect.height() * 0.16,
                color,
                ink,
            );
        }
        Value::As | Value::_7 | Value::_8 | Value::_9 | Value::_10 => {
            paint_suit(painter, center, rect.height() * 0.34, color, ink);
        }
    }
}

/// Grey out a card that cannot be chosen right now.
pub fn paint_dim(painter: &Painter, rect: Rect) {
    painter.rect_filled(rect, RADIUS, Color32::from_black_alpha(130));
}

/// A glowing border around a card that is being pointed at.
pub fn paint_highlight(painter: &Painter, rect: Rect) {
    painter.rect_stroke(
        rect.expand(2.0),
        CornerRadius::same(8),
        Stroke::new(3.0, GOLD),
        StrokeKind::Outside,
    );
}
