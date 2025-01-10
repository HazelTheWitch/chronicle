use std::{
    env,
    fs::{self, File},
    io::BufWriter,
};

use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::ARGUMENTS;

pub fn initialize_logging() -> anyhow::Result<()> {
    fs::create_dir_all(
        ARGUMENTS
            .log_location
            .parent()
            .ok_or(anyhow::anyhow!("Could not get log folder location"))?,
    )?;

    let log_file = File::create(&ARGUMENTS.log_location)?;

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
            ARGUMENTS.log_level,
        ));

    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}
