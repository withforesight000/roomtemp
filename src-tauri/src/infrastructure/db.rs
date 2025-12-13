use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::path::Path;
use std::path::PathBuf;
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
    establish_connection_pool_at(&db_path)
}

/// Testable helper: establish a connection pool pointing at the given database path.
pub fn establish_connection_pool_at(db_path: &Path) -> DbPool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::RunQueryDsl;
    use diesel::r2d2::ConnectionManager;
    use diesel::r2d2::Pool;
    use diesel::sqlite::SqliteConnection;
    use tempfile::NamedTempFile;

    #[test]
    fn run_migrations_creates_tables() {
        let f = NamedTempFile::new().expect("temp file");
        let path = f.path().to_str().unwrap().to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(path.clone());
        let pool = Pool::builder().build(manager).expect("pool");

        run_migrations(&pool);

        let mut conn = pool.get().expect("conn");
        // check that settings table exists
        #[derive(diesel::QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            c: i64,
        }

        let row: CountRow = diesel::sql_query(
            "SELECT count(*) as c FROM sqlite_master WHERE type='table' AND name='settings'",
        )
        .get_result(&mut *conn)
        .expect("query");

        assert!(row.c >= 1);
    }

    #[test]
    fn establish_connection_pool_at_creates_pool_and_allows_migrations() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("roomtemp.db");

        let pool = establish_connection_pool_at(&db_path);

        // should be able to run migrations against it
        run_migrations(&pool);

        // verify the file exists
        assert!(db_path.exists());
    }
}
