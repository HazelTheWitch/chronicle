use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of, space1},
    combinator::recognize,
    error::VerboseError,
    multi::{many0, many1},
    sequence::{delimited, pair},
    IResult,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("could not parse: {0}")]
    Parse(String),
    #[error("parser did not finish: '{0}'")]
    ParserDidNotFinish(String),
}

impl<'s> From<nom::Err<VerboseError<&'s str>>> for ParseError {
    fn from(value: nom::Err<VerboseError<&'s str>>) -> Self {
        Self::Parse(value.to_string())
    }
}

pub type ParseResult<'s, T> = IResult<&'s str, T, nom::error::VerboseError<&'s str>>;

pub fn identifier(input: &str) -> ParseResult<&str> {
    recognize(pair(
        alphanumeric1,
        many0(alt((tag("_"), tag("-"), alphanumeric1))),
    ))(input)
}

pub fn quote(input: &str) -> ParseResult<&str> {
    alt((tag("'"), tag("\"")))(input)
}

pub fn special_chars(input: &str) -> ParseResult<&str> {
    recognize(many1(one_of("$.+!*'(),;/?:@=&")))(input)
}

pub fn string(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    alt((
        delimited(
            quote,
            recognize(many1(alt((space1, identifier, special_chars)))),
            quote,
        ),
        identifier,
    ))(input)
}
