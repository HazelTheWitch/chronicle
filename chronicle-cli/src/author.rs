use std::process::ExitCode;

use chronicle::{
    author::{self, AuthorQuery},
    models::{Author, Work},
};
use console::style;
use dialoguer::Select;
use url::Url;

use crate::{
    args::{AuthorColumn, AuthorCommand, AuthorDisplayOptions},
    get_chronicle,
    table::Table,
    write_failure, write_success, TERMINAL,
};

pub async fn author_command(command: &AuthorCommand) -> anyhow::Result<ExitCode> {
    match command {
        AuthorCommand::List { display_options } => list_authors(display_options).await,
        AuthorCommand::Alias { query, alias } => alias_author(query, alias).await,
        AuthorCommand::AddUrl { query, url } => add_url_author(query, url).await,
    }
}

async fn add_url_author(query: &AuthorQuery, url: &Url) -> anyhow::Result<ExitCode> {
    let chronicle = get_chronicle().await;
    let mut tx = chronicle.begin().await?;
    let author = &Author::get(&mut tx, query).await?[0];

    author.add_url(&mut tx, url).await?;

    tx.commit().await?;

    write_success(&format!("Added url {url} -> {}", author.author_id))?;

    Ok(ExitCode::SUCCESS)
}

async fn alias_author(query: &AuthorQuery, alias: &str) -> anyhow::Result<ExitCode> {
    let chronicle = get_chronicle().await;
    let mut tx = chronicle.begin().await?;
    let authors = &Author::get(&mut tx, query).await?;

    let author = match authors.len() {
        0 => {
            write_failure("Failed to find author")?;
            return Ok(ExitCode::FAILURE);
        }
        1 => &authors[0],
        _ => {
            &authors[Select::new()
                .items(
                    &authors
                        .iter()
                        .map(|author| author.author_id.to_string())
                        .collect::<Vec<_>>(),
                )
                .with_prompt("Select the author id you wish to alias")
                .interact()?]
        }
    };

    author.add_alias(&mut tx, alias).await?;

    tx.commit().await?;

    write_success(&format!("Aliased {alias} -> {}", author.author_id))?;

    Ok(ExitCode::SUCCESS)
}

pub fn display_author_header(
    table: &mut Table,
    options: &AuthorDisplayOptions,
) -> anyhow::Result<()> {
    for column in &options.columns {
        table.push_left(style(column.to_string()).bold())?;
    }

    Ok(())
}
async fn list_authors(options: &AuthorDisplayOptions) -> anyhow::Result<ExitCode> {
    let chronicle = get_chronicle().await;

    let mut tx = chronicle.begin().await?;

    let mut console = TERMINAL.clone();
    let width = console.size().1 as usize;

    let mut table = Table::new(
        &mut console,
        options.columns.iter().map(AuthorColumn::behavior).collect(),
        width,
    );

    display_author_header(&mut table, options)?;

    let authors = Author::get_all(&mut tx).await?;

    for author in authors {
        let aliases = author.get_author_names(&mut tx).await?;
        let urls = author.get_author_urls(&mut tx).await?;

        for i in 0..usize::max(aliases.len(), urls.len()).max(1) {
            for column in options.columns.iter() {
                match column {
                    AuthorColumn::Id => {
                        if i == 0 {
                            table.push_left(author.author_id)?;
                        } else {
                            table.push_left("")?;
                        }
                    }
                    AuthorColumn::Aliases => {
                        if let Some(alias) = aliases.get(i) {
                            table.push_left(&alias.name)?;
                        } else {
                            table.push_left("")?;
                        }
                    }
                    AuthorColumn::Urls => {
                        if let Some(url) = urls.get(i) {
                            table.push_left(&url.url)?;
                        } else {
                            table.push_left("")?;
                        }
                    }
                }
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}
