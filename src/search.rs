use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    iter::{repeat_n, repeat_with},
};

use builder::SearchQueryBuilder;
use sqlx::{query, Execute};

use crate::{utils::hash_t, Work};

pub mod builder;
mod parse;

#[derive(PartialEq, Eq, Debug)]
pub enum QueryTerm {
    Tag(String),
    Title(String),
    Author(String),
    Caption(String),
    Url(String),
}

impl Display for QueryTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryTerm::Tag(text) => write!(f, r#"tag:"{text}""#),
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

#[derive(PartialEq, Eq, Debug)]
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

impl Query {
    pub fn parse(query: &str) -> anyhow::Result<Self> {
        let lower_query = query.to_lowercase();
        let (left, result) = parse::query(&lower_query)
            .map_err(|err| anyhow::anyhow!("Error parsing query: {err}"))?;

        if !left.is_empty() {
            anyhow::bail!("Query has invalid syntax, could not parse: {left}");
        }

        Ok(result.into_normalized())
    }

    pub fn table_name(&self) -> String {
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
        self._print_query_tree(0);
    }

    fn _print_query_tree(&self, indentation: usize) {
        let indent: String = repeat_n(" ", indentation).collect();

        match self {
            Query::Term(query_term) => println!("{indent}{query_term}"),
            Query::Not(query) => {
                println!("{indent}NOT (");
                query._print_query_tree(indentation + 2);
                println!("{indent})");
            }
            Query::And(queries) => {
                println!("{indent}AND (");
                for query in queries {
                    query._print_query_tree(indentation + 2);
                }
                println!("{indent})");
            }
            Query::Or(queries) => {
                println!("{indent}OR (");
                for query in queries {
                    query._print_query_tree(indentation + 2);
                }
                println!("{indent})");
            }
        }
    }

    fn or(queries: Vec<Self>) -> Self {
        assert!(!queries.is_empty());

        if queries.len() == 1 {
            return queries.into_iter().next().unwrap();
        }

        Self::Or(queries)
    }

    fn and(queries: Vec<Self>) -> Self {
        assert!(!queries.is_empty());

        if queries.len() == 1 {
            return queries.into_iter().next().unwrap();
        }

        Self::And(queries)
    }
}

pub async fn search(db: &sqlx::SqlitePool, search_query: &str) -> anyhow::Result<Vec<Work>> {
    let query = Query::parse(search_query)?;

    let mut builder = SearchQueryBuilder::new();

    let table = builder.push_query_table(&query);

    builder.query_builder.push(format_args!(
        "SELECT * FROM works WHERE work_id IN (SELECT work_id FROM {table});\n"
    ));

    builder.drop_tables();

    let built = builder.query_builder.build_query_as();

    Ok(built.fetch_all(db).await?)
}

pub async fn list(db: &sqlx::SqlitePool) -> anyhow::Result<Vec<Work>> {
    Ok(sqlx::query_as("SELECT * FROM works;").fetch_all(db).await?)
}

#[cfg(test)]
mod tests {
    use crate::utils::hash_t;

    use super::Query;

    #[test]
    fn test_normalization() {
        let query1 = Query::parse("a b c").unwrap();
        let query2 = Query::parse("a c b").unwrap();
        let query3 = Query::parse("c b a").unwrap();

        assert_eq!(hash_t(&query1), hash_t(&query2));
        assert_eq!(hash_t(&query1), hash_t(&query3));
    }

    #[test]
    fn test_deep_normalization() {
        let query1 = Query::parse("(a or b) and (c or d) and not e").unwrap();
        let query2 = Query::parse("(d or c) and not e (a or b)").unwrap();

        assert_eq!(hash_t(&query1), hash_t(&query2));
    }

    #[test]
    fn test_repetitive_normalization() {
        let query1 = Query::parse("a").unwrap();
        let query2 = Query::parse("a a a a a").unwrap();
        let query3 = Query::parse("a:a").unwrap();

        assert_eq!(hash_t(&query1), hash_t(&query2));
        assert_ne!(hash_t(&query1), hash_t(&query3));
    }
}
