use anyhow::{Context, Result};
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)
        .context("Failed to execute database schema")?;
    Ok(())
}

const SCHEMA: &str = r#"
-- ============================================
-- Core Gallery
-- ============================================

CREATE TABLE IF NOT EXISTS images (
    id              TEXT PRIMARY KEY,
    filename        TEXT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    positive_prompt TEXT,
    negative_prompt TEXT,
    original_idea   TEXT,
    checkpoint      TEXT,
    width           INTEGER,
    height          INTEGER,
    steps           INTEGER,
    cfg_scale       REAL,
    sampler         TEXT,
    scheduler       TEXT,
    seed            INTEGER,
    pipeline_log    TEXT,
    selected_concept INTEGER,
    auto_approved   BOOLEAN DEFAULT FALSE,
    caption         TEXT,
    caption_edited  BOOLEAN DEFAULT FALSE,
    rating          INTEGER,
    favorite        BOOLEAN DEFAULT FALSE,
    deleted         BOOLEAN DEFAULT FALSE,
    user_note       TEXT
);

-- ============================================
-- Tags (shared across images and seeds)
-- ============================================

CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS image_tags (
    image_id    TEXT REFERENCES images(id) ON DELETE CASCADE,
    tag_id      INTEGER REFERENCES tags(id),
    source      TEXT CHECK(source IN ('ai', 'user')),
    confidence  REAL,
    PRIMARY KEY (image_id, tag_id)
);

-- ============================================
-- Seed Library
-- ============================================

CREATE TABLE IF NOT EXISTS seeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_value      INTEGER NOT NULL,
    comment         TEXT NOT NULL,
    checkpoint      TEXT,
    sample_image_id TEXT REFERENCES images(id),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS seed_tags (
    seed_id  INTEGER REFERENCES seeds(id) ON DELETE CASCADE,
    tag_id   INTEGER REFERENCES tags(id),
    PRIMARY KEY (seed_id, tag_id)
);

CREATE TABLE IF NOT EXISTS seed_checkpoint_notes (
    seed_id         INTEGER REFERENCES seeds(id) ON DELETE CASCADE,
    checkpoint      TEXT NOT NULL,
    note            TEXT NOT NULL,
    sample_image_id TEXT REFERENCES images(id),
    PRIMARY KEY (seed_id, checkpoint)
);

-- ============================================
-- Checkpoint Knowledge Database
-- ============================================

CREATE TABLE IF NOT EXISTS checkpoints (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    filename            TEXT UNIQUE NOT NULL,
    display_name        TEXT,
    base_model          TEXT,
    created_at          DATETIME DEFAULT CURRENT_TIMESTAMP,
    strengths           TEXT,
    weaknesses          TEXT,
    preferred_cfg       REAL,
    cfg_range_low       REAL,
    cfg_range_high      REAL,
    preferred_sampler   TEXT,
    preferred_scheduler TEXT,
    optimal_resolution  TEXT,
    notes               TEXT
);

CREATE TABLE IF NOT EXISTS checkpoint_prompt_terms (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    checkpoint_id   INTEGER REFERENCES checkpoints(id) ON DELETE CASCADE,
    term            TEXT NOT NULL,
    effect          TEXT NOT NULL,
    strength        TEXT CHECK(strength IN ('strong', 'moderate', 'weak', 'broken')),
    example_image_id TEXT REFERENCES images(id),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS checkpoint_observations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    checkpoint_id   INTEGER REFERENCES checkpoints(id) ON DELETE CASCADE,
    observation     TEXT NOT NULL,
    source          TEXT CHECK(source IN ('user', 'ab_comparison', 'pipeline_note', 'auto_rating')),
    comparison_id   TEXT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- A/B Comparisons
-- ============================================

CREATE TABLE IF NOT EXISTS comparisons (
    id              TEXT PRIMARY KEY,
    image_a_id      TEXT REFERENCES images(id),
    image_b_id      TEXT REFERENCES images(id),
    variable_changed TEXT NOT NULL,
    note            TEXT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- Smart Queue (persistent)
-- ============================================

CREATE TABLE IF NOT EXISTS queue_jobs (
    id              TEXT PRIMARY KEY,
    priority        INTEGER DEFAULT 1,
    status          TEXT CHECK(status IN ('pending', 'generating', 'completed', 'failed', 'cancelled')),
    positive_prompt TEXT NOT NULL,
    negative_prompt TEXT NOT NULL,
    settings_json   TEXT NOT NULL,
    pipeline_log    TEXT,
    original_idea   TEXT,
    linked_comparison_id TEXT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at      DATETIME,
    completed_at    DATETIME,
    result_image_id TEXT REFERENCES images(id)
);

-- ============================================
-- Indexes
-- ============================================

CREATE INDEX IF NOT EXISTS idx_images_checkpoint ON images(checkpoint);
CREATE INDEX IF NOT EXISTS idx_images_seed ON images(seed);
CREATE INDEX IF NOT EXISTS idx_images_created ON images(created_at);
CREATE INDEX IF NOT EXISTS idx_images_rating ON images(rating);
CREATE INDEX IF NOT EXISTS idx_images_deleted ON images(deleted);
CREATE INDEX IF NOT EXISTS idx_checkpoint_terms_checkpoint ON checkpoint_prompt_terms(checkpoint_id);
CREATE INDEX IF NOT EXISTS idx_queue_status ON queue_jobs(status, priority);
CREATE INDEX IF NOT EXISTS idx_seeds_value ON seeds(seed_value);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_run_successfully() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run(&conn).unwrap();
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run(&conn).unwrap();
        run(&conn).unwrap();
    }

    #[test]
    fn test_all_tables_created() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let expected = vec![
            "checkpoint_observations",
            "checkpoint_prompt_terms",
            "checkpoints",
            "comparisons",
            "image_tags",
            "images",
            "queue_jobs",
            "seed_checkpoint_notes",
            "seed_tags",
            "seeds",
            "tags",
        ];

        for table in &expected {
            assert!(
                tables.contains(&table.to_string()),
                "Missing table: {}",
                table
            );
        }
    }
}
