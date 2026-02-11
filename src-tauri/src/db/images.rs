use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::types::gallery::{GalleryFilter, GallerySortField, ImageEntry, SortOrder};

pub fn insert_image(conn: &Connection, image: &ImageEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO images (
            id, filename, created_at, positive_prompt, negative_prompt,
            original_idea, checkpoint, width, height, steps, cfg_scale,
            sampler, scheduler, seed, pipeline_log, selected_concept,
            auto_approved, caption, caption_edited, rating, favorite,
            deleted, user_note
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
        )",
        params![
            image.id,
            image.filename,
            image.created_at,
            image.positive_prompt,
            image.negative_prompt,
            image.original_idea,
            image.checkpoint,
            image.width,
            image.height,
            image.steps,
            image.cfg_scale,
            image.sampler,
            image.scheduler,
            image.seed,
            image.pipeline_log,
            image.selected_concept,
            image.auto_approved,
            image.caption,
            image.caption_edited,
            image.rating,
            image.favorite,
            image.deleted,
            image.user_note,
        ],
    )
    .context("Failed to insert image")?;
    Ok(())
}

pub fn get_image(conn: &Connection, id: &str) -> Result<Option<ImageEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, filename, created_at, positive_prompt, negative_prompt,
                    original_idea, checkpoint, width, height, steps, cfg_scale,
                    sampler, scheduler, seed, pipeline_log, selected_concept,
                    auto_approved, caption, caption_edited, rating, favorite,
                    deleted, user_note
             FROM images WHERE id = ?1",
        )
        .context("Failed to prepare get_image query")?;

    let mut rows = stmt
        .query_map(params![id], row_to_image)
        .context("Failed to execute get_image query")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("Failed to read image row")?)),
        None => Ok(None),
    }
}

pub fn list_images(conn: &Connection, filter: &GalleryFilter) -> Result<Vec<ImageEntry>> {
    let (where_clause, mut param_values, next_idx) = build_filter_conditions(filter);

    let sort_col = match filter.sort_by {
        Some(GallerySortField::Rating) => "rating",
        Some(GallerySortField::Random) => "RANDOM()",
        _ => "created_at",
    };
    let sort_dir = match filter.sort_order {
        Some(SortOrder::Asc) => "ASC",
        _ => "DESC",
    };

    let limit = filter.limit.unwrap_or(50);
    let offset = filter.offset.unwrap_or(0);

    let sql = format!(
        "SELECT id, filename, created_at, positive_prompt, negative_prompt,
                original_idea, checkpoint, width, height, steps, cfg_scale,
                sampler, scheduler, seed, pipeline_log, selected_concept,
                auto_approved, caption, caption_edited, rating, favorite,
                deleted, user_note
         FROM images WHERE {} ORDER BY {} {} LIMIT ?{} OFFSET ?{}",
        where_clause,
        sort_col,
        sort_dir,
        next_idx,
        next_idx + 1
    );

    param_values.push(Box::new(limit));
    param_values.push(Box::new(offset));

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .context("Failed to prepare list_images query")?;
    let rows = stmt
        .query_map(params_ref.as_slice(), row_to_image)
        .context("Failed to execute list_images query")?;

    let mut images = Vec::new();
    for row in rows {
        images.push(row.context("Failed to read image row")?);
    }
    Ok(images)
}

fn build_filter_conditions(
    filter: &GalleryFilter,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>, usize) {
    let mut conditions = vec!["1=1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1;

    let show_deleted = filter.show_deleted.unwrap_or(false);
    conditions.push(format!("deleted = ?{}", idx));
    params.push(Box::new(show_deleted));
    idx += 1;

    if let Some(ref checkpoint) = filter.checkpoint {
        conditions.push(format!("checkpoint = ?{}", idx));
        params.push(Box::new(checkpoint.clone()));
        idx += 1;
    }
    if let Some(min_rating) = filter.min_rating {
        conditions.push(format!("rating >= ?{}", idx));
        params.push(Box::new(min_rating));
        idx += 1;
    }
    if filter.favorite_only.unwrap_or(false) {
        conditions.push(format!("favorite = ?{}", idx));
        params.push(Box::new(true));
        idx += 1;
    }
    if let Some(auto_approved) = filter.auto_approved {
        conditions.push(format!("auto_approved = ?{}", idx));
        params.push(Box::new(auto_approved));
        idx += 1;
    }
    if let Some(ref search) = filter.search {
        let like = format!("%{}%", search);
        conditions.push(format!(
            "(positive_prompt LIKE ?{p} OR negative_prompt LIKE ?{p} \
             OR original_idea LIKE ?{p} OR caption LIKE ?{p})",
            p = idx
        ));
        params.push(Box::new(like));
        idx += 1;
    }

    (conditions.join(" AND "), params, idx)
}

pub fn update_image_rating(conn: &Connection, id: &str, rating: Option<u32>) -> Result<()> {
    conn.execute(
        "UPDATE images SET rating = ?1 WHERE id = ?2",
        params![rating, id],
    )
    .context("Failed to update image rating")?;
    Ok(())
}

pub fn update_image_favorite(conn: &Connection, id: &str, favorite: bool) -> Result<()> {
    conn.execute(
        "UPDATE images SET favorite = ?1 WHERE id = ?2",
        params![favorite, id],
    )
    .context("Failed to update image favorite")?;
    Ok(())
}

pub fn update_image_caption(
    conn: &Connection,
    id: &str,
    caption: &str,
    edited: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE images SET caption = ?1, caption_edited = ?2 WHERE id = ?3",
        params![caption, edited, id],
    )
    .context("Failed to update image caption")?;
    Ok(())
}

pub fn update_image_note(conn: &Connection, id: &str, note: &str) -> Result<()> {
    conn.execute(
        "UPDATE images SET user_note = ?1 WHERE id = ?2",
        params![note, id],
    )
    .context("Failed to update image note")?;
    Ok(())
}

pub fn soft_delete_image(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE images SET deleted = TRUE WHERE id = ?1",
        params![id],
    )
    .context("Failed to soft-delete image")?;
    Ok(())
}

pub fn restore_image(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE images SET deleted = FALSE WHERE id = ?1",
        params![id],
    )
    .context("Failed to restore image")?;
    Ok(())
}

pub fn permanently_delete_image(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM images WHERE id = ?1", params![id])
        .context("Failed to permanently delete image")?;
    Ok(())
}

pub fn row_to_image(row: &rusqlite::Row) -> rusqlite::Result<ImageEntry> {
    Ok(ImageEntry {
        id: row.get(0)?,
        filename: row.get(1)?,
        created_at: row.get(2)?,
        positive_prompt: row.get(3)?,
        negative_prompt: row.get(4)?,
        original_idea: row.get(5)?,
        checkpoint: row.get(6)?,
        width: row.get(7)?,
        height: row.get(8)?,
        steps: row.get(9)?,
        cfg_scale: row.get(10)?,
        sampler: row.get(11)?,
        scheduler: row.get(12)?,
        seed: row.get(13)?,
        pipeline_log: row.get(14)?,
        selected_concept: row.get(15)?,
        auto_approved: row.get(16)?,
        caption: row.get(17)?,
        caption_edited: row.get(18)?,
        rating: row.get(19)?,
        favorite: row.get(20)?,
        deleted: row.get(21)?,
        user_note: row.get(22)?,
        tags: None,
    })
}

#[cfg(test)]
#[path = "images_test.rs"]
mod tests;
