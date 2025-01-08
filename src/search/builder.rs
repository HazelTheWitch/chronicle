use sqlx::{QueryBuilder, Sqlite};

pub struct SearchQueryBuilder<'args> {
    pub query_builder: QueryBuilder<'args, Sqlite>,
}

impl<'args> SearchQueryBuilder<'args> {
    pub fn new() -> Self {
        Self {
            query_builder: QueryBuilder::new(""),
        }
    }

    // pub fn build_table(&mut self)
}
