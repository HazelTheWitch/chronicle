use std::{
    borrow::Cow,
    fmt::Display,
    io::{self, Write},
    iter::repeat,
};

use console::{pad_str, truncate_str, Alignment, Term};

use crate::{
    args::{AuthorColumn, WorkColumn},
    fallible,
};

pub struct ColumnBehavior {
    pub size: usize,
    pub grow: bool,
    pub min_size: usize,
}

impl WorkColumn {
    pub fn behavior(&self) -> ColumnBehavior {
        match self {
            WorkColumn::Id => ColumnBehavior {
                size: 8,
                grow: false,
                min_size: 8,
            },
            WorkColumn::Path => ColumnBehavior {
                size: 42,
                grow: false,
                min_size: 12,
            },
            WorkColumn::Hash => ColumnBehavior {
                size: 9,
                grow: false,
                min_size: 9,
            },
            WorkColumn::Title => ColumnBehavior {
                size: 32,
                grow: false,
                min_size: 8,
            },
            WorkColumn::AuthorId => ColumnBehavior {
                size: 8,
                grow: false,
                min_size: 8,
            },
            WorkColumn::Caption => ColumnBehavior {
                size: 42,
                grow: true,
                min_size: 12,
            },
            WorkColumn::Url => ColumnBehavior {
                size: 48,
                grow: true,
                min_size: 12,
            },
            WorkColumn::Size => ColumnBehavior {
                size: 12,
                grow: false,
                min_size: 12,
            },
        }
    }
}

impl AuthorColumn {
    pub fn behavior(&self) -> ColumnBehavior {
        match self {
            AuthorColumn::Id => ColumnBehavior {
                size: 8,
                grow: false,
                min_size: 8,
            },
            AuthorColumn::Aliases => ColumnBehavior {
                size: 32,
                grow: true,
                min_size: 24,
            },
            AuthorColumn::Urls => ColumnBehavior {
                size: 32,
                grow: true,
                min_size: 24,
            },
        }
    }
}

pub struct Table<'t> {
    column_widths: Vec<usize>,
    term: &'t Term,
    current_column: usize,
}

impl<'t> Table<'t> {
    pub fn new(term: &'t Term, columns: Vec<ColumnBehavior>, mut size: usize) -> Self {
        let column_count = columns.len();

        size = size.max(column_count * 3);

        let mut sizes: Vec<usize> = columns.iter().map(|b| b.size).collect();

        let total: usize = sizes.iter().sum::<usize>() + column_count;

        let grow_columns: Vec<usize> = (0..columns.len()).filter(|i| columns[*i].grow).collect();

        if total > size {
            let mut shrink_needed = total - size;

            loop {
                let mut shrunk = false;

                for (item, column) in sizes.iter_mut().zip(columns.iter()) {
                    if shrink_needed == 0 {
                        break;
                    }

                    if *item <= column.min_size {
                        continue;
                    }

                    *item -= 1;
                    shrunk = true;
                    shrink_needed -= 1;
                }

                if !shrunk {
                    break;
                }
            }
        } else if total < size && !grow_columns.is_empty() {
            let grow_column_count = grow_columns.len();

            let growth_needed = size - total;

            let rem = growth_needed.rem_euclid(grow_column_count);

            grow_columns.into_iter().enumerate().for_each(|(i, index)| {
                *sizes.get_mut(index).expect("grow column out of bounds") +=
                    growth_needed / grow_column_count + if i < rem { 1 } else { 0 }
            });
        }

        Self {
            column_widths: sizes,
            term,
            current_column: 0,
        }
    }

    pub fn push_cell(&mut self, item: impl Display) -> io::Result<()> {
        let full_string = item.to_string().replace("\n", " ");

        let width = self.column_widths[self.current_column];

        let string = if self.term.is_term() {
            pad_str(&full_string, width, Alignment::Left, Some("..."))
        } else {
            Cow::Borrowed(full_string.as_str())
        };

        let ender = if self.current_column == self.column_widths.len() - 1 {
            "\n"
        } else {
            " "
        };

        self.term.write_fmt(format_args!("{string}{ender}"))?;

        self.current_column = (self.current_column + 1) % self.column_widths.len();

        Ok(())
    }
}
