use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

use chronicle::{models::Work, record::Record, search::Query};
use console::style;
use dialoguer::Input;
use indicatif::{BinaryBytes, ProgressBar};
use uuid::Uuid;

use crate::{
    args::{WorkColumn, WorkCommand, WorkDetails, WorkDisplayOptions},
    get_chronicle,
    table::Table,
    utils::format_hash,
    write_failure, write_success, PREFIX_STYLE, SPINNER_STYLE, TERMINAL,
};

pub async fn work_command(command: &WorkCommand) -> anyhow::Result<ExitCode> {
    match command {
        WorkCommand::Add { path, details } => work_add(path, details).await,
        WorkCommand::Import { source, details } => work_import(source, details).await,
        WorkCommand::Search {
            display_options,
            query,
        } => {
            let query = if let Some(query) = query {
                query
            } else {
                &Input::<Query>::new()
                    .with_prompt("Enter your query")
                    .interact_text()?
            };

            work_search(query, display_options).await
        }
        WorkCommand::List { display_options } => work_list(display_options).await,
    }
}

pub async fn work_import(source: &url::Url, details: &WorkDetails) -> anyhow::Result<ExitCode> {
    let spinner = ProgressBar::new_spinner();

    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_style(SPINNER_STYLE.clone());

    spinner.set_prefix(PREFIX_STYLE.apply_to("Importing").to_string());
    spinner.set_message(source.to_string());

    let works =
        Work::import_works_from_url(get_chronicle().await, source, Some(&details.clone().into()))
            .await;

    spinner.finish_and_clear();

    match works {
        Ok(works) => {
            if works.is_empty() {
                write_failure("Failed to import any works")?;
            } else {
                write_success(&format!(
                    "Successfully imported {count} {}",
                    if works.len() == 1 { "work" } else { "works" },
                    count = style(works.len().to_string()).bold()
                ))?;
            }

            Ok(ExitCode::SUCCESS)
        }
        Err(err) => {
            write_failure(&format!("Failed {err}"))?;

            Ok(ExitCode::FAILURE)
        }
    }
}

pub fn display_work_header(table: &mut Table, options: &WorkDisplayOptions) -> anyhow::Result<()> {
    for column in &options.columns {
        table.push_cell(style(column.to_string()).bold())?;
    }

    Ok(())
}

pub fn display_work(
    table: &mut Table,
    work: &Work,
    options: &WorkDisplayOptions,
) -> anyhow::Result<()> {
    for column in &options.columns {
        match column {
            WorkColumn::Id => {
                table.push_cell(work.work_id)?;
            }
            WorkColumn::Path => {
                table.push_cell(&work.path)?;
            }
            WorkColumn::Hash => {
                table.push_cell(format_hash(work.hash))?;
            }
            WorkColumn::Title => {
                table.push_cell(work.title.as_ref().map(|t| t.clone()).unwrap_or_default())?;
            }
            WorkColumn::AuthorId => {
                table.push_cell(work.author_id.map(|id| id.to_string()).unwrap_or_default())?;
            }
            WorkColumn::Caption => {
                table.push_cell(work.caption.as_ref().map(|c| c.clone()).unwrap_or_default())?;
            }
            WorkColumn::Url => {
                table.push_cell(work.url.as_ref().map(|c| c.clone()).unwrap_or_default())?;
            }
            WorkColumn::Size => {
                table.push_cell(&format!("{}", BinaryBytes(work.size)))?;
            }
        }
    }

    Ok(())
}

pub fn print_works(works: &Vec<Work>, options: &WorkDisplayOptions) -> anyhow::Result<()> {
    let mut console = TERMINAL.clone();
    let width = console.size().1 as usize;

    let mut table = Table::new(
        &mut console,
        options.columns.iter().map(WorkColumn::behavior).collect(),
        width,
    );

    display_work_header(&mut table, options)?;

    for work in works {
        display_work(&mut table, &work, options)?;
    }

    Ok(())
}

pub async fn work_list(options: &WorkDisplayOptions) -> anyhow::Result<ExitCode> {
    let works = Work::get_all(get_chronicle().await).await?;

    print_works(&works, options)?;

    Ok(ExitCode::SUCCESS)
}

pub async fn work_search(query: &Query, options: &WorkDisplayOptions) -> anyhow::Result<ExitCode> {
    let works = Work::search(get_chronicle().await, query).await?;

    print_works(&works, options)?;

    Ok(ExitCode::SUCCESS)
}

pub async fn work_add(path: impl AsRef<Path>, details: &WorkDetails) -> anyhow::Result<ExitCode> {
    let original_path = path.as_ref();

    let file_name = if let Some(extension) = original_path.extension() {
        format!("{}.{}", Uuid::new_v4(), extension.to_string_lossy())
    } else {
        Uuid::new_v4().to_string()
    };

    let chronicle = get_chronicle().await;

    let new_full_path = chronicle.config.data_path.join(&file_name);

    fs::copy(&original_path, &new_full_path)?;

    let record = Record::from_path(
        &chronicle,
        PathBuf::from(&file_name),
        details.clone().into(),
    )?;

    let work = Work::create_from_record(&chronicle, &record).await?;

    TERMINAL.write_line(&format!(
        "{}: chronicled {} as {}",
        style("Success").bold().green(),
        style(&original_path.to_string_lossy()).dim(),
        style(&work.work_id.to_string()).bold(),
    ))?;

    Ok(ExitCode::SUCCESS)
}
