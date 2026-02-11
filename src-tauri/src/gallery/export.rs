use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::gallery::storage;
use crate::types::gallery::ImageEntry;

/// Export manifest entry — included in the ZIP as JSON.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEntry {
    filename: String,
    positive_prompt: Option<String>,
    negative_prompt: Option<String>,
    original_idea: Option<String>,
    checkpoint: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    steps: Option<u32>,
    cfg_scale: Option<f64>,
    sampler: Option<String>,
    scheduler: Option<String>,
    seed: Option<i64>,
    rating: Option<u32>,
    caption: Option<String>,
}

/// Create a ZIP bundle containing the specified images and a JSON manifest.
/// Returns the path to the created ZIP file.
pub fn create_export_bundle(
    images: &[ImageEntry],
    output_path: &Path,
) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .with_context(|| format!("Failed to create export file at {}", output_path.display()))?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Stored);

    let mut manifest = Vec::new();

    for image in images {
        let image_path = storage::get_image_path(&image.filename);

        if image_path.exists() {
            let image_bytes = std::fs::read(&image_path)
                .with_context(|| format!("Failed to read {}", image_path.display()))?;

            zip.start_file(&image.filename, options)
                .context("Failed to add file to ZIP")?;
            zip.write_all(&image_bytes)
                .context("Failed to write image to ZIP")?;
        }

        manifest.push(ManifestEntry {
            filename: image.filename.clone(),
            positive_prompt: image.positive_prompt.clone(),
            negative_prompt: image.negative_prompt.clone(),
            original_idea: image.original_idea.clone(),
            checkpoint: image.checkpoint.clone(),
            width: image.width,
            height: image.height,
            steps: image.steps,
            cfg_scale: image.cfg_scale,
            sampler: image.sampler.clone(),
            scheduler: image.scheduler.clone(),
            seed: image.seed,
            rating: image.rating,
            caption: image.caption.clone(),
        });
    }

    // Write JSON manifest
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize manifest")?;
    zip.start_file("manifest.json", options)
        .context("Failed to add manifest to ZIP")?;
    zip.write_all(manifest_json.as_bytes())
        .context("Failed to write manifest to ZIP")?;

    // Write CSV manifest
    let csv = build_csv_manifest(&manifest);
    zip.start_file("manifest.csv", options)
        .context("Failed to add CSV manifest to ZIP")?;
    zip.write_all(csv.as_bytes())
        .context("Failed to write CSV manifest to ZIP")?;

    zip.finish().context("Failed to finalize ZIP")?;
    Ok(())
}

fn build_csv_manifest(entries: &[ManifestEntry]) -> String {
    let mut csv = String::from(
        "filename,positivePrompt,negativePrompt,checkpoint,width,height,steps,cfgScale,sampler,scheduler,seed,rating,caption\n"
    );

    for e in entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&e.filename),
            csv_escape(&e.positive_prompt.as_deref().unwrap_or("")),
            csv_escape(&e.negative_prompt.as_deref().unwrap_or("")),
            csv_escape(&e.checkpoint.as_deref().unwrap_or("")),
            e.width.map(|v| v.to_string()).unwrap_or_default(),
            e.height.map(|v| v.to_string()).unwrap_or_default(),
            e.steps.map(|v| v.to_string()).unwrap_or_default(),
            e.cfg_scale.map(|v| v.to_string()).unwrap_or_default(),
            csv_escape(&e.sampler.as_deref().unwrap_or("")),
            csv_escape(&e.scheduler.as_deref().unwrap_or("")),
            e.seed.map(|v| v.to_string()).unwrap_or_default(),
            e.rating.map(|v| v.to_string()).unwrap_or_default(),
            csv_escape(&e.caption.as_deref().unwrap_or("")),
        ));
    }

    csv
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_escape_no_special() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_with_comma() {
        assert_eq!(csv_escape("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn test_csv_escape_with_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_build_csv_manifest() {
        let entries = vec![ManifestEntry {
            filename: "test.png".to_string(),
            positive_prompt: Some("a cat".to_string()),
            negative_prompt: Some("lowres".to_string()),
            original_idea: None,
            checkpoint: Some("ds8".to_string()),
            width: Some(512),
            height: Some(768),
            steps: Some(25),
            cfg_scale: Some(7.5),
            sampler: Some("dpmpp_2m".to_string()),
            scheduler: Some("karras".to_string()),
            seed: Some(42),
            rating: Some(4),
            caption: None,
        }];
        let csv = build_csv_manifest(&entries);
        assert!(csv.contains("filename,"));
        assert!(csv.contains("test.png"));
        assert!(csv.contains("a cat"));
    }

    #[test]
    fn test_create_export_bundle() {
        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("export.zip");

        // Empty export (no actual image files on disk)
        let images = vec![ImageEntry {
            id: "img-1".to_string(),
            filename: "nonexistent.png".to_string(),
            created_at: "2026-01-15T10:00:00".to_string(),
            positive_prompt: Some("a cat".to_string()),
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
        }];

        create_export_bundle(&images, &zip_path).unwrap();
        assert!(zip_path.exists());

        // Verify ZIP contains manifest
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.contains(&"manifest.json".to_string()));
        assert!(names.contains(&"manifest.csv".to_string()));
    }
}
