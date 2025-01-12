use std::{
    fs::File,
    future::Future,
    io::{stdin, BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
    time::Duration,
};

use chronicle::{
    models::Work,
    record::{Record, RecordDetails},
    tag::TagExpression,
};
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar};
use tokio::{sync::mpsc, task::JoinSet, time::sleep};
use url::Url;
use uuid::Uuid;

use crate::{
    args::{BulkCommand, WorkDetails},
    get_chronicle, write_failure, write_success, ERROR_STYLE, PREFIX_STYLE, SPINNER_STYLE,
    TERMINAL,
};

pub async fn bulk_operation<
    T: ToString + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    O: Send + 'static,
    Fut: Send + Future<Output = O>,
>(
    inputs: Vec<T>,
    context: C,
    operation: impl Fn(ProgressBar, T, C) -> Fut + Copy + Send + 'static,
    verb: &'static str,
    tasks: usize,
) -> anyhow::Result<Vec<O>> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (t_data, r_data) = async_channel::unbounded::<T>();

    let count = inputs.len();

    let mut set = JoinSet::new();

    let multi_bar = MultiProgress::new();

    let mut in_progress = Vec::<ProgressBar>::with_capacity(inputs.len());

    for _ in 0..tasks {
        let tx = tx.clone();
        let multi_bar = multi_bar.clone();
        let r_data = r_data.clone();

        let context = context.clone();

        set.spawn(async move {
            let mut results = Vec::with_capacity(count / tasks + 1);
            while let Ok(input) = r_data.recv().await {
                let bar =
                    multi_bar.add(ProgressBar::new_spinner().with_style(SPINNER_STYLE.clone()));

                bar.set_prefix(PREFIX_STYLE.apply_to(verb).to_string());
                let input_string = input.to_string();
                bar.set_message(input_string.clone());

                tx.send(bar.clone())?;

                let result = operation(bar.clone(), input, context.clone()).await;

                results.push(result);

                bar.finish_and_clear();
            }

            Ok(results)
        });
    }

    for i in inputs {
        t_data.send(i).await?;
    }

    drop(tx);
    drop(r_data);
    drop(t_data);

    while !rx.is_closed() {
        in_progress.retain(|pb| !pb.is_finished());
        in_progress.iter().for_each(ProgressBar::tick);

        tokio::select! {
            Some(pb) = rx.recv() => {
                in_progress.push(pb);
            },
            _ = sleep(Duration::from_millis(100)) => {},
        }
    }

    Ok(set
        .join_all()
        .await
        .into_iter()
        .map(|v: anyhow::Result<Vec<_>>| v.expect("task failed").into_iter())
        .flatten()
        .collect::<Vec<_>>())
}

#[derive(Parser)]
struct BulkImportCommand {
    pub url: Url,
    #[command(flatten)]
    pub details: WorkDetails,
}

pub async fn bulk(tasks: usize, command: &BulkCommand) -> anyhow::Result<ExitCode> {
    match command {
        BulkCommand::Import { path, details } => {
            let reader = BufReader::new(File::open(&path)?);

            let urls = reader.lines();

            let record_details: RecordDetails = details.clone().into();

            let works = bulk_operation(
                urls.flatten().collect(),
                record_details,
                |bar: ProgressBar, line: String, details: RecordDetails| async move {
                    let chronicle = get_chronicle().await;

                    let Ok(mut tx) = chronicle.begin().await else {
                        bar.println(
                            &ERROR_STYLE
                                .apply_to("Could not start transaction")
                                .to_string(),
                        );
                        return Vec::new();
                    };

                    match Url::parse(&line) {
                        Ok(url) => {
                            match Work::import_works_from_url(
                                chronicle,
                                &mut tx,
                                &url,
                                Some(&details),
                            )
                            .await
                            {
                                Ok(works) => {
                                    if let Err(err) = tx.commit().await {
                                        bar.println(
                                            &ERROR_STYLE
                                                .apply_to(&format!(
                                                    "Could not commit transaction: {err}"
                                                ))
                                                .to_string(),
                                        );
                                    }
                                    works
                                }
                                Err(err) => {
                                    bar.println(
                                        ERROR_STYLE
                                            .apply_to(format!("Unable to import '{line}': {err}"))
                                            .to_string(),
                                    );
                                    Vec::new()
                                }
                            }
                        }
                        Err(_) => {
                            bar.println(
                                ERROR_STYLE
                                    .apply_to(format!("Invalid url '{line}'"))
                                    .to_string(),
                            );
                            Vec::new()
                        }
                    }
                },
                "Importing",
                tasks,
            )
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

            write_success(&format!("Imported {} works", works.len()))?;
        }
        BulkCommand::Add { path, details } => {
            let reader = BufReader::new(File::open(&path)?);

            let paths = reader.lines();

            let record_details: RecordDetails = details.clone().into();

            let works = bulk_operation(
                paths.flatten().collect(),
                record_details,
                |bar: ProgressBar, path: String, details: RecordDetails| async move {
                    let chronicle = get_chronicle().await;

                    let original_path = PathBuf::from(path);

                    let file_name = if let Some(extension) = original_path.extension() {
                        format!("{}.{}", Uuid::new_v4(), extension.to_string_lossy())
                    } else {
                        Uuid::new_v4().to_string()
                    };

                    let new_full_path = chronicle.config.data_path.join(&file_name);

                    if let Err(err) = tokio::fs::copy(&original_path, &new_full_path).await {
                        bar.println(
                            &ERROR_STYLE
                                .apply_to(format!(
                                    "Could not copy {original_path:?} -> {new_full_path:?}: {err}"
                                ))
                                .to_string(),
                        );
                        return None;
                    }

                    let record = match Record::from_path(
                        &chronicle,
                        PathBuf::from(&file_name),
                        details.clone(),
                    ) {
                        Ok(record) => record,
                        Err(err) => {
                            bar.println(
                                &ERROR_STYLE
                                    .apply_to(format!("Could not create record from file: {err}"))
                                    .to_string(),
                            );
                            return None;
                        }
                    };

                    let Ok(mut tx) = get_chronicle().await.begin().await else {
                        bar.println(
                            &ERROR_STYLE
                                .apply_to("Could not start transaction")
                                .to_string(),
                        );
                        return None;
                    };

                    let work = match Work::create_from_record(&mut tx, &record).await {
                        Ok(work) => Some(work),
                        Err(err) => {
                            bar.println(
                                &ERROR_STYLE
                                    .apply_to(format!("Could not create work: {err}"))
                                    .to_string(),
                            );
                            return None;
                        }
                    };

                    if let Err(err) = tx.commit().await {
                        bar.println(
                            &ERROR_STYLE
                                .apply_to(&format!("Could not commit transaction: {err}"))
                                .to_string(),
                        );
                    }

                    work
                },
                "Adding",
                tasks,
            )
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<Work>>();

            write_success(&format!("Added {} works", works.len(),))?;
        }
        BulkCommand::Tag { path } => {
            let reader = BufReader::new(File::open(&path)?);

            let paths = reader.lines();

            let tagged = bulk_operation(
                paths.flatten().collect(),
                (),
                |bar: ProgressBar, tag_expression: String, _: ()| async move {
                    let expression = match TagExpression::from_str(&tag_expression) {
                        Ok(expression) => expression,
                        Err(err) => {
                            bar.println(
                                &ERROR_STYLE
                                    .apply_to(format!("Could not parse '{tag_expression}': {err}"))
                                    .to_string(),
                            );

                            return 0;
                        }
                    };

                    let Ok(mut tx) = get_chronicle().await.begin().await else {
                        bar.println(
                            &ERROR_STYLE
                                .apply_to("Could not start transaction")
                                .to_string(),
                        );
                        return 0;
                    };

                    let total = match expression.execute(&mut tx).await {
                        Ok(total) => total,
                        Err(err) => {
                            bar.println(
                                &ERROR_STYLE
                                    .apply_to(format!(
                                        "Could not execute '{tag_expression}': {err}"
                                    ))
                                    .to_string(),
                            );

                            return 0;
                        }
                    };

                    if let Err(err) = tx.commit().await {
                        bar.println(
                            &ERROR_STYLE
                                .apply_to(&format!("Could not commit transaction: {err}"))
                                .to_string(),
                        );
                    }

                    total
                },
                "Tagging",
                tasks,
            )
            .await?
            .into_iter()
            .sum::<usize>();

            write_success(&format!("Tagged {tagged} new connections"))?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
