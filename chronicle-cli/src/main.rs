mod args;
mod logging;
mod ui;

use std::{
    fs::{self, OpenOptions},
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use anyhow::bail;
use args::{Arguments, Command, ServiceCommand};
use chronicle::{
    author::AuthorQuery,
    import::SERVICES,
    models::{Author, Tag, Work},
    record::Record,
    search::QueryTerm,
    Chronicle, Config,
};
use clap::Parser;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use logging::initialize_logging;
use tokio::sync::OnceCell;
use tracing::{error, info, warn};
use uuid::Uuid;

lazy_static! {
    pub static ref ARGUMENTS: Arguments = Arguments::parse();
    pub static ref PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("dev.setaria", "HazelTheWitch", "chronicle-cli")
            .expect("could not get project directories");
}

static CHRONICLE: OnceCell<Chronicle> = OnceCell::const_new();

pub async fn get_chronicle() -> &'static Chronicle {
    CHRONICLE
        .get_or_init(|| async {
            Chronicle::from_path(ARGUMENTS.config.clone())
                .await
                .expect("could not load config file")
        })
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{:?}", *ARGUMENTS);

    Ok(())
}
