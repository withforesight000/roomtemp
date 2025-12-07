use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use std::path::PathBuf;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tauri::{AppHandle, Manager as _};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migration");

pub fn establish_connection_pool(app_handle: &AppHandle) -> DbPool {
    // AppHandle 経由で PathResolver の app_data_dir() を取得
    let app_data: PathBuf = app_handle
        .path()
        .app_data_dir()
        .expect("failed to get app data directory");
    std::fs::create_dir_all(&app_data).expect("failed to create app data directory");

    let db_path = app_data.join("roomtemp.db");
    println!("DB path: {db_path:?}");
    let database_url = db_path.to_str().expect("invalid db path");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool")
}

pub fn run_migrations(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}
