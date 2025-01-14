pub mod commands;
pub mod create;

use chronicle::{Chronicle, Config};
use tauri::{async_runtime::block_on, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let chronicle_config = Config {
                database_path: data_dir.join("database.db"),
                data_path: data_dir.join("works"),
            };
            let chronicle = block_on(Chronicle::from_config(chronicle_config))?;

            app.manage(chronicle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::works::work_query,
            commands::works::import_work_create,
            commands::tags::parse_tag,
            commands::works::create_work,
            commands::works::get_work_edit_by_id
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
