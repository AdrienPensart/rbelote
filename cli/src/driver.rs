//! Turns the engine's questions into terminal prompts, or into dice rolls for
//! the random players.

use crate::players::Players;
use colored::Colorize;
use inquire::{Confirm, Select};
use rand::seq::IteratorRandom;
use rand::{rng, RngExt};
use rbelote_core::card::{Card, Color};
use rbelote_core::contract::Contract;
use rbelote_core::errors::BeloteErrorKind;
use rbelote_core::game::{Answer, Game, Request};
use tracing::info;

/// A decision, or the player asking to stop.
pub enum Decision {
    Answer(Answer),
    Interrupted,
}

/// Cards and colors are rendered here, not in the engine: a browser has no use
/// for ANSI escapes.
pub fn colorize(color: Color) -> String {
    match color {
        Color::Club => "♣".blue(),
        Color::Diamond => "♦".red(),
        Color::Heart => "♥".red(),
        Color::Spade => "♠".blue(),
    }
    .to_string()
}

fn show_card(card: Card) -> String {
    format!("{} {}", colorize(card.color()), card.value())
}

fn show_contract(contract: Contract) -> String {
    contract
        .color()
        .map_or_else(|| "Pass".to_string(), colorize)
}

pub fn decide(game: &Game, players: Players, request: &Request) -> Result<Decision, BeloteErrorKind> {
    let position = request.position();
    if players[position].random() {
        return random(players, request).map(Decision::Answer);
    }
    ask(game, request)
}

pub fn random(players: Players, request: &Request) -> Result<Answer, BeloteErrorKind> {
    match request {
        Request::Take { .. } => Ok(Answer::Take(rng().random_bool(players.randomization()))),
        Request::Trump { choices, .. } => {
            if !rng().random_bool(players.randomization()) {
                return Ok(Answer::Trump(Contract::Pass));
            }
            // pass is a separate decision, so only real colors are drawn here
            choices
                .iter()
                .filter(|contract| contract.color().is_some())
                .choose(&mut rng())
                .map_or(Ok(Answer::Trump(Contract::Pass)), |contract| {
                    Ok(Answer::Trump(*contract))
                })
        }
        Request::Play { choices, .. } => choices.iter().choose(&mut rng()).map_or_else(
            || {
                Err(BeloteErrorKind::InvalidCase(
                    "cannot find a random card choice".to_string(),
                ))
            },
            |card| Ok(Answer::Play(*card)),
        ),
    }
}

fn ask(game: &Game, request: &Request) -> Result<Decision, BeloteErrorKind> {
    let position = request.position();
    match request {
        Request::Take { card_returned, .. } => {
            info!(
                "{position} must decide if he is taking {} : {}",
                show_card(*card_returned),
                game.hand(position)
            );
            loop {
                match Confirm::new("Do you take ? (ESC to cancel)")
                    .with_default(false)
                    .prompt_skippable()
                {
                    Ok(Some(take)) => return Ok(Decision::Answer(Answer::Take(take))),
                    Ok(None) => return Ok(Decision::Interrupted),
                    Err(_) => info!("Error with questionnaire, try again."),
                }
            }
        }
        Request::Trump {
            card_returned,
            choices,
            ..
        } => {
            info!(
                "Nobody took {}, {position} may choose a color for trumps : {}",
                show_card(*card_returned),
                game.hand(position)
            );
            select("Which color do you choose ? (ESC to cancel)", choices, |c| {
                (show_contract(*c), Answer::Trump(*c))
            })
        }
        Request::Play { choices, .. } => {
            info!("Hand of {position} : {}", game.hand(position));
            game
                .turn()
                .and_then(rbelote_core::turn::Turn::called_color)
                .map_or_else(
                    || info!("{position} is first to play, he can choose a color"),
                    |called| info!("{position} must play color {}", colorize(called)),
                );
            select("Which card do you choose ? (ESC to cancel)", choices, |c| {
                (show_card(*c), Answer::Play(*c))
            })
        }
    }
}

/// Shows `choices` as strings and gives back the value that was picked, without
/// ever parsing the rendered text back into a card.
fn select<T, F>(message: &str, choices: &[T], render: F) -> Result<Decision, BeloteErrorKind>
where
    F: Fn(&T) -> (String, Answer),
{
    let rendered: Vec<(String, Answer)> = choices.iter().map(render).collect();
    let labels: Vec<String> = rendered.iter().map(|(label, _)| label.clone()).collect();
    let page_size = labels.len().max(1);
    loop {
        match Select::new(message, labels.clone())
            .with_page_size(page_size)
            .raw_prompt_skippable()
        {
            Ok(Some(chosen)) => {
                let Some((_, answer)) = rendered.get(chosen.index) else {
                    return Err(BeloteErrorKind::InvalidCase(
                        "impossible choice index".to_string(),
                    ));
                };
                return Ok(Decision::Answer(*answer));
            }
            Ok(None) => return Ok(Decision::Interrupted),
            Err(_) => info!("Error with questionnaire, try again."),
        }
    }
}
