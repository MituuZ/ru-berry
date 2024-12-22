use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

pub fn create_pool(database_url: &str) -> SqlitePool {
    let manager = SqliteConnectionManager::file(database_url);
    Pool::builder()
        .max_size(1) // Single connection
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn get_conn(pool: &SqlitePool) -> SqlitePooledConnection {
    pool.get().expect("Failed to get connection.")
}