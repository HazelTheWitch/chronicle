use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicI32, Ordering},
};

use tempfile::{tempdir, TempDir};

use crate::{record::Record, Chronicle, Config};

#[allow(dead_code)]
pub struct TempChronicle {
    dir: TempDir,
    chronicle: Chronicle,
}

impl AsRef<Chronicle> for TempChronicle {
    fn as_ref(&self) -> &Chronicle {
        &self.chronicle
    }
}

pub async fn temp_chronicle() -> TempChronicle {
    let dir = tempdir().expect("could not get temp directory");

    let data_path = dir.path().join("works");
    let database_path = dir.path().join("database.db");
    let config_path = dir.path().join("config.toml");

    fs::write(
        &config_path,
        &toml::to_string(&Config {
            database_path,
            data_path,
        })
        .expect("could not serialize config"),
    )
    .expect("could not write config file");

    let chronicle = Chronicle::from_path(config_path)
        .await
        .expect("could not initialize temp chronicle");

    TempChronicle { dir, chronicle }
}

impl Record {
    pub fn dummy() -> Self {
        static WORK_NUMBER: AtomicI32 = AtomicI32::new(0);

        Self {
            path: PathBuf::from(format!("{}", WORK_NUMBER.fetch_add(1, Ordering::Relaxed))),
            hash: 0,
            details: Default::default(),
        }
    }
}

mod tests {
    use std::iter::{repeat_n, repeat_with};

    use crate::record::Record;

    #[tokio::test]
    async fn test_work_crud() -> Result<(), crate::Error> {
        let records: Vec<Record> = repeat_with(Record::dummy).take(10).collect();
        Ok(())
    }
}
