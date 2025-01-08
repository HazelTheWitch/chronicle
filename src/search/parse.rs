use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of, space0, space1},
    combinator::{fail, map, recognize},
    multi::{many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair},
    IResult,
};

use super::{Query, QueryTerm};

fn term_kind(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    alt((
        tag("tag"),
        tag("title"),
        tag("artist"),
        tag("caption"),
        tag("url"),
        tag("t"),
        tag("a"),
        tag("c"),
        tag("u"),
    ))(input)
}

fn identifier(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    recognize(many1(alt((tag("_"), tag("-"), alphanumeric1))))(input)
}

fn quote(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    alt((tag("'"), tag("\"")))(input)
}

fn special_chars(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    recognize(many1(one_of("$.+!*'(),;/?:@=&")))(input)
}

fn string(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    alt((
        delimited(
            quote,
            recognize(many1(alt((space1, identifier, special_chars)))),
            quote,
        ),
        preceded(nom::combinator::not(not), identifier),
    ))(input)
}

fn tagged_term(input: &str) -> IResult<&str, QueryTerm, nom::error::VerboseError<&str>> {
    let (i, (kind, string)) = separated_pair(term_kind, tag(":"), string)(input)?;

    let text = string.to_owned();

    let term = match kind {
        "tag" => QueryTerm::Tag(text),
        "t" | "title" => QueryTerm::Title(text),
        "a" | "artist" => QueryTerm::Artist(text),
        "c" | "caption" => QueryTerm::Caption(text),
        "u" | "url" => QueryTerm::Url(text),
        _ => return fail("invalid term tag"),
    };

    Ok((i, term))
}

fn term(input: &str) -> IResult<&str, QueryTerm, nom::error::VerboseError<&str>> {
    preceded(
        nom::combinator::not(alt((and_separator, or_separator))),
        alt((
            tagged_term,
            map(string, |string| QueryTerm::Tag(string.to_owned())),
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
        Query::and,
    )(input)
}

fn or_sequence(input: &str) -> IResult<&str, Query, nom::error::VerboseError<&str>> {
    map(separated_list1(or_separator, and_sequence), Query::or)(input)
}

pub fn query(input: &str) -> IResult<&str, Query, nom::error::VerboseError<&str>> {
    delimited(space0, or_sequence, space0)(input)
}

#[cfg(test)]
mod tests {
    use nom::Parser;

    use crate::search::{
        parse::{query, term},
        Query, QueryTerm,
    };

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
            "t", "title", "tag", "a", "artist", "c", "caption", "u", "url",
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
        assert_eq!(
            term("arlefuri"),
            Ok(("", QueryTerm::Tag(String::from("arlefuri"))))
        );
    }

    #[test]
    fn test_query() {
        assert_eq!(
            query(r#"arlecchino and (furina or yae_miko t:"title here" -silly)"#),
            Ok((
                "",
                Query::And(vec![
                    Query::Term(QueryTerm::Tag(String::from("arlecchino"))),
                    Query::Or(vec![
                        Query::Term(QueryTerm::Tag(String::from("furina"))),
                        Query::And(vec![
                            Query::Term(QueryTerm::Tag(String::from("yae_miko"))),
                            Query::Term(QueryTerm::Title(String::from("title here"))),
                            Query::Not(Box::new(Query::Term(QueryTerm::Tag(String::from(
                                "silly"
                            ))))),
                        ]),
                    ]),
                ])
            ))
        );
    }
}
