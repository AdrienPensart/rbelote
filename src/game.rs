use crate::card::Color;
use crate::deck::Deck;
use crate::distribution::Distribution;
use crate::errors::BeloteErrorKind;
use crate::hands::Hands;
use crate::helpers::read_index;
use crate::order::Order;
use crate::players::Players;
use crate::points::Points;
use crate::position::Position;
use crate::team::Team;
use crate::traits::PlayingOrder;
use crate::turn::Turn;
use rand::seq::IteratorRandom;
use strum::IntoEnumIterator;

#[derive(Debug, Default)]
pub struct Initial {
    number: u64,
    order: Order,
}

impl Initial {
    pub fn new() -> Self {
        Self {
            number: 0,
            order: Order::default(),
        }
    }
    pub fn next(mut self) -> Self {
        self.order.rotate();
        self.number += 1;
        self
    }
    pub fn order(&self) -> Order {
        self.order
    }
    pub fn number(&self) -> u64 {
        self.number
    }
}

pub struct Playing {
    pub taker: Position,
    pub trump_color: Color,
    pub distribution: Distribution,
}

#[derive(Debug, Default)]
pub struct Game<State> {
    pub deck: Deck,
    pub players: Players,
    pub points: Points,
    pub state: State,
}

impl<State> Game<State> {
    pub fn points(&self) -> &Points {
        &self.points
    }
}

impl PlayingOrder for Game<Initial> {
    fn order(&self) -> Order {
        self.state.order()
    }
}

impl Game<Initial> {
    pub fn number(&self) -> u64 {
        self.state.number
    }

    pub fn new(players: Players) -> Self {
        Self {
            players,
            deck: Deck::build_deck(),
            points: Points::default(),
            state: Initial::new(),
        }
    }

    pub fn distribute(self) -> Game<Distribution> {
        let order = self.order();
        let mut deck = self.deck;
        let mut hands = Hands::default();
        for position in order {
            hands[position].append(deck.give(3));
        }
        for position in order {
            hands[position].append(deck.give(2));
            println!("{position} : {}", hands[position]);
        }
        Game {
            points: self.points,
            players: self.players,
            deck,
            state: Distribution::new(self.state, hands),
        }
    }
}

impl PlayingOrder for Game<Playing> {
    fn order(&self) -> Order {
        self.state.distribution.order()
    }
}

pub enum PlayingResult {
    NextGame(Game<Initial>),
    Interrupted,
}

impl Game<Playing> {
    pub fn play(mut self) -> Result<PlayingResult, BeloteErrorKind> {
        let mut belote_rebelote: Option<Team> = None;
        for position in self.order() {
            if self
                .state
                .distribution
                .hand(position)
                .belote_rebelote(self.state.trump_color)
            {
                belote_rebelote = Some(position.team());
            }
        }
        let mut current_position = self.order()[0];
        let mut attack_points: u64 = 0;
        let mut stack = Deck::default();

        for turn_number in 0..8 {
            let mut turn = Turn::new(turn_number + 1);
            loop {
                println!("{current_position} to play for {turn_number}");
                println!("{turn}");
                println!(
                    "Hand of {current_position} : {}",
                    self.state.distribution.hand(current_position)
                );
                println!("Choices :");

                let choices = &self.players[current_position].choices(
                    self.state.distribution.hand(current_position),
                    &current_position,
                    &turn,
                    self.state.trump_color,
                )?;
                if choices.is_empty() {
                    return Err(BeloteErrorKind::InvalidCase(
                        "no choices available".to_string(),
                    ));
                }
                for i in choices {
                    println!(
                        "\t{0: <2} : {1}",
                        i,
                        self.state.distribution.hand(current_position).0[*i]
                    );
                }

                if let Some(master_card) = turn.master_card() {
                    println!("{current_position} must play color {}", master_card.color())
                } else {
                    println!("{current_position} is first to play, you can choose a color")
                }

                let index = if self.players[current_position].random {
                    let mut rng = rand::thread_rng();
                    let Some(choice): Option<usize> = choices.iter().choose(&mut rng).copied()
                    else {
                        return Err(BeloteErrorKind::InvalidCase(
                            "invalid index for choice".to_string(),
                        ));
                    };
                    choice
                } else {
                    loop {
                        let choice_index = read_index();
                        if choices.contains(&choice_index) {
                            break choice_index;
                        } else {
                            println!("Error, please retry")
                        }
                    }
                };

                let given_card = self
                    .state
                    .distribution
                    .hand(current_position)
                    .give_one_at(index);
                turn.put(current_position, given_card);
                match turn.master_card() {
                    None => {
                        println!(
                            "First card is {given_card}, so player {current_position} becomes master"
                        );
                        turn.set_master_position(current_position);
                    }
                    Some(master_card) => {
                        if master_card.master(given_card, self.state.trump_color) {
                            println!(
                                "Master card is {master_card}, so player {current_position} stays master"
                            );
                        } else {
                            println!(
                                "Master card is {given_card}, so player {current_position} becomes master"
                            );
                            turn.set_master_position(current_position);
                        }
                    }
                }
                if turn.finished() {
                    break;
                }
                current_position = current_position.next();
            }

            match turn.master_position() {
                None => {
                    return Err(BeloteErrorKind::InvalidCase(
                        "no master position at the end of turn".to_string(),
                    ))
                }
                Some(master_position) => {
                    let cards = turn.take();
                    let turn_points = cards.points(self.state.trump_color);
                    println!("Fold master is player {}", master_position);
                    stack.append(cards);
                    if self.state.taker.team() == master_position.team() {
                        attack_points += turn_points;
                    }
                }
            }

            println!("Stack size: {}", stack.len());
        }

        if current_position.team() == self.state.taker.team() {
            attack_points += 10;
        }

        let (final_attack_points, final_defense_points): (u64, u64) = match attack_points {
            0 => (0, 252),                                    // capot for defense
            1..=80 => (0, 162),                               // chute
            81 => (81, 81),                                   // litige
            82..=161 => (attack_points, 162 - attack_points), // reussite
            162 => (252, 0),                                  // capot for attack
            _ => {
                return Err(BeloteErrorKind::InvalidCase(format!(
                    "bad points number : {attack_points}"
                )))
            }
        };

        for team in Team::iter() {
            if belote_rebelote == Some(team) {
                self.points[team] += 20;
            }
            if team == self.state.taker.team() {
                self.points[team] += final_attack_points;
            } else {
                self.points[team] += final_defense_points;
            }
        }
        Ok(PlayingResult::NextGame(Game {
            deck: stack,
            players: self.players,
            state: self.state.distribution.next(),
            points: self.points,
        }))
    }
}
