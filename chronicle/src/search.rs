use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    io::{self, stdout, Write},
    iter::{once, repeat_n},
    str::FromStr,
};

use builder::SearchQueryBuilder;
use sqlx::{Execute, Sqlite, Transaction};

use crate::{models::Work, parse::ParseError, tag::DiscriminatedTag, utils::hash_t, Chronicle};

pub mod builder;
pub(crate) mod parse;

// TODO: Add Id query term
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum QueryTerm {
    Tag(DiscriminatedTag),
    Title(String),
    Author(String),
    Caption(String),
    Url(String),
}

impl Display for QueryTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryTerm::Tag(tag) => write!(f, r#"tag:"{tag}""#),
            QueryTerm::Title(text) => write!(f, r#"title:"{text}""#),
            QueryTerm::Author(text) => write!(f, r#"author:"{text}""#),
            QueryTerm::Caption(text) => write!(f, r#"caption:"{text}""#),
            QueryTerm::Url(text) => write!(f, r#"url:"{text}""#),
        }
    }
}

fn hash_and(state: &mut impl Hasher, number: u8, value: impl Hash) {
    state.write_u8(number);
    value.hash(state);
}

impl Hash for QueryTerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            QueryTerm::Tag(text) => hash_and(state, 1, text),
            QueryTerm::Title(text) => hash_and(state, 2, text),
            QueryTerm::Author(text) => hash_and(state, 3, text),
            QueryTerm::Caption(text) => hash_and(state, 4, text),
            QueryTerm::Url(text) => hash_and(state, 5, text),
        }
    }
}

impl From<QueryTerm> for Query {
    fn from(term: QueryTerm) -> Self {
        Self::Term(term)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Query {
    Term(QueryTerm),
    Not(Box<Query>),
    And(Vec<Query>),
    Or(Vec<Query>),
}

impl Hash for Query {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Query::Term(query_term) => hash_and(state, 1, query_term),
            Query::Not(query) => hash_and(state, 2, query),
            Query::And(terms) => hash_and(state, 3, terms),
            Query::Or(terms) => hash_and(state, 4, terms),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Operation {
    And,
    Or,
}

impl FromStr for Query {
    type Err = ParseError;

    fn from_str(query: &str) -> Result<Self, Self::Err> {
        let lower_query = query.to_lowercase();
        let (left, result) = parse::query(&lower_query)?;

        if !left.is_empty() {
            return Err(ParseError::ParserDidNotFinish(left.to_string()));
        }

        Ok(result.into_normalized())
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Term(query_term) => write!(f, "{query_term}"),
            Query::Not(query) => write!(f, "-{query}"),
            Query::And(terms) => write!(
                f,
                "({})",
                terms
                    .iter()
                    .map(|term| term.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Query::Or(terms) => write!(
                f,
                "({})",
                terms
                    .iter()
                    .map(|term| term.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl Query {
    fn table_name(&self) -> String {
        format!("t{hash:X}", hash = hash_t(self))
    }

    pub fn into_normalized(self) -> Self {
        match self {
            Query::And(mut terms) => {
                terms = terms.into_iter().map(Self::into_normalized).collect();
                terms.sort_by_key(hash_t);
                terms.dedup_by_key(|term| hash_t(term));

                if terms.len() == 1 {
                    return terms.remove(0);
                }

                Query::And(terms)
            }
            Query::Or(mut terms) => {
                terms = terms.into_iter().map(Self::into_normalized).collect();
                terms.sort_by_key(hash_t);
                terms.dedup_by_key(|term| hash_t(term));

                if terms.len() == 1 {
                    return terms.remove(0);
                }

                Query::Or(terms)
            }
            query => query,
        }
    }

    pub fn print_query_tree(&self) {
        self.write_query_tree(&mut stdout())
            .expect("could not write to stdout");
    }

    pub fn write_query_tree(&self, writer: &mut impl Write) -> io::Result<()> {
        self._write_query_tree(0, 0, writer)
    }

    pub fn not(self) -> Self {
        Query::Not(Box::new(self))
    }

    pub fn and(self, other: impl IntoIterator<Item = Self>) -> Self {
        Self::And(once(self).chain(other.into_iter()).collect())
    }

    pub fn or(self, other: impl IntoIterator<Item = Self>) -> Self {
        Self::Or(once(self).chain(other.into_iter()).collect())
    }

    pub fn operation_count(&self) -> usize {
        match self {
            Query::Term(_) => 1,
            Query::Not(query) => 1 + query.operation_count(),
            Query::And(terms) => {
                1 + terms
                    .iter()
                    .map(|term| term.operation_count())
                    .sum::<usize>()
            }
            Query::Or(terms) => {
                1 + terms
                    .iter()
                    .map(|term| term.operation_count())
                    .sum::<usize>()
            }
        }
    }

    fn _write_query_tree(
        &self,
        indentation: usize,
        negations: usize,
        writer: &mut impl Write,
    ) -> io::Result<()> {
        let indent: String = repeat_n(" ", indentation - negations).collect();
        let negate: String = repeat_n("-", negations).collect();

        match self {
            Query::Term(query_term) => {
                writeln!(writer, "{indent}{negate}{query_term}")?;
            }
            Query::Not(query) => {
                query._write_query_tree(indentation, negations + 1, writer)?;
            }
            Query::And(queries) => {
                writeln!(writer, "{indent}{negate}AND (")?;
                for query in queries {
                    query._write_query_tree(indentation + 2, 0, writer)?;
                }
                writeln!(writer, "{indent}{negate})")?;
            }
            Query::Or(queries) => {
                writeln!(writer, "{indent}{negate}OR (")?;
                for query in queries {
                    query._write_query_tree(indentation + 2, 0, writer)?;
                }
                writeln!(writer, "{indent}{negate})")?;
            }
        }

        Ok(())
    }

    fn new_or(queries: Vec<Self>) -> Self {
        assert!(!queries.is_empty());

        if queries.len() == 1 {
            return queries.into_iter().next().unwrap();
        }

        Self::Or(queries)
    }

    fn new_and(queries: Vec<Self>) -> Self {
        assert!(!queries.is_empty());

        if queries.len() == 1 {
            return queries.into_iter().next().unwrap();
        }

        Self::And(queries)
    }
}

impl Work {
    pub async fn get_all(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<Work>, crate::Error> {
        Ok(sqlx::query_as("SELECT * FROM works;")
            .fetch_all(&mut **tx)
            .await?)
    }

    pub async fn search(
        tx: &mut Transaction<'_, Sqlite>,
        query: &Query,
    ) -> Result<Vec<Work>, crate::Error> {
        let mut builder = SearchQueryBuilder::new();

        let table = builder.push_query_table(query);

        builder.query_builder.push(format_args!(
            "SELECT * FROM works WHERE work_id IN (SELECT work_id FROM {table});\n"
        ));

        builder.drop_tables();

        let built = builder.query_builder.build_query_as();

        Ok(built.fetch_all(&mut **tx).await?)
    }

    pub async fn search_by_str(
        tx: &mut Transaction<'_, Sqlite>,
        search_query: &str,
    ) -> Result<Vec<Work>, crate::Error> {
        let query = Query::from_str(search_query)?;

        Self::search(tx, &query).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::utils::hash_t;

    use super::Query;

    #[test]
    fn test_normalization() {
        let query1 = Query::from_str("a b c").unwrap();
        let query2 = Query::from_str("a c b").unwrap();
        let query3 = Query::from_str("c b a").unwrap();

        assert_eq!(hash_t(&query1), hash_t(&query2));
        assert_eq!(hash_t(&query1), hash_t(&query3));
    }

    #[test]
    fn test_deep_normalization() {
        let query1 = Query::from_str("(a or b) and (c or d) and not e").unwrap();
        let query2 = Query::from_str("(d or c) and not e (a or b)").unwrap();

        assert_eq!(hash_t(&query1), hash_t(&query2));
    }

    #[test]
    fn test_repetitive_normalization() {
        let query1 = Query::from_str("a").unwrap();
        let query2 = Query::from_str("a a a a a").unwrap();
        let query3 = Query::from_str("a:a").unwrap();

        assert_eq!(hash_t(&query1), hash_t(&query2));
        assert_ne!(hash_t(&query1), hash_t(&query3));
    }
}
