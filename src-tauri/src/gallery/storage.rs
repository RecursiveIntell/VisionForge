use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::manager;

const THUMBNAIL_SIZE: u32 = 256;

/// Generate a filename for a new image: YYYY-MM-DD_HH-MM-SS_<short_uuid>.png
pub fn generate_filename() -> String {
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S");
    let short_id = &uuid::Uuid::new_v4().to_string()[..8];
    format!("{}_{}.png", timestamp, short_id)
}

/// Get the path to the originals directory.
pub fn originals_dir() -> PathBuf {
    manager::data_dir().join("images/originals")
}

/// Get the path to the thumbnails directory.
pub fn thumbnails_dir() -> PathBuf {
    manager::data_dir().join("images/thumbnails")
}

/// Get the full path to an original image by filename.
pub fn get_image_path(filename: &str) -> PathBuf {
    originals_dir().join(filename)
}

/// Get the full path to a thumbnail by original filename.
pub fn get_thumbnail_path(filename: &str) -> PathBuf {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    thumbnails_dir().join(format!("{}_thumb.jpg", stem))
}

/// Save raw image bytes to the originals directory and create a thumbnail.
/// Returns the filename used.
pub fn save_image_from_bytes(bytes: &[u8], filename: &str) -> Result<()> {
    let orig_path = get_image_path(filename);
    std::fs::write(&orig_path, bytes)
        .with_context(|| format!("Failed to write image to {}", orig_path.display()))?;

    create_thumbnail(&orig_path, filename)?;
    Ok(())
}

/// Create a 256px thumbnail from an original image file.
pub fn create_thumbnail(original_path: &Path, filename: &str) -> Result<()> {
    let img = image::open(original_path)
        .with_context(|| format!("Failed to open image {}", original_path.display()))?;

    let thumb = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let thumb_path = get_thumbnail_path(filename);

    thumb
        .save(&thumb_path)
        .with_context(|| format!("Failed to save thumbnail to {}", thumb_path.display()))?;

    Ok(())
}

/// Delete both original and thumbnail files for an image.
pub fn delete_image_files(filename: &str) -> Result<()> {
    let orig = get_image_path(filename);
    if orig.exists() {
        std::fs::remove_file(&orig)
            .with_context(|| format!("Failed to delete image {}", orig.display()))?;
    }

    let thumb = get_thumbnail_path(filename);
    if thumb.exists() {
        std::fs::remove_file(&thumb)
            .with_context(|| format!("Failed to delete thumbnail {}", thumb.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename_format() {
        let name = generate_filename();
        // Format: YYYY-MM-DD_HH-MM-SS_xxxxxxxx.png
        assert!(name.ends_with(".png"));
        assert_eq!(name.len(), 32); // 10 date + 1 _ + 8 time + 1 _ + 8 uuid + 4 .png
    }

    #[test]
    fn test_get_thumbnail_path() {
        let thumb = get_thumbnail_path("2026-01-15_12-30-45_abc12345.png");
        let filename = thumb.file_name().unwrap().to_str().unwrap();
        assert_eq!(filename, "2026-01-15_12-30-45_abc12345_thumb.jpg");
    }

    #[test]
    fn test_get_image_path() {
        let path = get_image_path("test.png");
        assert!(path.to_str().unwrap().contains("originals"));
        assert!(path.to_str().unwrap().ends_with("test.png"));
    }

    #[test]
    fn test_save_and_thumbnail() {
        // Create a small test image in memory
        let img = image::RgbImage::new(64, 64);
        let mut bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
        image::ImageEncoder::write_image(
            encoder,
            img.as_raw(),
            64,
            64,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();

        // Use a temp dir to avoid polluting real data dir
        let tmp = tempfile::tempdir().unwrap();
        let orig_path = tmp.path().join("test.png");
        std::fs::write(&orig_path, &bytes).unwrap();

        let thumb_path = tmp.path().join("test_thumb.jpg");
        let img_loaded = image::open(&orig_path).unwrap();
        let thumb = img_loaded.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
        thumb.save(&thumb_path).unwrap();

        assert!(thumb_path.exists());
    }
}
