use std::fmt::{Debug, Formatter};
use std::time::Duration;
use r2d2::{CustomizeConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

struct RetryConnectionCustomizer {
    retries: i32
}

pub fn create_pool(database_url: &str) -> SqlitePool {
    let manager = SqliteConnectionManager::file(database_url);
    let pool = Pool::builder()
        .max_size(1) // Single connection
        .connection_timeout(Duration::from_secs(5))
        .connection_customizer(Box::new(RetryConnectionCustomizer { retries: 3 }))
        .build(manager)
        .expect("Failed to create pool.");

    setup_database(&pool);

    pool
}

/// Set up the database by creating the necessary tables
fn setup_database(pool: &SqlitePool) {
    let conn = get_conn(&pool);
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            topic TEXT NOT NULL,
            payload TEXT NOT NULL,
            received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).expect("Failed to create messages table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sensor_data (
            id INTEGER PRIMARY KEY,
            temperature DECIMAL(4,2) NOT NULL,
            humidity INTEGER NOT NULL,
            linkquality INTEGER NOT NULL,
            device_id TEXT NOT NULL,
            received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).expect("Failed to create sensor_data table");
}

pub fn get_conn(pool: &SqlitePool) -> SqlitePooledConnection {
    pool.get().expect("Failed to get connection.")
}

impl Debug for RetryConnectionCustomizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RetryConnectionCustomizer {{ retries: {} }}", self.retries)
    }
}

impl CustomizeConnection<rusqlite::Connection, rusqlite::Error> for RetryConnectionCustomizer {
    fn on_acquire(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        for _ in 0..self.retries {
            if conn.is_autocommit() {
                return Ok(());
            }
        }
        Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            Some("Failed to acquire connection after retries".to_string())
        ))
    }
}
