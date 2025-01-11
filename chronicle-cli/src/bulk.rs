use std::{
    fs::File,
    future::Future,
    io::{stdin, BufRead, BufReader, Read},
    path::Path,
    process::ExitCode,
    time::Duration,
};

use chronicle::{models::Work, record::RecordDetails};
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar};
use tokio::{sync::mpsc, task::JoinSet, time::sleep};
use url::Url;

use crate::{
    args::{BulkCommand, WorkDetails},
    get_chronicle, write_success, ERROR_STYLE, PREFIX_STYLE, SPINNER_STYLE, TERMINAL,
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

fn file_or_stdin(path: Option<&Path>) -> anyhow::Result<Box<dyn Read>> {
    match path {
        Some(path) => Ok(Box::new(File::open(path)?)),
        None => Ok(Box::new(stdin())),
    }
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
            let reader = BufReader::new(file_or_stdin(path.as_deref())?);

            let urls = reader.lines();

            let record_details: RecordDetails = details.clone().into();

            let works = bulk_operation(
                urls.flatten().collect(),
                record_details,
                |bar: ProgressBar, line: String, details: RecordDetails| async move {
                    match Url::parse(&line) {
                        Ok(url) => {
                            match Work::import_works_from_url(
                                get_chronicle().await,
                                &url,
                                Some(&details),
                            )
                            .await
                            {
                                Ok(works) => works,
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
        BulkCommand::Add { path } => todo!(),
        BulkCommand::Tag { path } => todo!(),
    }

    Ok(ExitCode::SUCCESS)
}
