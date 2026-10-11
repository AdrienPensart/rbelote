//! Turning what happens at the table into sentences for the table talk panel,
//! and a finished deal into a score sheet.

use crate::cards;
use crate::game::Summary;
use rbelote_core::card::Color;
use rbelote_core::constants::MAX_CARDS_BY_PLAYER;
use rbelote_core::contract::Contract;
use rbelote_core::game::{Answer, DealOutcome, Request};
use rbelote_core::points::Points;
use rbelote_core::position::Position;
use rbelote_core::team::Team;

/// A suit, the way a player names it.
pub const fn color_name(color: Color) -> &'static str {
    match color {
        Color::Heart => "hearts",
        Color::Spade => "spades",
        Color::Diamond => "diamonds",
        Color::Club => "clubs",
    }
}

/// A suit symbol followed by its name.
pub fn trump_label(color: Color) -> String {
    format!("{color} {}", color_name(color))
}

/// What a contract reads on a button or in a sentence.
pub fn contract_label(contract: Contract) -> String {
    contract
        .color()
        .map_or_else(|| "Pass".to_string(), trump_label)
}

/// One sentence for an answer, the way a kibitzer would say it.
pub fn answered(name: &str, request: &Request, answer: Answer) -> String {
    match (request, answer) {
        (Request::Take { card_returned, .. }, Answer::Take(true)) => {
            format!(
                "{name} takes the {} and plays {}",
                cards::name(*card_returned),
                trump_label(card_returned.color())
            )
        }
        (Request::Take { .. }, Answer::Take(false)) => format!("{name} passes"),
        (Request::Trump { .. }, Answer::Trump(contract)) => contract.color().map_or_else(
            || format!("{name} passes again"),
            |color| format!("{name} names {} as trump", trump_label(color)),
        ),
        (Request::Play { .. }, Answer::Play(card)) => {
            format!("{name} plays {}", cards::name(card))
        }
        // An answer the engine will refuse anyway: say what was tried.
        (Request::Take { .. } | Request::Trump { .. } | Request::Play { .. }, _) => {
            format!("{name} answers {answer:?}")
        }
    }
}

pub fn trick_won(name: &str, cards: usize) -> String {
    format!("{name} wins the trick ({cards} cards)")
}

pub fn redeal() -> String {
    "Nobody took, the cards are dealt again".to_string()
}

pub fn deal_started(deals_left: u64, dealer: &str, cutter: Option<&str>) -> String {
    let cut = cutter.map_or_else(String::new, |cutter| format!("{cutter} cuts, "));
    format!("— New deal, {deals_left} left, {cut}{dealer} deals —")
}

pub fn game_over() -> String {
    "— Game over —".to_string()
}

pub fn deal_scored(outcome: &DealOutcome, name: &str) -> String {
    format!(
        "{name} took on {}: attack {} points, defense {}, scoring {} to {}",
        trump_label(outcome.trump_color),
        outcome.points.attack,
        outcome.points.defense,
        outcome.score.attack,
        outcome.score.defense
    )
}

/// How the deal went for the attack, in one word or two.
const fn verdict(outcome: &DealOutcome) -> &'static str {
    let tricks = outcome.points.attack_tricks;
    if tricks == MAX_CARDS_BY_PLAYER as u64 {
        "capot for the attack"
    } else if tricks == 0 {
        "capot for the defense"
    } else if outcome.score.attack > outcome.score.defense {
        "contract made"
    } else if outcome.score.litige > outcome.points.litige {
        "litige, the attack's points are held over"
    } else {
        "dedans, the defense takes everything"
    }
}

/// The score sheet of a deal, from its outcome and the totals it led to.
pub fn summary(
    outcome: &DealOutcome,
    totals: Points,
    human: Position,
    name: impl Fn(Position) -> String,
) -> Summary {
    let attack_team = outcome.taker.team();
    let mut rows = vec![
        ("Taker".to_string(), name(outcome.taker)),
        ("Trump".to_string(), trump_label(outcome.trump_color)),
        (
            "Attack".to_string(),
            format!(
                "{} points, {} tricks",
                outcome.points.attack, outcome.points.attack_tricks
            ),
        ),
        (
            "Defense".to_string(),
            format!(
                "{} points, {} tricks",
                outcome.points.defense,
                MAX_CARDS_BY_PLAYER as u64 - outcome.points.attack_tricks
            ),
        ),
    ];
    if let Some(team) = outcome.belote {
        rows.push(("Belote".to_string(), format!("{team} (20 points)")));
    }
    if outcome.points.litige > 0 {
        rows.push((
            "Litige carried in".to_string(),
            format!("{} points", outcome.points.litige),
        ));
    }
    rows.push(("Result".to_string(), verdict(outcome).to_string()));
    rows.push((
        format!("{attack_team} score"),
        format!("{:+}", i128::from(outcome.score.attack)),
    ));
    rows.push((
        format!("{} score", attack_team.other()),
        format!("{:+}", i128::from(outcome.score.defense)),
    ));
    if outcome.score.litige > 0 {
        rows.push((
            "Held over".to_string(),
            format!("{} points", outcome.score.litige),
        ));
    }
    for team in [Team::NorthSouth, Team::WestEast] {
        rows.push((format!("{team} total"), totals[team].to_string()));
    }
    let (mine, theirs) = if human.team() == attack_team {
        (outcome.score.attack, outcome.score.defense)
    } else {
        (outcome.score.defense, outcome.score.attack)
    };
    Summary { rows, mine, theirs }
}
