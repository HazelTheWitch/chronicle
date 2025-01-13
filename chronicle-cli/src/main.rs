mod args;
mod author;
mod bulk;
mod logging;
mod table;
mod tag;
mod utils;
mod work;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    process::ExitCode,
};

use args::{Arguments, Command, ServiceCommand};
use author::author_command;
use chronicle::{
    import::{write_secrets, SERVICES},
    Chronicle, DEFAULT_CONFIG,
};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use console::{Style, Term};
use dialoguer::{Password, Select};
use directories::ProjectDirs;
use indicatif::ProgressStyle;
use lazy_static::lazy_static;
use logging::initialize_logging;
use tag::{execute_tag_expression, tag_command};
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
        fs::write(&ARGUMENTS.config, &*DEFAULT_CONFIG)?;
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
        Command::Tag { command } => tag_command(command).await,
        Command::Author { command } => author_command(command).await,
        Command::Service { command } => match command {
            ServiceCommand::Login { service } => {
                let services: Vec<_> = SERVICES
                    .iter()
                    .filter_map(|s| {
                        if s.secrets().is_empty() {
                            None
                        } else {
                            Some(s.name())
                        }
                    })
                    .collect();

                let service = match service {
                    Some(service) => service,
                    None => {
                        let index = Select::new()
                            .with_prompt("Select service to login")
                            .items(&services)
                            .interact()?;
                        &SERVICES[index].name().to_owned()
                    }
                };

                let Some(service) = SERVICES.iter().find(|s| s.name() == service) else {
                    write_failure(&format!("Unknown service {service}"))?;
                    return Ok(ExitCode::FAILURE);
                };

                let mut secrets = HashMap::new();

                for secret in service.secrets() {
                    let Ok(value) = Password::new().with_prompt(secret.to_string()).interact()
                    else {
                        write_failure("Failed to recieve secret")?;
                        return Ok(ExitCode::FAILURE);
                    };

                    secrets.insert(secret.to_string(), value);
                }

                write_secrets(service.name(), secrets)?;

                write_success(&format!(
                    "Successfully wrote secrets for {}",
                    service.name()
                ))?;

                Ok(ExitCode::SUCCESS)
            }
            ServiceCommand::List => {
                for service in SERVICES.iter() {
                    TERMINAL.write_line(service.name())?;
                }

                Ok(ExitCode::SUCCESS)
            }
        },
        Command::Bulk { command, tasks } => bulk::bulk(*tasks, command).await,
        Command::Completions { shell, output } => {
            let Some(shell) = shell.or_else(|| Shell::from_env()) else {
                write_failure("Failed detecting shell from environment, please provide a shell")?;
                return Ok(ExitCode::FAILURE);
            };

            let mut command = Arguments::command();

            if let Some(output) = output {
                let mut writer =
                    BufWriter::new(File::options().append(true).create(true).open(&output)?);
                generate(shell, &mut command, "chronicle", &mut writer);
            } else {
                generate(shell, &mut command, "chronicle", &mut TERMINAL.clone());
            }

            Ok(ExitCode::SUCCESS)
        }
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

    if result.is_err() {
        write_failure(&format!("Encountered an error which could not be recovered from, please report this at https://github.com/HazelTheWitch/chronicle/issues/new"))?;
    }

    result
}
