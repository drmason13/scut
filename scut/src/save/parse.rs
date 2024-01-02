use std::fmt;

use parsely::*;

use crate::{config::TeamNames, Turn};

use super::{Save, Side};

pub fn parse_side(team_names: &TeamNames) -> impl Parse<Output = Side> + '_ {
    switch([
        (itoken(&team_names.allies), Side::Allies),
        (itoken(&team_names.axis), Side::Axis),
    ])
}

pub fn parse_player(input: &str) -> ParseResult<'_, String> {
    alpha().many(1..100).map(|s| s.to_string()).parse(input)
}

pub fn parse_turn(input: &str) -> ParseResult<'_, u32> {
    digit()
        .many(1..5)
        .try_map(|n| n.parse::<u32>())
        .parse(input)
}

pub fn parse_part(input: &str) -> ParseResult<'_, String> {
    let (_, remaining) = "part ".optional().lex(input)?;
    alphanum()
        .many(1..100)
        .map(|s| s.to_string())
        .parse(remaining)
}

pub fn parse_save(team_names: &TeamNames) -> impl Parse<Output = Save> + '_ {
    // Side Start Turn
    // "Axis[ ]start 123";
    let side_start_turn = parse_side(team_names)
        .then_skip(' '.many(0..))
        .then_skip(itoken("start"))
        .then_skip(' '.many(0..))
        .then(parse_turn)
        .map(|(side, turn)| Save {
            player: None,
            turn: Turn::new(side, turn),
            part: None,
        });

    // Side[ ]Player[ ]Turn
    // "Axis DM 123";
    let side_player_turn = parse_side(team_names)
        .then_skip(' '.many(0..))
        .then(parse_player)
        .then_skip(' '.many(0..))
        .then(parse_turn)
        .then_skip(' '.many(0..))
        .then(parse_part.optional())
        .map(|(((side, player), turn), part)| Save {
            player: Some(player),
            turn: Turn::new(side, turn),
            part,
        });

    // Side[ ]Turn[ ]Player
    // "Axis 123 DM";
    let side_turn_player = parse_side(team_names)
        .then_skip(' '.many(0..))
        .then(parse_turn)
        .then_skip(' '.many(0..))
        .then(parse_player)
        .then_skip(' '.many(0..))
        .then(parse_part.optional())
        .map(|(((side, turn), player), part)| Save {
            player: Some(player),
            turn: Turn::new(side, turn),
            part,
        });

    // Side[ ]Turn
    // "Axis 123";
    let side_turn = parse_side(team_names)
        .then_skip(' '.many(0..))
        .then(parse_turn)
        .map(|(side, turn)| Save {
            player: None,
            turn: Turn::new(side, turn),
            part: None,
        });

    (side_start_turn)
        .or(side_turn_player)
        .or(side_player_turn)
        .or(side_turn)
}

#[derive(Debug, PartialEq)]
pub struct ParseSaveError;

impl fmt::Display for ParseSaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse save filename")
    }
}

impl std::error::Error for ParseSaveError {}
