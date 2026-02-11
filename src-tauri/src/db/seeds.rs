use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::types::seeds::{SeedCheckpointNote, SeedEntry, SeedFilter};

pub fn insert_seed(conn: &Connection, seed: &SeedEntry) -> Result<i64> {
    conn.execute(
        "INSERT INTO seeds (seed_value, comment, checkpoint, sample_image_id)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            seed.seed_value,
            seed.comment,
            seed.checkpoint,
            seed.sample_image_id,
        ],
    )
    .context("Failed to insert seed")?;

    Ok(conn.last_insert_rowid())
}

pub fn get_seed(conn: &Connection, id: i64) -> Result<Option<SeedEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, seed_value, comment, checkpoint, sample_image_id, created_at
             FROM seeds WHERE id = ?1",
        )
        .context("Failed to prepare get_seed query")?;

    let mut rows = stmt
        .query_map(params![id], row_to_seed)
        .context("Failed to execute get_seed query")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("Failed to read seed row")?)),
        None => Ok(None),
    }
}

#[allow(unused_assignments)]
pub fn list_seeds(conn: &Connection, filter: &SeedFilter) -> Result<Vec<SeedEntry>> {
    let mut conditions = vec!["1=1".to_string()];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;

    if let Some(ref checkpoint) = filter.checkpoint {
        conditions.push(format!("s.checkpoint = ?{}", param_idx));
        param_values.push(Box::new(checkpoint.clone()));
        param_idx += 1;
    }

    if let Some(ref search) = filter.search {
        let pattern = format!("%{}%", search);
        conditions.push(format!("s.comment LIKE ?{}", param_idx));
        param_values.push(Box::new(pattern));
        param_idx += 1;
    }

    if let Some(ref tags) = filter.tags {
        if !tags.is_empty() {
            let placeholders: Vec<String> = tags
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", param_idx + i))
                .collect();
            conditions.push(format!(
                "s.id IN (SELECT st.seed_id FROM seed_tags st JOIN tags t ON st.tag_id = t.id WHERE t.name IN ({}))",
                placeholders.join(", ")
            ));
            for tag in tags {
                param_values.push(Box::new(tag.clone()));
            }
            param_idx += tags.len();
        }
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT s.id, s.seed_value, s.comment, s.checkpoint, s.sample_image_id, s.created_at
         FROM seeds s
         WHERE {}
         ORDER BY s.created_at DESC",
        where_clause
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .context("Failed to prepare list_seeds query")?;
    let rows = stmt
        .query_map(params_ref.as_slice(), row_to_seed)
        .context("Failed to execute list_seeds query")?;

    let mut seeds = Vec::new();
    for row in rows {
        seeds.push(row.context("Failed to read seed row")?);
    }
    Ok(seeds)
}

pub fn delete_seed(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM seed_tags WHERE seed_id = ?1", params![id])
        .context("Failed to remove seed tag associations")?;
    conn.execute(
        "DELETE FROM seed_checkpoint_notes WHERE seed_id = ?1",
        params![id],
    )
    .context("Failed to remove seed checkpoint notes")?;
    conn.execute("DELETE FROM seeds WHERE id = ?1", params![id])
        .context("Failed to delete seed")?;
    Ok(())
}

pub fn add_seed_tag(conn: &Connection, seed_id: i64, tag_name: &str) -> Result<i64> {
    let tag_id = super::tags::get_or_create_tag(conn, tag_name)?;
    conn.execute(
        "INSERT OR IGNORE INTO seed_tags (seed_id, tag_id) VALUES (?1, ?2)",
        params![seed_id, tag_id],
    )
    .context("Failed to add seed tag")?;
    Ok(tag_id)
}

pub fn remove_seed_tag(conn: &Connection, seed_id: i64, tag_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM seed_tags WHERE seed_id = ?1 AND tag_id = ?2",
        params![seed_id, tag_id],
    )
    .context("Failed to remove seed tag")?;
    Ok(())
}

pub fn add_checkpoint_note(conn: &Connection, note: &SeedCheckpointNote) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO seed_checkpoint_notes (seed_id, checkpoint, note, sample_image_id)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            note.seed_id,
            note.checkpoint,
            note.note,
            note.sample_image_id,
        ],
    )
    .context("Failed to add seed checkpoint note")?;
    Ok(())
}

pub fn get_checkpoint_notes(conn: &Connection, seed_id: i64) -> Result<Vec<SeedCheckpointNote>> {
    let mut stmt = conn
        .prepare(
            "SELECT seed_id, checkpoint, note, sample_image_id
             FROM seed_checkpoint_notes
             WHERE seed_id = ?1
             ORDER BY checkpoint",
        )
        .context("Failed to prepare get_checkpoint_notes query")?;

    let rows = stmt
        .query_map(params![seed_id], |row| {
            Ok(SeedCheckpointNote {
                seed_id: row.get(0)?,
                checkpoint: row.get(1)?,
                note: row.get(2)?,
                sample_image_id: row.get(3)?,
            })
        })
        .context("Failed to execute get_checkpoint_notes query")?;

    let mut notes = Vec::new();
    for row in rows {
        notes.push(row.context("Failed to read checkpoint note row")?);
    }
    Ok(notes)
}

fn row_to_seed(row: &rusqlite::Row) -> rusqlite::Result<SeedEntry> {
    Ok(SeedEntry {
        id: Some(row.get(0)?),
        seed_value: row.get(1)?,
        comment: row.get(2)?,
        checkpoint: row.get(3)?,
        sample_image_id: row.get(4)?,
        created_at: row.get(5)?,
        tags: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Connection {
        db::open_memory_database().unwrap()
    }

    fn make_test_seed() -> SeedEntry {
        SeedEntry {
            id: None,
            seed_value: 12345,
            comment: "Strong center composition".to_string(),
            checkpoint: Some("dreamshaper_8.safetensors".to_string()),
            sample_image_id: None,
            created_at: None,
            tags: None,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let conn = setup();
        let seed = make_test_seed();
        let id = insert_seed(&conn, &seed).unwrap();

        let retrieved = get_seed(&conn, id).unwrap().unwrap();
        assert_eq!(retrieved.seed_value, 12345);
        assert_eq!(retrieved.comment, "Strong center composition");
        assert_eq!(retrieved.checkpoint.unwrap(), "dreamshaper_8.safetensors");
    }

    #[test]
    fn test_list_seeds_no_filter() {
        let conn = setup();
        insert_seed(&conn, &make_test_seed()).unwrap();
        insert_seed(
            &conn,
            &SeedEntry {
                seed_value: 99999,
                comment: "Chaotic multi-element".to_string(),
                ..make_test_seed()
            },
        )
        .unwrap();

        let seeds = list_seeds(&conn, &SeedFilter::default()).unwrap();
        assert_eq!(seeds.len(), 2);
    }

    #[test]
    fn test_list_seeds_with_checkpoint_filter() {
        let conn = setup();
        insert_seed(&conn, &make_test_seed()).unwrap();
        insert_seed(
            &conn,
            &SeedEntry {
                checkpoint: Some("deliberate.safetensors".to_string()),
                ..make_test_seed()
            },
        )
        .unwrap();

        let filter = SeedFilter {
            checkpoint: Some("dreamshaper_8.safetensors".to_string()),
            ..Default::default()
        };
        let seeds = list_seeds(&conn, &filter).unwrap();
        assert_eq!(seeds.len(), 1);
    }

    #[test]
    fn test_list_seeds_with_search() {
        let conn = setup();
        insert_seed(&conn, &make_test_seed()).unwrap();
        insert_seed(
            &conn,
            &SeedEntry {
                comment: "Portrait framing".to_string(),
                ..make_test_seed()
            },
        )
        .unwrap();

        let filter = SeedFilter {
            search: Some("center".to_string()),
            ..Default::default()
        };
        let seeds = list_seeds(&conn, &filter).unwrap();
        assert_eq!(seeds.len(), 1);
    }

    #[test]
    fn test_delete_seed() {
        let conn = setup();
        let id = insert_seed(&conn, &make_test_seed()).unwrap();
        delete_seed(&conn, id).unwrap();

        let result = get_seed(&conn, id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_seed_tags() {
        let conn = setup();
        let seed_id = insert_seed(&conn, &make_test_seed()).unwrap();

        let tag_id = add_seed_tag(&conn, seed_id, "portrait").unwrap();
        add_seed_tag(&conn, seed_id, "symmetric").unwrap();

        // Filter by tag
        let filter = SeedFilter {
            tags: Some(vec!["portrait".to_string()]),
            ..Default::default()
        };
        let seeds = list_seeds(&conn, &filter).unwrap();
        assert_eq!(seeds.len(), 1);

        remove_seed_tag(&conn, seed_id, tag_id).unwrap();

        let filter2 = SeedFilter {
            tags: Some(vec!["portrait".to_string()]),
            ..Default::default()
        };
        let seeds2 = list_seeds(&conn, &filter2).unwrap();
        assert_eq!(seeds2.len(), 0);
    }

    #[test]
    fn test_checkpoint_notes() {
        let conn = setup();
        let seed_id = insert_seed(&conn, &make_test_seed()).unwrap();

        add_checkpoint_note(
            &conn,
            &SeedCheckpointNote {
                seed_id,
                checkpoint: "dreamshaper_8.safetensors".to_string(),
                note: "Great for portraits".to_string(),
                sample_image_id: None,
            },
        )
        .unwrap();

        add_checkpoint_note(
            &conn,
            &SeedCheckpointNote {
                seed_id,
                checkpoint: "deliberate.safetensors".to_string(),
                note: "More abstract results".to_string(),
                sample_image_id: None,
            },
        )
        .unwrap();

        let notes = get_checkpoint_notes(&conn, seed_id).unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].checkpoint, "deliberate.safetensors");
        assert_eq!(notes[1].checkpoint, "dreamshaper_8.safetensors");
    }

    #[test]
    fn test_checkpoint_note_upsert() {
        let conn = setup();
        let seed_id = insert_seed(&conn, &make_test_seed()).unwrap();

        let note = SeedCheckpointNote {
            seed_id,
            checkpoint: "dreamshaper_8.safetensors".to_string(),
            note: "Original note".to_string(),
            sample_image_id: None,
        };
        add_checkpoint_note(&conn, &note).unwrap();

        let updated = SeedCheckpointNote {
            note: "Updated note".to_string(),
            ..note
        };
        add_checkpoint_note(&conn, &updated).unwrap();

        let notes = get_checkpoint_notes(&conn, seed_id).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note, "Updated note");
    }
}
