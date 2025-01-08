use sqlx::{QueryBuilder, Sqlite};

use crate::utils::{hash_t, hash_t_hex};

use super::{Query, QueryTerm};

impl QueryTerm {
    fn push_select<'args>(&'args self, b: &mut QueryBuilder<'args, Sqlite>) {
        match self {
            QueryTerm::Tag(tag) => {
                b.push("WITH RECURSIVE implied(tag_id) AS (SELECT (SELECT id FROM tags WHERE name = ")
                    .push_bind(tag)
                    .push(") UNION SELECT target FROM meta_tags JOIN implied ON meta_tags.tag = implied.tag_id) SELECT work_id FROM works WHERE work_id IN (SELECT work_id FROM work_tags JOIN implied ON work_tags.tag = implied.tag_id)");
            }
            QueryTerm::Title(title) => {
                b.push("SELECT work_id FROM works WHERE title LIKE '%' || ")
                    .push_bind(title)
                    .push(" || '%'");
            }
            QueryTerm::Author(author) => {
                b.push("SELECT work_id FROM works JOIN authors ON works.author_id = authors.author_id WHERE authors.name = ")
                    .push_bind(author);
            }
            QueryTerm::Caption(caption) => {
                b.push("SELECT work_id FROM works WHERE caption LIKE '%' || ")
                    .push_bind(caption)
                    .push(" || '%'");
            }
            QueryTerm::Url(url) => {
                b.push("SELECT work_id FROM works WHERE url = ")
                    .push_bind(url);
            }
        }
    }
}

pub struct SearchQueryBuilder<'args> {
    pub query_builder: QueryBuilder<'args, Sqlite>,
    added_tables: Vec<String>,
    has_everything: bool,
}

impl<'args> SearchQueryBuilder<'args> {
    pub fn new() -> Self {
        Self {
            query_builder: QueryBuilder::new(""),
            added_tables: Vec::new(),
            has_everything: false,
        }
    }

    /// Pushes a table named `everything` which contains every work.
    pub fn push_everything(&mut self) {
        self.query_builder
            .push("CREATE TEMP TABLE everything AS SELECT work_id FROM works;");
        self.added_tables.push(String::from("everything"));
        self.has_everything = true;
    }

    /// Visit a build the corresponding SQL query returning the name of the table.
    ///
    /// * `query`: the query to build a SQL query for
    pub fn push_query_table(&mut self, query: &'args Query) -> String {
        let table_name = query.table_name();

        if self.added_tables.contains(&table_name) {
            return table_name;
        }

        match query {
            Query::Term(query_term) => {
                self.query_builder
                    .push(format_args!("CREATE TEMP TABLE {table_name} AS "));

                query_term.push_select(&mut self.query_builder);

                self.query_builder.push(";\n");
            }
            Query::Not(query) => {
                if !self.has_everything {
                    self.push_everything();
                }

                let child = self.push_query_table(query);

                self.query_builder
                    .push(format_args!("CREATE TEMP TABLE {table_name} AS SELECT everything.work_id FROM everything LEFT OUTER JOIN {child} ON everything.work_id = {child}.work_id WHERE {child}.work_id IS null;\n"));
            }
            Query::And(terms) => {
                let mut children: Vec<String> = terms
                    .iter()
                    .map(|term| self.push_query_table(term))
                    .collect();

                if children.len() == 1 {
                    return children.remove(0);
                }

                self.query_builder
                    .push(format_args!("CREATE TEMP TABLE {table_name} AS "));

                for (i, child) in children.into_iter().enumerate() {
                    self.query_builder
                        .push(format_args!("SELECT work_id FROM {child}"));

                    if i != terms.len() - 1 {
                        self.query_builder.push(" INTERSECT ");
                    }
                }

                self.query_builder.push(";\n");
            }
            Query::Or(terms) => {
                let mut children: Vec<String> = terms
                    .iter()
                    .map(|term| self.push_query_table(term))
                    .collect();

                if children.len() == 1 {
                    return children.remove(0);
                }

                self.query_builder
                    .push(format_args!("CREATE TEMP TABLE {table_name} AS "));

                for (i, child) in children.into_iter().enumerate() {
                    self.query_builder
                        .push(format_args!("SELECT work_id FROM {child}"));

                    if i != terms.len() - 1 {
                        self.query_builder.push(" UNION ");
                    }
                }

                self.query_builder.push(";\n");
            }
        }

        self.added_tables.push(table_name.clone());

        table_name
    }

    pub fn drop_tables(&mut self) {
        for table in self.added_tables.drain(..) {
            self.query_builder
                .push(format_args!("DROP TABLE {table};\n"));
        }
    }
}
