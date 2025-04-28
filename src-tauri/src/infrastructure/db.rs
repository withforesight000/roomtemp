use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use std::path::PathBuf;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tauri::{AppHandle, Manager as _};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// アプリケーション全体で共有する状態
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
}

// Tauri の tauri::Builder::default().setup() に渡すクロージャ内で実際には使われている
/// embed_migrations! マクロで、migrations 配下のマイグレーションを組み込みます。
#[allow(dead_code)]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migration");

// Tauri の tauri::generate_handler! マクロ経由で実際には使われている
/// DB 接続プールの作成（app_data_dir を利用して安全な場所に DB ファイルを配置）
#[allow(dead_code)]
pub fn establish_connection_pool(app_handle: &AppHandle) -> DbPool {
    // AppHandle 経由で PathResolver の app_data_dir() を取得
    let app_data: PathBuf = app_handle
        .path()
        .app_data_dir()
        .expect("failed to get app data directory");
    std::fs::create_dir_all(&app_data).expect("failed to create app data directory");

    let db_path = app_data.join("settings.db");
    println!("DB path: {:?}", db_path);
    let database_url = db_path.to_str().expect("invalid db path");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool")
}

// Tauri の tauri::generate_handler! マクロ経由で実際には使われている
/// マイグレーションの実行
#[allow(dead_code)]
pub fn run_migrations(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}
