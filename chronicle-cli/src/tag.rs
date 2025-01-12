use std::{process::ExitCode, time::Duration};

use chronicle::{
    models::{Tag, Work},
    tag::TagExpression,
};
use indicatif::ProgressBar;

use crate::{get_chronicle, write_success, PREFIX_STYLE, SPINNER_STYLE};

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
