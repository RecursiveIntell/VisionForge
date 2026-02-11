use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::types::checkpoints::{
    CheckpointObservation, CheckpointProfile, ObservationSource, PromptTerm, TermStrength,
};

pub fn upsert_checkpoint(conn: &Connection, profile: &CheckpointProfile) -> Result<i64> {
    let strengths_json = profile
        .strengths
        .as_ref()
        .map(|s| serde_json::to_string(s).unwrap_or_default());
    let weaknesses_json = profile
        .weaknesses
        .as_ref()
        .map(|w| serde_json::to_string(w).unwrap_or_default());

    conn.execute(
        "INSERT INTO checkpoints (
            filename, display_name, base_model, strengths, weaknesses,
            preferred_cfg, cfg_range_low, cfg_range_high, preferred_sampler,
            preferred_scheduler, optimal_resolution, notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(filename) DO UPDATE SET
            display_name = COALESCE(excluded.display_name, display_name),
            base_model = COALESCE(excluded.base_model, base_model),
            strengths = COALESCE(excluded.strengths, strengths),
            weaknesses = COALESCE(excluded.weaknesses, weaknesses),
            preferred_cfg = COALESCE(excluded.preferred_cfg, preferred_cfg),
            cfg_range_low = COALESCE(excluded.cfg_range_low, cfg_range_low),
            cfg_range_high = COALESCE(excluded.cfg_range_high, cfg_range_high),
            preferred_sampler = COALESCE(excluded.preferred_sampler, preferred_sampler),
            preferred_scheduler = COALESCE(excluded.preferred_scheduler, preferred_scheduler),
            optimal_resolution = COALESCE(excluded.optimal_resolution, optimal_resolution),
            notes = COALESCE(excluded.notes, notes)",
        params![
            profile.filename,
            profile.display_name,
            profile.base_model,
            strengths_json,
            weaknesses_json,
            profile.preferred_cfg,
            profile.cfg_range_low,
            profile.cfg_range_high,
            profile.preferred_sampler,
            profile.preferred_scheduler,
            profile.optimal_resolution,
            profile.notes,
        ],
    )
    .context("Failed to upsert checkpoint")?;

    let id: i64 = conn
        .query_row(
            "SELECT id FROM checkpoints WHERE filename = ?1",
            params![profile.filename],
            |row| row.get(0),
        )
        .context("Failed to get checkpoint id after upsert")?;

    Ok(id)
}

pub fn get_checkpoint(conn: &Connection, filename: &str) -> Result<Option<CheckpointProfile>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, filename, display_name, base_model, created_at,
                    strengths, weaknesses, preferred_cfg, cfg_range_low,
                    cfg_range_high, preferred_sampler, preferred_scheduler,
                    optimal_resolution, notes
             FROM checkpoints WHERE filename = ?1",
        )
        .context("Failed to prepare get_checkpoint query")?;

    let mut rows = stmt
        .query_map(params![filename], row_to_profile)
        .context("Failed to execute get_checkpoint query")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("Failed to read checkpoint row")?)),
        None => Ok(None),
    }
}

pub fn list_checkpoints(conn: &Connection) -> Result<Vec<CheckpointProfile>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, filename, display_name, base_model, created_at,
                    strengths, weaknesses, preferred_cfg, cfg_range_low,
                    cfg_range_high, preferred_sampler, preferred_scheduler,
                    optimal_resolution, notes
             FROM checkpoints ORDER BY filename",
        )
        .context("Failed to prepare list_checkpoints query")?;

    let rows = stmt
        .query_map([], row_to_profile)
        .context("Failed to execute list_checkpoints query")?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.context("Failed to read checkpoint row")?);
    }
    Ok(profiles)
}

pub fn add_prompt_term(conn: &Connection, term: &PromptTerm) -> Result<i64> {
    conn.execute(
        "INSERT INTO checkpoint_prompt_terms (checkpoint_id, term, effect, strength, example_image_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            term.checkpoint_id,
            term.term,
            term.effect,
            term.strength.as_str(),
            term.example_image_id,
        ],
    )
    .context("Failed to add prompt term")?;
    Ok(conn.last_insert_rowid())
}

pub fn get_prompt_terms(conn: &Connection, checkpoint_id: i64) -> Result<Vec<PromptTerm>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, checkpoint_id, term, effect, strength, example_image_id, created_at
             FROM checkpoint_prompt_terms WHERE checkpoint_id = ?1 ORDER BY term",
        )
        .context("Failed to prepare get_prompt_terms query")?;

    let rows = stmt
        .query_map(params![checkpoint_id], |row| {
            let strength_str: String = row.get(4)?;
            Ok(PromptTerm {
                id: Some(row.get(0)?),
                checkpoint_id: row.get(1)?,
                term: row.get(2)?,
                effect: row.get(3)?,
                strength: TermStrength::from_str(&strength_str).unwrap_or(TermStrength::Moderate),
                example_image_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .context("Failed to execute get_prompt_terms query")?;

    let mut terms = Vec::new();
    for row in rows {
        terms.push(row.context("Failed to read prompt term row")?);
    }
    Ok(terms)
}

pub fn add_observation(conn: &Connection, obs: &CheckpointObservation) -> Result<i64> {
    conn.execute(
        "INSERT INTO checkpoint_observations (checkpoint_id, observation, source, comparison_id)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            obs.checkpoint_id,
            obs.observation,
            obs.source.as_str(),
            obs.comparison_id,
        ],
    )
    .context("Failed to add checkpoint observation")?;
    Ok(conn.last_insert_rowid())
}

pub fn get_observations(
    conn: &Connection,
    checkpoint_id: i64,
) -> Result<Vec<CheckpointObservation>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, checkpoint_id, observation, source, comparison_id, created_at
             FROM checkpoint_observations WHERE checkpoint_id = ?1
             ORDER BY created_at DESC",
        )
        .context("Failed to prepare get_observations query")?;

    let rows = stmt
        .query_map(params![checkpoint_id], |row| {
            let source_str: String = row.get(3)?;
            Ok(CheckpointObservation {
                id: Some(row.get(0)?),
                checkpoint_id: row.get(1)?,
                observation: row.get(2)?,
                source: ObservationSource::from_str(&source_str).unwrap_or(ObservationSource::User),
                comparison_id: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .context("Failed to execute get_observations query")?;

    let mut observations = Vec::new();
    for row in rows {
        observations.push(row.context("Failed to read observation row")?);
    }
    Ok(observations)
}

pub fn get_checkpoint_context(conn: &Connection, filename: &str) -> Result<String> {
    let profile = get_checkpoint(conn, filename)?;
    let Some(profile) = profile else {
        return Ok(String::new());
    };

    let checkpoint_id = profile.id.unwrap_or(0);
    let terms = get_prompt_terms(conn, checkpoint_id)?;

    let mut context = String::new();
    if let Some(name) = &profile.display_name {
        context.push_str(&format!("Checkpoint: {}\n", name));
    }
    if let Some(base) = &profile.base_model {
        context.push_str(&format!("Base model: {}\n", base));
    }
    if let Some(ref strengths) = profile.strengths {
        context.push_str(&format!("Strengths: {}\n", strengths.join(", ")));
    }
    if let Some(ref weaknesses) = profile.weaknesses {
        context.push_str(&format!("Weaknesses: {}\n", weaknesses.join(", ")));
    }
    if let Some(notes) = &profile.notes {
        context.push_str(&format!("Notes: {}\n", notes));
    }
    if !terms.is_empty() {
        context.push_str("Known terms:\n");
        for t in &terms {
            context.push_str(&format!(
                "- {} ({}): {}\n",
                t.term,
                t.strength.as_str(),
                t.effect
            ));
        }
    }
    Ok(context)
}

fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<CheckpointProfile> {
    let strengths_raw: Option<String> = row.get(5)?;
    let weaknesses_raw: Option<String> = row.get(6)?;

    let strengths = strengths_raw.and_then(|s| serde_json::from_str(&s).ok());
    let weaknesses = weaknesses_raw.and_then(|s| serde_json::from_str(&s).ok());

    Ok(CheckpointProfile {
        id: Some(row.get(0)?),
        filename: row.get(1)?,
        display_name: row.get(2)?,
        base_model: row.get(3)?,
        created_at: row.get(4)?,
        strengths,
        weaknesses,
        preferred_cfg: row.get(7)?,
        cfg_range_low: row.get(8)?,
        cfg_range_high: row.get(9)?,
        preferred_sampler: row.get(10)?,
        preferred_scheduler: row.get(11)?,
        optimal_resolution: row.get(12)?,
        notes: row.get(13)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Connection {
        db::open_memory_database().unwrap()
    }

    fn make_profile() -> CheckpointProfile {
        CheckpointProfile {
            id: None,
            filename: "dreamshaper_8.safetensors".to_string(),
            display_name: Some("DreamShaper v8".to_string()),
            base_model: Some("SD 1.5".to_string()),
            created_at: None,
            strengths: Some(vec![
                "photorealism".to_string(),
                "cinematic lighting".to_string(),
            ]),
            weaknesses: Some(vec!["text rendering".to_string()]),
            preferred_cfg: Some(7.5),
            cfg_range_low: Some(6.0),
            cfg_range_high: Some(9.0),
            preferred_sampler: Some("dpmpp_2m".to_string()),
            preferred_scheduler: Some("karras".to_string()),
            optimal_resolution: Some("512x768".to_string()),
            notes: Some("Good all-around checkpoint".to_string()),
        }
    }

    #[test]
    fn test_upsert_and_get() {
        let conn = setup();
        let id = upsert_checkpoint(&conn, &make_profile()).unwrap();
        assert!(id > 0);

        let profile = get_checkpoint(&conn, "dreamshaper_8.safetensors")
            .unwrap()
            .unwrap();
        assert_eq!(profile.display_name.unwrap(), "DreamShaper v8");
        assert_eq!(profile.strengths.unwrap().len(), 2);
    }

    #[test]
    fn test_upsert_updates_existing() {
        let conn = setup();
        upsert_checkpoint(&conn, &make_profile()).unwrap();

        let updated = CheckpointProfile {
            notes: Some("Updated notes".to_string()),
            ..make_profile()
        };
        upsert_checkpoint(&conn, &updated).unwrap();

        let all = list_checkpoints(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].notes.as_deref(), Some("Updated notes"));
    }

    #[test]
    fn test_prompt_terms() {
        let conn = setup();
        let cp_id = upsert_checkpoint(&conn, &make_profile()).unwrap();

        add_prompt_term(
            &conn,
            &PromptTerm {
                id: None,
                checkpoint_id: cp_id,
                term: "cinematic lighting".to_string(),
                effect: "Strong volumetric light".to_string(),
                strength: TermStrength::Strong,
                example_image_id: None,
                created_at: None,
            },
        )
        .unwrap();

        let terms = get_prompt_terms(&conn, cp_id).unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].term, "cinematic lighting");
    }

    #[test]
    fn test_observations() {
        let conn = setup();
        let cp_id = upsert_checkpoint(&conn, &make_profile()).unwrap();

        add_observation(
            &conn,
            &CheckpointObservation {
                id: None,
                checkpoint_id: cp_id,
                observation: "Great for portraits".to_string(),
                source: ObservationSource::User,
                comparison_id: None,
                created_at: None,
            },
        )
        .unwrap();

        let obs = get_observations(&conn, cp_id).unwrap();
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].observation, "Great for portraits");
    }

    #[test]
    fn test_checkpoint_context_string() {
        let conn = setup();
        let cp_id = upsert_checkpoint(&conn, &make_profile()).unwrap();
        add_prompt_term(
            &conn,
            &PromptTerm {
                id: None,
                checkpoint_id: cp_id,
                term: "cinematic lighting".to_string(),
                effect: "Produces volumetric rays".to_string(),
                strength: TermStrength::Strong,
                example_image_id: None,
                created_at: None,
            },
        )
        .unwrap();

        let ctx = get_checkpoint_context(&conn, "dreamshaper_8.safetensors").unwrap();
        assert!(ctx.contains("DreamShaper v8"));
        assert!(ctx.contains("photorealism"));
        assert!(ctx.contains("cinematic lighting"));
    }

    #[test]
    fn test_get_nonexistent_checkpoint() {
        let conn = setup();
        assert!(get_checkpoint(&conn, "nope.safetensors").unwrap().is_none());
    }

    #[test]
    fn test_empty_context_for_unknown_checkpoint() {
        let conn = setup();
        let ctx = get_checkpoint_context(&conn, "unknown.safetensors").unwrap();
        assert!(ctx.is_empty());
    }
}
