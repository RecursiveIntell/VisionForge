use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::types::comparison::Comparison;

pub fn insert_comparison(conn: &Connection, comparison: &Comparison) -> Result<()> {
    conn.execute(
        "INSERT INTO comparisons (id, image_a_id, image_b_id, variable_changed, note)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            comparison.id,
            comparison.image_a_id,
            comparison.image_b_id,
            comparison.variable_changed,
            comparison.note,
        ],
    )
    .context("Failed to insert comparison")?;
    Ok(())
}

pub fn get_comparison(conn: &Connection, id: &str) -> Result<Option<Comparison>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, image_a_id, image_b_id, variable_changed, note, created_at
             FROM comparisons WHERE id = ?1",
        )
        .context("Failed to prepare get_comparison query")?;

    let mut rows = stmt
        .query_map(params![id], row_to_comparison)
        .context("Failed to execute get_comparison query")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("Failed to read comparison row")?)),
        None => Ok(None),
    }
}

pub fn list_comparisons(conn: &Connection) -> Result<Vec<Comparison>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, image_a_id, image_b_id, variable_changed, note, created_at
             FROM comparisons ORDER BY created_at DESC",
        )
        .context("Failed to prepare list_comparisons query")?;

    let rows = stmt
        .query_map([], row_to_comparison)
        .context("Failed to execute list_comparisons query")?;

    let mut comparisons = Vec::new();
    for row in rows {
        comparisons.push(row.context("Failed to read comparison row")?);
    }
    Ok(comparisons)
}

pub fn list_comparisons_for_checkpoint(
    conn: &Connection,
    checkpoint: &str,
) -> Result<Vec<Comparison>> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.image_a_id, c.image_b_id, c.variable_changed, c.note, c.created_at
             FROM comparisons c
             JOIN images ia ON c.image_a_id = ia.id
             JOIN images ib ON c.image_b_id = ib.id
             WHERE ia.checkpoint = ?1 OR ib.checkpoint = ?1
             ORDER BY c.created_at DESC",
        )
        .context("Failed to prepare list_comparisons_for_checkpoint query")?;

    let rows = stmt
        .query_map(params![checkpoint], row_to_comparison)
        .context("Failed to execute list_comparisons_for_checkpoint query")?;

    let mut comparisons = Vec::new();
    for row in rows {
        comparisons.push(row.context("Failed to read comparison row")?);
    }
    Ok(comparisons)
}

pub fn update_comparison_note(conn: &Connection, id: &str, note: &str) -> Result<()> {
    conn.execute(
        "UPDATE comparisons SET note = ?1 WHERE id = ?2",
        params![note, id],
    )
    .context("Failed to update comparison note")?;
    Ok(())
}

pub fn delete_comparison(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM comparisons WHERE id = ?1", params![id])
        .context("Failed to delete comparison")?;
    Ok(())
}

fn row_to_comparison(row: &rusqlite::Row) -> rusqlite::Result<Comparison> {
    Ok(Comparison {
        id: row.get(0)?,
        image_a_id: row.get(1)?,
        image_b_id: row.get(2)?,
        variable_changed: row.get(3)?,
        note: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::images;
    use crate::types::gallery::ImageEntry;

    fn setup() -> Connection { db::open_memory_database().unwrap() }

    fn insert_test_image(conn: &Connection, id: &str, checkpoint: &str) {
        let img = ImageEntry {
            id: id.to_string(),
            filename: format!("{}.png", id),
            created_at: "2026-01-15T10:00:00".to_string(),
            checkpoint: Some(checkpoint.to_string()),
            positive_prompt: None, negative_prompt: None, original_idea: None,
            width: None, height: None, steps: None, cfg_scale: None,
            sampler: None, scheduler: None, seed: None, pipeline_log: None,
            selected_concept: None, auto_approved: false, caption: None,
            caption_edited: false, rating: None, favorite: false,
            deleted: false, user_note: None, tags: None,
        };
        images::insert_image(conn, &img).unwrap();
    }

    #[test]
    fn test_insert_and_get() {
        let conn = setup();
        insert_test_image(&conn, "img-a", "dreamshaper");
        insert_test_image(&conn, "img-b", "deliberate");

        let comp = Comparison {
            id: "cmp-001".to_string(),
            image_a_id: "img-a".to_string(),
            image_b_id: "img-b".to_string(),
            variable_changed: "checkpoint".to_string(),
            note: Some("DreamShaper has better lighting".to_string()),
            created_at: None,
        };
        insert_comparison(&conn, &comp).unwrap();

        let retrieved = get_comparison(&conn, "cmp-001").unwrap().unwrap();
        assert_eq!(retrieved.variable_changed, "checkpoint");
        assert_eq!(retrieved.note.unwrap(), "DreamShaper has better lighting");
    }

    #[test]
    fn test_list_comparisons() {
        let conn = setup();
        insert_test_image(&conn, "img-a", "ds");
        insert_test_image(&conn, "img-b", "dl");

        for i in 0..3 {
            insert_comparison(&conn, &Comparison {
                id: format!("cmp-{}", i),
                image_a_id: "img-a".to_string(),
                image_b_id: "img-b".to_string(),
                variable_changed: "cfg".to_string(),
                note: None, created_at: None,
            }).unwrap();
        }

        let all = list_comparisons(&conn).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_list_for_checkpoint() {
        let conn = setup();
        insert_test_image(&conn, "img-a", "dreamshaper");
        insert_test_image(&conn, "img-b", "deliberate");
        insert_test_image(&conn, "img-c", "dreamshaper");

        insert_comparison(&conn, &Comparison {
            id: "cmp-1".to_string(),
            image_a_id: "img-a".to_string(),
            image_b_id: "img-b".to_string(),
            variable_changed: "checkpoint".to_string(),
            note: None, created_at: None,
        }).unwrap();

        let ds_comps = list_comparisons_for_checkpoint(&conn, "dreamshaper").unwrap();
        assert_eq!(ds_comps.len(), 1);

        let dl_comps = list_comparisons_for_checkpoint(&conn, "deliberate").unwrap();
        assert_eq!(dl_comps.len(), 1);
    }

    #[test]
    fn test_update_note() {
        let conn = setup();
        insert_test_image(&conn, "img-a", "ds");
        insert_test_image(&conn, "img-b", "dl");

        insert_comparison(&conn, &Comparison {
            id: "cmp-1".to_string(),
            image_a_id: "img-a".to_string(),
            image_b_id: "img-b".to_string(),
            variable_changed: "sampler".to_string(),
            note: None, created_at: None,
        }).unwrap();

        update_comparison_note(&conn, "cmp-1", "euler gives sharper edges").unwrap();
        let comp = get_comparison(&conn, "cmp-1").unwrap().unwrap();
        assert_eq!(comp.note.unwrap(), "euler gives sharper edges");
    }

    #[test]
    fn test_delete() {
        let conn = setup();
        insert_test_image(&conn, "img-a", "ds");
        insert_test_image(&conn, "img-b", "dl");

        insert_comparison(&conn, &Comparison {
            id: "cmp-1".to_string(),
            image_a_id: "img-a".to_string(),
            image_b_id: "img-b".to_string(),
            variable_changed: "cfg".to_string(),
            note: None, created_at: None,
        }).unwrap();

        delete_comparison(&conn, "cmp-1").unwrap();
        assert!(get_comparison(&conn, "cmp-1").unwrap().is_none());
    }
}
