use strum::Display;

mod author;
mod tag;
mod work;

#[derive(Debug, Display)]
pub enum ModelKind {
    Work,
    Author,
    Tag,
}

pub use author::*;
pub use tag::*;
pub use work::*;
