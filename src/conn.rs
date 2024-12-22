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
    Pool::builder()
        .max_size(1) // Single connection
        .connection_timeout(Duration::from_secs(5))
        .connection_customizer(Box::new(RetryConnectionCustomizer { retries: 3 }))
        .build(manager)
        .expect("Failed to create pool.")
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
