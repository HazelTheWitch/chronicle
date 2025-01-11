mod args;
mod bulk;
mod logging;
mod table;
mod tag;
mod utils;
mod work;

use std::{fs, process::ExitCode};

use args::{Arguments, Command};
use chronicle::{Chronicle, DEFAULT_CONFIG};
use clap::Parser;
use console::{Style, Term};
use directories::ProjectDirs;
use indicatif::ProgressStyle;
use lazy_static::lazy_static;
use logging::initialize_logging;
use tag::execute_tag_expression;
use tokio::sync::OnceCell;
use tracing::error;
use work::work_command;

lazy_static! {
    pub static ref ARGUMENTS: Arguments = Arguments::parse();
    pub static ref PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("dev.setaria", "HazelTheWitch", "chronicle-cli")
            .expect("could not get project directories");
    pub static ref TERMINAL: Term = Term::stdout();
    pub static ref PREFIX_STYLE: Style = Style::new().green().bold();
    pub static ref ERROR_STYLE: Style = Style::new().red().bold();
    pub static ref SPINNER_STYLE: ProgressStyle =
        ProgressStyle::with_template("[{elapsed}] {spinner:^3} {prefix} {wide_msg}")
            .expect("invalid format");
    pub static ref PROGRESS_STYLE: ProgressStyle =
        ProgressStyle::with_template("[{elapsed}] [{bar:16}] {prefix} {wide_msg}")
            .expect("invalid format")
            .progress_chars("=> ");
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

fn ensure_config() -> anyhow::Result<()> {
    if !fs::exists(&ARGUMENTS.config)? {
        fs::create_dir_all(
            ARGUMENTS
                .config
                .parent()
                .ok_or_else(|| anyhow::anyhow!("could not get config path directory"))?,
        )?;
        fs::write(&ARGUMENTS.config, DEFAULT_CONFIG)?;
    }

    Ok(())
}

async fn fallible() -> anyhow::Result<ExitCode> {
    if let Err(err) = ensure_config() {
        error!("Could not create the config: {err}");
        return Ok(ExitCode::FAILURE);
    }

    match &ARGUMENTS.command {
        Command::Work { command } => work_command(command).await,
        Command::Tag { expression } => execute_tag_expression(expression).await,
        Command::Author { command } => todo!(),
        Command::Service { command } => todo!(),
        Command::Bulk { command, tasks } => bulk::bulk(*tasks, command).await,
    }
}

pub fn write_with(string: &str, style: &Style) -> anyhow::Result<()> {
    let Some((first_word, rest)) = string.split_once(" ") else {
        TERMINAL.write_line(&format!("{}", style.apply_to(string)))?;

        return Ok(());
    };

    TERMINAL.write_line(&format!("{} {}", style.apply_to(first_word), rest))?;

    Ok(())
}

pub fn write_success(string: &str) -> anyhow::Result<()> {
    write_with(string, &*PREFIX_STYLE)
}

pub fn write_failure(string: &str) -> anyhow::Result<()> {
    write_with(string, &*ERROR_STYLE)
}

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
    initialize_logging()?;

    let result = fallible().await;

    if let Err(err) = &result {
        error!("Chronicle encountered an error which it could not recover from, please report this at https://github.com/HazelTheWitch/chronicle/issues/new");
        error!("{err}");
    }

    result
}
