use std::{
    iter,
    os::unix::process::ExitStatusExt,
    process::{ExitCode, ExitStatus},
    time::Duration,
};

use anyhow::Ok;
use chronicle::{
    models::{Tag, Work},
    tag::{DiscriminatedTag, TagExpression, TagPart},
};
use console::style;
use dialoguer::Select;
use indicatif::ProgressBar;
use itertools::Itertools;

use crate::{
    args::TagCommand,
    get_chronicle,
    table::{ColumnBehavior, Table},
    write_failure, write_success, PREFIX_STYLE, SPINNER_STYLE, TERMINAL,
};

pub async fn tag_command(command: &TagCommand) -> anyhow::Result<ExitCode> {
    match command {
        TagCommand::Apply { expression } => execute_tag_expression(expression).await,
        TagCommand::Info { tag } => display_tag_info(tag).await,
        TagCommand::Discriminate { tag, discriminator } => {
            discriminate_tag(tag, discriminator).await
        }
    }
}

pub async fn discriminate_tag(
    tag_name: &TagPart,
    discriminator: &TagPart,
) -> anyhow::Result<ExitCode> {
    let chronicle = get_chronicle().await;

    let mut tx = chronicle.begin().await?;

    let Some(mut tag) = Tag::try_get_discriminated(&mut tx, &tag_name.0, None).await? else {
        write_failure(&format!("Failure finding {tag_name}"))?;
        return Ok(ExitCode::FAILURE);
    };

    tag.discriminate(&mut tx, &discriminator.0).await?;

    tx.commit().await?;

    write_success(&format!(
        "Discriminated {tag_name} -> {tag_name}#{discriminator}"
    ))?;

    Ok(ExitCode::SUCCESS)
}

pub async fn display_tag_info(tag: &DiscriminatedTag) -> anyhow::Result<ExitCode> {
    let chronicle = get_chronicle().await;

    let mut tx = chronicle.begin().await?;

    let tags = match tag {
        DiscriminatedTag {
            name,
            discriminator: Some(discriminator),
        } => Tag::try_get_discriminated(&mut tx, name, Some(discriminator))
            .await?
            .into_iter()
            .collect(),
        DiscriminatedTag {
            name,
            discriminator: None,
        } => Tag::get(&mut tx, name).await?,
    };

    let tag = match tags.len() {
        0 => {
            write_failure(&format!("Error finding {tag}"))?;
            return Ok(ExitCode::FAILURE);
        }
        1 => &tags[0],
        _ => {
            let selected = Select::new()
                .with_prompt("Select which tag you want information for")
                .items(&tags)
                .interact()?;
            &tags[selected]
        }
    };

    let mut ancestors = tag.ancestors(&mut tx).await?;
    let mut descendants = tag.descendants(&mut tx).await?;

    ancestors.sort_by_key(|t| t.depth);
    descendants.sort_by_key(|t| t.depth);

    let max_length = usize::max(
        ancestors
            .iter()
            .map(|a| a.tag.name.len())
            .max()
            .unwrap_or_default(),
        descendants
            .iter()
            .map(|d| d.tag.name.len())
            .max()
            .unwrap_or_default(),
    ) + 1;

    let mut table = Table::new(
        &TERMINAL,
        vec![
            ColumnBehavior {
                size: 6,
                grow: false,
                min_size: 6,
            },
            ColumnBehavior {
                size: max_length,
                grow: true,
                min_size: max_length,
            },
        ],
        TERMINAL.size().1.into(),
    );

    table.push_right("DEPTH")?;
    table.push_left("TAG")?;

    for (depth, group) in descendants
        .into_iter()
        .rev()
        .filter(|a| a.depth != 0)
        .chunk_by(|a| a.depth)
        .into_iter()
    {
        let joined = group.into_iter().map(|t| t.tag.to_string()).join(", ");

        table.push_right(depth.to_string())?;
        table.push_left(joined)?;
    }

    table.push_right("0")?;
    table.push_left(&style(tag.to_string()).bold().to_string())?;

    for (depth, group) in ancestors
        .into_iter()
        .rev()
        .filter(|a| a.depth != 0)
        .chunk_by(|a| a.depth)
        .into_iter()
    {
        let joined = group.into_iter().map(|t| t.tag.to_string()).join(", ");

        table.push_right(depth.to_string())?;
        table.push_left(joined)?;
    }

    tx.commit().await?;

    Ok(ExitCode::SUCCESS)
}

pub async fn execute_tag_expression(expression: &TagExpression) -> anyhow::Result<ExitCode> {
    let chronicle = get_chronicle().await;

    let spinner = ProgressBar::new_spinner().with_style(SPINNER_STYLE.clone());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_prefix(PREFIX_STYLE.apply_to("Tagging").to_string());
    spinner.set_message(format!(
        "approximately {} connections",
        expression.approximate_connections()
    ));

    let mut tx = chronicle.begin().await?;
    let total = expression.execute(&mut tx).await?;

    tx.commit().await?;

    spinner.finish_and_clear();

    write_success(&format!("Tagged {total} connections"))?;

    Ok(ExitCode::SUCCESS)
}
