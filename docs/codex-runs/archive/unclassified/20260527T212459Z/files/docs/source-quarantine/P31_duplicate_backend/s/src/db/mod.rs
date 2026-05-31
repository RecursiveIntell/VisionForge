pub mod checkpoints;
pub mod comparisons;
pub mod images;
pub mod migrations;
pub mod queue;
pub mod seeds;
pub mod tags;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

pub fn open_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )
    .context("Failed to set database pragmas")?;

    migrations::run(&conn).context("Failed to run database migrations")?;

    Ok(conn)
}

#[cfg(test)]
pub fn open_memory_database() -> Result<Connection> {
    let conn = Connection::open_in_memory().context("Failed to open in-memory database")?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .context("Failed to set foreign keys pragma")?;

    migrations::run(&conn).context("Failed to run database migrations")?;

    Ok(conn)
}
