mod app_state;
mod controller;
mod domain;
mod infrastructure;
mod presentation;
mod repository;
mod usecase;

use std::sync::Arc;

use app_state::AppState;
use infrastructure::db::{establish_connection_pool, run_migrations};
use presentation::commands::{connect_to_grpc_server, get_graph_data, get_settings, set_settings};
use tauri::Manager as _;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // AppHandle を渡して DB 接続プールを生成
            let pool = establish_connection_pool(app.handle());
            // マイグレーションの実行
            run_migrations(&pool);
            // アプリ全体で共有する状態として登録
            let state = AppState {
                pool,
                grpc_connection: Arc::new(Mutex::new(None)),
            };
            app.manage(state);

            // if cfg!(debug_assertions) {
            //     app.handle().plugin(
            //         tauri_plugin_log::Builder::default()
            //             .level(log::LevelFilter::Info)
            //             .build(),
            //     )?;
            // }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            connect_to_grpc_server,
            get_graph_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
