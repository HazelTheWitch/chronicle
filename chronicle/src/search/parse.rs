use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of, space0, space1},
    combinator::{fail, map, recognize},
    multi::{many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::{parse::string, tag::parse::discriminated_tag};

use super::{Query, QueryTerm};

fn term_kind(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    alt((
        tag("tag"),
        tag("title"),
        tag("artist"),
        tag("author"),
        tag("caption"),
        tag("url"),
        tag("t"),
        tag("a"),
        tag("c"),
        tag("u"),
    ))(input)
}

fn tagged_term(input: &str) -> IResult<&str, QueryTerm, nom::error::VerboseError<&str>> {
    let (i, kind) = terminated(term_kind, tag(":"))(input)?;

    match kind {
        "tag" => map(discriminated_tag, |t| QueryTerm::Tag(t.into()))(i),
        "t" | "title" => map(string, |s| QueryTerm::Title(s.to_owned()))(i),
        "a" | "artist" | "author" => map(string, |s| QueryTerm::Author(s.to_owned()))(i),
        "c" | "caption" => map(string, |s| QueryTerm::Caption(s.to_owned()))(i),
        "u" | "url" => map(string, |s| QueryTerm::Url(s.to_owned()))(i),
        _ => return fail("invalid term tag"),
    }
}

fn term(input: &str) -> IResult<&str, QueryTerm, nom::error::VerboseError<&str>> {
    preceded(
        nom::combinator::not(alt((and_separator, or_separator))),
        alt((
            tagged_term,
            map(discriminated_tag, |t| QueryTerm::Tag(t.into())),
        )),
    )(input)
}

fn not(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    alt((recognize(pair(tag("not"), space1)), tag("!"), tag("-")))(input)
}

fn term_query(input: &str) -> IResult<&str, Query, nom::error::VerboseError<&str>> {
    alt((
        paren_query,
        map(term, Query::Term),
        preceded(
            not,
            map(preceded(nom::combinator::not(not), term_query), |query| {
                Query::Not(Box::new(query))
            }),
        ),
    ))(input)
}

fn paren_query(input: &str) -> IResult<&str, Query, nom::error::VerboseError<&str>> {
    delimited(tag("("), query, tag(")"))(input)
}

fn and_separator(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    delimited(space0, alt((tag("and"), tag("&"), tag("&&"))), space0)(input)
}

fn or_separator(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    delimited(
        space0,
        alt((tag("or"), tag("|"), tag("||"), tag(","))),
        space0,
    )(input)
}

fn and_sequence(input: &str) -> IResult<&str, Query, nom::error::VerboseError<&str>> {
    map(
        separated_list1(
            alt((and_separator, space1)),
            preceded(nom::combinator::not(or_separator), term_query),
        ),
        Query::new_and,
    )(input)
}

fn or_sequence(input: &str) -> IResult<&str, Query, nom::error::VerboseError<&str>> {
    map(separated_list1(or_separator, and_sequence), Query::new_or)(input)
}

pub fn query(input: &str) -> IResult<&str, Query, nom::error::VerboseError<&str>> {
    delimited(space0, or_sequence, space0)(input)
}

#[cfg(test)]
mod tests {
    use nom::Parser;

    use crate::search::{parse::term, QueryTerm};

    use super::term_kind;

    fn assert_matches<'s>(
        mut p: impl Parser<&'s str, &'s str, nom::error::VerboseError<&'s str>>,
        test: &'s str,
    ) {
        assert_eq!(p.parse(test), Ok(("", test)));
    }

    #[test]
    fn test_term_kind() {
        for kind in &[
            "t", "title", "tag", "a", "artist", "author", "c", "caption", "u", "url",
        ] {
            assert_matches(term_kind, kind);
        }

        assert!(term_kind("").is_err());
        assert!(term_kind("notakind").is_err());
        assert_eq!(term_kind("ti"), Ok(("i", "t")));
    }

    #[test]
    fn test_term() {
        assert_eq!(
            term("t:Splatoon"),
            Ok(("", QueryTerm::Title(String::from("Splatoon"))))
        );
        assert_eq!(
            term(r#"t:"Ace Attorney""#),
            Ok(("", QueryTerm::Title(String::from("Ace Attorney"))))
        );
    }
}
