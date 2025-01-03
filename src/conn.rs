use r2d2::{CustomizeConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, OpenFlags};
use std::fmt::{Debug, Formatter};
use std::time::Duration;

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

struct RetryConnectionCustomizer {
    retries: i32,
}

pub fn create_pool(database_url: &str) -> Result<SqlitePool, &'static str> {
    if is_database_locked(database_url) {
        return Err("Database is locked by another process.");
    }

    let manager = SqliteConnectionManager::file(database_url);
    let pool = Pool::builder()
        .max_size(1) // Single connection
        .connection_timeout(Duration::from_secs(5))
        .connection_customizer(Box::new(RetryConnectionCustomizer { retries: 3 }))
        .build(manager)
        .expect("Failed to create pool.");

    setup_database(&pool);

    Ok(pool)
}

fn is_database_locked(database_url: &str) -> bool {
    match Connection::open_with_flags(
        database_url,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    ) {
        Ok(_) => false,
        Err(_) => true,
    }
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
    )
    .expect("Failed to create messages table");

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
    )
    .expect("Failed to create sensor_data table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS topic_configuration (
        id INTEGER PRIMARY KEY,
        topic_name TEXT NOT NULL,
        status_type TEXT NOT NULL
    )",
        [],
    )
    .expect("Failed to create topic_configuration table");
}

pub fn get_conn(pool: &SqlitePool) -> SqlitePooledConnection {
    pool.get().expect("Failed to get connection.")
}

impl Debug for RetryConnectionCustomizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RetryConnectionCustomizer {{ retries: {} }}",
            self.retries
        )
    }
}

impl CustomizeConnection<Connection, rusqlite::Error> for RetryConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        for _ in 0..self.retries {
            if conn.is_autocommit() {
                return Ok(());
            }
        }
        Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            Some("Failed to acquire connection after retries".to_string()),
        ))
    }
}

#[cfg(test)]
pub fn get_test_pool() -> SqlitePool {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder().build(manager).unwrap();

    setup_database(&pool);

    pool
}
