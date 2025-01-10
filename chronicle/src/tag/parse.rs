use nom::{
    branch::alt, character::complete::char, combinator::map, multi::separated_list1,
    sequence::delimited,
};

use crate::parse::{string, ParseResult};

fn tag_sequence(i: &str) -> ParseResult<Vec<&str>> {
    alt((
        map(string, |tag| vec![tag]),
        delimited(char('('), separated_list1(char(','), string), char(')')),
    ))(i)
}

pub fn tag_expression(i: &str) -> ParseResult<Vec<Vec<&str>>> {
    separated_list1(char('/'), tag_sequence)(i)
}

#[cfg(test)]
mod test {
    use crate::tag::parse::tag_expression;

    #[test]
    fn test_expression() {
        assert_eq!(
            Ok(("", vec![vec!["a"], vec!["b", "c"], vec!["d"]])),
            tag_expression("a<(b,c)<d")
        )
    }
}
