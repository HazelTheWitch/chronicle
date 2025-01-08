use std::{
    fmt::Display,
    iter::{repeat_n, repeat_with},
};

use sqlx::query;

mod parse;

#[derive(PartialEq, Eq, Debug)]
pub enum QueryTerm {
    Tag(String),
    Title(String),
    Artist(String),
    Caption(String),
    Url(String),
}

impl Display for QueryTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryTerm::Tag(text) => write!(f, r#"tag:"{text}""#),
            QueryTerm::Title(text) => write!(f, r#"title:"{text}""#),
            QueryTerm::Artist(text) => write!(f, r#"artist:"{text}""#),
            QueryTerm::Caption(text) => write!(f, r#"caption:"{text}""#),
            QueryTerm::Url(text) => write!(f, r#"url:"{text}""#),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Query {
    Term(QueryTerm),
    Not(Box<Query>),
    And(Vec<Query>),
    Or(Vec<Query>),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Operation {
    And,
    Or,
}

impl Query {
    pub fn parse(query: &str) -> anyhow::Result<Self> {
        let lower_query = query.to_lowercase();
        let (left, mut result) = parse::query(&lower_query)
            .map_err(|err| anyhow::anyhow!("Error parsing query: {err}"))?;

        if !left.is_empty() {
            anyhow::bail!("Query has invalid syntax, could not parse: {left}");
        }

        Ok(result)
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

    pub fn or(queries: Vec<Self>) -> Self {
        assert!(!queries.is_empty());

        if queries.len() == 1 {
            return queries.into_iter().next().unwrap();
        }

        Self::Or(queries)
    }

    pub fn and(queries: Vec<Self>) -> Self {
        assert!(!queries.is_empty());

        if queries.len() == 1 {
            return queries.into_iter().next().unwrap();
        }

        Self::And(queries)
    }
}

pub async fn search(search_query: &str) {}
