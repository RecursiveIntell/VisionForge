use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::types::gallery::TagEntry;

pub fn get_or_create_tag(conn: &Connection, name: &str) -> Result<i64> {
    let normalized = name.trim().to_lowercase();

    conn.execute(
        "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
        params![normalized],
    )
    .context("Failed to insert tag")?;

    let id: i64 = conn
        .query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![normalized],
            |row| row.get(0),
        )
        .context("Failed to get tag id")?;

    Ok(id)
}

pub fn get_tag_by_name(conn: &Connection, name: &str) -> Result<Option<TagEntry>> {
    let normalized = name.trim().to_lowercase();
    let mut stmt = conn
        .prepare("SELECT id, name FROM tags WHERE name = ?1")
        .context("Failed to prepare get_tag query")?;

    let mut rows = stmt
        .query_map(params![normalized], |row| {
            Ok(TagEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                source: None,
                confidence: None,
            })
        })
        .context("Failed to execute get_tag query")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("Failed to read tag row")?)),
        None => Ok(None),
    }
}

pub fn list_all_tags(conn: &Connection) -> Result<Vec<TagEntry>> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM tags ORDER BY name")
        .context("Failed to prepare list_tags query")?;

    let rows = stmt
        .query_map([], |row| {
            Ok(TagEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                source: None,
                confidence: None,
            })
        })
        .context("Failed to execute list_tags query")?;

    let mut tags = Vec::new();
    for row in rows {
        tags.push(row.context("Failed to read tag row")?);
    }
    Ok(tags)
}

pub fn delete_tag(conn: &Connection, tag_id: i64) -> Result<()> {
    conn.execute("DELETE FROM image_tags WHERE tag_id = ?1", params![tag_id])
        .context("Failed to remove tag associations")?;
    conn.execute("DELETE FROM seed_tags WHERE tag_id = ?1", params![tag_id])
        .context("Failed to remove seed tag associations")?;
    conn.execute("DELETE FROM tags WHERE id = ?1", params![tag_id])
        .context("Failed to delete tag")?;
    Ok(())
}

pub fn add_image_tag(
    conn: &Connection,
    image_id: &str,
    tag_name: &str,
    source: &str,
    confidence: Option<f64>,
) -> Result<i64> {
    let tag_id = get_or_create_tag(conn, tag_name)?;

    conn.execute(
        "INSERT OR REPLACE INTO image_tags (image_id, tag_id, source, confidence)
         VALUES (?1, ?2, ?3, ?4)",
        params![image_id, tag_id, source, confidence],
    )
    .context("Failed to add image tag")?;

    Ok(tag_id)
}

pub fn remove_image_tag(conn: &Connection, image_id: &str, tag_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM image_tags WHERE image_id = ?1 AND tag_id = ?2",
        params![image_id, tag_id],
    )
    .context("Failed to remove image tag")?;
    Ok(())
}

pub fn get_image_tags(conn: &Connection, image_id: &str) -> Result<Vec<TagEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, it.source, it.confidence
             FROM tags t
             JOIN image_tags it ON t.id = it.tag_id
             WHERE it.image_id = ?1
             ORDER BY t.name",
        )
        .context("Failed to prepare get_image_tags query")?;

    let rows = stmt
        .query_map(params![image_id], |row| {
            Ok(TagEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                source: row.get(2)?,
                confidence: row.get(3)?,
            })
        })
        .context("Failed to execute get_image_tags query")?;

    let mut tags = Vec::new();
    for row in rows {
        tags.push(row.context("Failed to read tag row")?);
    }
    Ok(tags)
}

pub fn search_tags(conn: &Connection, query: &str) -> Result<Vec<TagEntry>> {
    let pattern = format!("%{}%", query.trim().to_lowercase());
    let mut stmt = conn
        .prepare("SELECT id, name FROM tags WHERE name LIKE ?1 ORDER BY name LIMIT 20")
        .context("Failed to prepare search_tags query")?;

    let rows = stmt
        .query_map(params![pattern], |row| {
            Ok(TagEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                source: None,
                confidence: None,
            })
        })
        .context("Failed to execute search_tags query")?;

    let mut tags = Vec::new();
    for row in rows {
        tags.push(row.context("Failed to read tag row")?);
    }
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::images;
    use crate::types::gallery::ImageEntry;

    fn setup() -> Connection {
        db::open_memory_database().unwrap()
    }

    fn insert_test_image(conn: &Connection, id: &str) {
        let img = ImageEntry {
            id: id.to_string(),
            filename: format!("{}.png", id),
            created_at: "2026-01-15T10:00:00".to_string(),
            positive_prompt: None,
            negative_prompt: None,
            original_idea: None,
            checkpoint: None,
            width: None,
            height: None,
            steps: None,
            cfg_scale: None,
            sampler: None,
            scheduler: None,
            seed: None,
            pipeline_log: None,
            selected_concept: None,
            auto_approved: false,
            caption: None,
            caption_edited: false,
            rating: None,
            favorite: false,
            deleted: false,
            user_note: None,
            tags: None,
        };
        images::insert_image(conn, &img).unwrap();
    }

    #[test]
    fn test_get_or_create_tag() {
        let conn = setup();
        let id1 = get_or_create_tag(&conn, "portrait").unwrap();
        let id2 = get_or_create_tag(&conn, "portrait").unwrap();
        assert_eq!(id1, id2);

        let id3 = get_or_create_tag(&conn, "  Portrait  ").unwrap();
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_get_tag_by_name() {
        let conn = setup();
        get_or_create_tag(&conn, "landscape").unwrap();

        let tag = get_tag_by_name(&conn, "landscape").unwrap().unwrap();
        assert_eq!(tag.name, "landscape");

        let none = get_tag_by_name(&conn, "nonexistent").unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_list_all_tags() {
        let conn = setup();
        get_or_create_tag(&conn, "portrait").unwrap();
        get_or_create_tag(&conn, "landscape").unwrap();
        get_or_create_tag(&conn, "anime").unwrap();

        let tags = list_all_tags(&conn).unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "anime");
        assert_eq!(tags[1].name, "landscape");
        assert_eq!(tags[2].name, "portrait");
    }

    #[test]
    fn test_add_and_get_image_tags() {
        let conn = setup();
        insert_test_image(&conn, "img-001");

        add_image_tag(&conn, "img-001", "cat", "ai", Some(0.95)).unwrap();
        add_image_tag(&conn, "img-001", "throne", "user", None).unwrap();

        let tags = get_image_tags(&conn, "img-001").unwrap();
        assert_eq!(tags.len(), 2);

        let cat_tag = tags.iter().find(|t| t.name == "cat").unwrap();
        assert_eq!(cat_tag.source.as_deref(), Some("ai"));
        assert!((cat_tag.confidence.unwrap() - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_remove_image_tag() {
        let conn = setup();
        insert_test_image(&conn, "img-001");

        let tag_id = add_image_tag(&conn, "img-001", "cat", "user", None).unwrap();
        remove_image_tag(&conn, "img-001", tag_id).unwrap();

        let tags = get_image_tags(&conn, "img-001").unwrap();
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_delete_tag() {
        let conn = setup();
        insert_test_image(&conn, "img-001");

        let tag_id = add_image_tag(&conn, "img-001", "cat", "user", None).unwrap();
        delete_tag(&conn, tag_id).unwrap();

        let tags = get_image_tags(&conn, "img-001").unwrap();
        assert_eq!(tags.len(), 0);

        let all_tags = list_all_tags(&conn).unwrap();
        assert_eq!(all_tags.len(), 0);
    }

    #[test]
    fn test_search_tags() {
        let conn = setup();
        get_or_create_tag(&conn, "portrait").unwrap();
        get_or_create_tag(&conn, "landscape portrait").unwrap();
        get_or_create_tag(&conn, "anime").unwrap();

        let results = search_tags(&conn, "port").unwrap();
        assert_eq!(results.len(), 2);
    }
}
