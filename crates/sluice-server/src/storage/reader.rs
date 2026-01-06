//! Read connection pool for concurrent subscriptions.
//!
//! Uses r2d2 with r2d2_sqlite for pooled read access.
//! SQLite WAL mode allows concurrent readers.

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;
use std::path::Path;
use thiserror::Error;

use super::schema::apply_reader_pragmas;

/// Error type for reader pool operations.
#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("Failed to create connection pool: {0}")]
    PoolCreation(#[from] r2d2::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

/// Read connection pool for subscription queries.
///
/// Provides pooled read-only connections for concurrent access.
/// SQLite WAL mode allows multiple concurrent readers.
#[derive(Clone)]
pub struct ReaderPool {
    pool: Pool<SqliteConnectionManager>,
}

impl ReaderPool {
    /// Create a new reader pool for the given database path.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file
    /// * `max_size` - Maximum number of connections in the pool
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot be created.
    pub fn new<P: AsRef<Path>>(db_path: P, max_size: u32) -> Result<Self, ReaderError> {
        let manager = SqliteConnectionManager::file(db_path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX);

        let pool = Pool::builder()
            .max_size(max_size)
            .connection_customizer(Box::new(ReaderConnectionCustomizer))
            .build(manager)?;

        Ok(Self { pool })
    }

    /// Get a connection from the pool.
    pub fn get(&self) -> Result<PooledConnection<SqliteConnectionManager>, ReaderError> {
        Ok(self.pool.get()?)
    }

    /// Get the current pool state for monitoring.
    pub fn state(&self) -> r2d2::State {
        self.pool.state()
    }

    /// List topics known to the server.
    ///
    /// Returns topics in lexicographic order by name for stable UI ordering.
    pub fn list_topics(&self) -> Result<Vec<(String, i64)>, ReaderError> {
        let conn = self.get()?;
        let mut stmt = conn.prepare("SELECT name, created_at FROM topics ORDER BY name ASC")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

/// Connection customizer that applies reader pragmas.
#[derive(Debug)]
struct ReaderConnectionCustomizer;

impl r2d2::CustomizeConnection<rusqlite::Connection, rusqlite::Error>
    for ReaderConnectionCustomizer
{
    fn on_acquire(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        apply_reader_pragmas(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::schema::{apply_pragmas, initialize_schema, insert_or_get_topic};
    use rusqlite::Connection;
    use tempfile::TempDir;

    #[test]
    fn test_reader_pool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create the database first with write connection
        {
            let conn = Connection::open(&db_path).unwrap();
            apply_pragmas(&conn).unwrap();
            initialize_schema(&conn).unwrap();
            insert_or_get_topic(&conn, "test-topic", 1234567890000).unwrap();
        }

        // Create reader pool
        let pool = ReaderPool::new(&db_path, 5).unwrap();

        // Get a connection
        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM topics", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
