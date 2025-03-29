mod domain;
mod usecase;
mod repository;
mod controller;
mod presentation;
mod infrastructure;

use presentation::commands::{my_custom_command, get_setting_command, set_setting_command};
use infrastructure::db::{establish_connection_pool, run_migrations, AppState};
use tauri::Manager as _;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // AppHandle を渡して DB 接続プールを生成
            let pool = establish_connection_pool(app.handle());
            // マイグレーションの実行
            run_migrations(&pool);
            // アプリ全体で共有する状態として登録
            let state = AppState { pool };
            app.manage(state);

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![my_custom_command, get_setting_command, set_setting_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

