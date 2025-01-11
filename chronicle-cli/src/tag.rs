use std::{process::ExitCode, time::Duration};

use chronicle::{
    models::{Tag, Work},
    tag::TagExpression,
};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{get_chronicle, write_success, PREFIX_STYLE, PROGRESS_STYLE, SPINNER_STYLE};

pub async fn execute_tag_expression(expression: &TagExpression) -> anyhow::Result<ExitCode> {
    let chronicle = get_chronicle().await;

    let tx = chronicle.pool.begin().await?;

    let works = if let Some(query) = &expression.query {
        Work::search(&chronicle, query).await?
    } else {
        Vec::new()
    };

    if !works.is_empty() {
        let tags = &expression.hierarchy[0];

        let spinner = ProgressBar::new_spinner().with_style(SPINNER_STYLE.clone());
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_prefix(PREFIX_STYLE.apply_to("Tagging").to_string());
        spinner.set_message(format!("{} works with {tags:?}", works.len()));

        for work in &works {
            for tag in tags {
                work.tag(&chronicle, tag).await?;
            }
        }

        spinner.finish_and_clear();
    }

    if expression.hierarchy.len() > 1 {
        let bar = ProgressBar::new(expression.hierarchy.len() as u64 - 1)
            .with_style(PROGRESS_STYLE.clone());

        bar.set_prefix(PREFIX_STYLE.apply_to("Tagging").to_string());
        bar.set_message(format!(
            "{} levels of hierarchal tags",
            expression.hierarchy.len()
        ));

        for window in expression.hierarchy.windows(2) {
            let previous_tags = &window[0];
            let next_tags = &window[1];

            for tag in previous_tags {
                let tag = Tag::get_or_create(&chronicle, &tag).await?;

                for next in next_tags {
                    tag.tag(&chronicle, next).await?;
                }
            }

            bar.inc(1);
        }

        bar.finish_and_clear();
    }

    tx.commit().await?;

    let tagged = works.len()
        + expression.hierarchy[..expression.hierarchy.len() - 1]
            .iter()
            .fold(0, |total, level| total + level.len());

    write_success(&format!("Tagged {tagged} items"))?;

    Ok(ExitCode::SUCCESS)
}
