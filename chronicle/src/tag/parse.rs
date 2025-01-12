use nom::{
    branch::alt,
    character::complete::char,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair},
};

use crate::{
    parse::{string, ParseResult},
    search::parse::query,
};

use super::TagExpression;

pub struct ParsedTag<'s> {
    pub name: &'s str,
    pub discriminator: Option<&'s str>,
}

pub fn discriminated_tag(i: &str) -> ParseResult<ParsedTag> {
    alt((
        map(
            separated_pair(string, char('#'), string),
            |(name, discriminator)| ParsedTag {
                name,
                discriminator: Some(discriminator),
            },
        ),
        map(string, |name| ParsedTag {
            name,
            discriminator: None,
        }),
    ))(i)
}

fn tag_sequence(i: &str) -> ParseResult<Vec<ParsedTag>> {
    alt((
        map(discriminated_tag, |tag| vec![tag]),
        delimited(
            char('('),
            separated_list1(char(','), discriminated_tag),
            char(')'),
        ),
    ))(i)
}

pub fn tag_expression(i: &str) -> ParseResult<TagExpression> {
    alt((
        map(
            separated_pair(
                delimited(char('<'), query, char('>')),
                char('/'),
                separated_list1(char('/'), tag_sequence),
            ),
            |(query, hierarchy)| TagExpression::new(Some(query), hierarchy),
        ),
        map(separated_list1(char('/'), tag_sequence), |hierarchy| {
            TagExpression::new(None, hierarchy)
        }),
    ))(i)
}

#[cfg(test)]
mod test {
    use crate::{
        search::{Query, QueryTerm},
        tag::{parse::tag_expression, TagExpression},
    };

    #[test]
    fn test_expression() {
        assert_eq!(
            Ok((
                "",
                TagExpression {
                    query: None,
                    hierarchy: vec![
                        vec!["a".into()],
                        vec!["b".into(), "c".into()],
                        vec!["d".into()]
                    ]
                }
            )),
            tag_expression("a/(b,c)/d")
        )
    }

    #[test]
    fn test_query() {
        assert_eq!(
            Ok((
                "",
                TagExpression {
                    query: Some(Query::Term(QueryTerm::Tag("hello".into()))),
                    hierarchy: vec![vec!["world".into()]],
                }
            )),
            tag_expression("<hello>/world")
        )
    }
}
