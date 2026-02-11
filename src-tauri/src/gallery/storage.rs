use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::manager;
use crate::types::config::AppConfig;

const THUMBNAIL_SIZE: u32 = 256;

/// Generate a filename for a new image: YYYY-MM-DD_HH-MM-SS_<short_uuid>.png
pub fn generate_filename() -> String {
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S");
    let short_id = &uuid::Uuid::new_v4().to_string()[..8];
    format!("{}_{}.png", timestamp, short_id)
}

/// Get the path to the originals directory, using default data dir.
pub fn originals_dir() -> PathBuf {
    manager::data_dir().join("images/originals")
}

/// Get the path to the originals directory for a given config.
pub fn originals_dir_for(config: &AppConfig) -> PathBuf {
    manager::image_dir(config).join("originals")
}

/// Get the path to the thumbnails directory, using default data dir.
pub fn thumbnails_dir() -> PathBuf {
    manager::data_dir().join("images/thumbnails")
}

/// Get the path to the thumbnails directory for a given config.
pub fn thumbnails_dir_for(config: &AppConfig) -> PathBuf {
    manager::image_dir(config).join("thumbnails")
}

/// Get the full path to an original image by filename (default dir).
pub fn get_image_path(filename: &str) -> PathBuf {
    originals_dir().join(filename)
}

/// Get the full path to an original image by filename for a given config.
pub fn get_image_path_for(config: &AppConfig, filename: &str) -> PathBuf {
    originals_dir_for(config).join(filename)
}

/// Get the full path to a thumbnail by original filename (default dir).
pub fn get_thumbnail_path(filename: &str) -> PathBuf {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    thumbnails_dir().join(format!("{}_thumb.jpg", stem))
}

/// Get the full path to a thumbnail by original filename for a given config.
pub fn get_thumbnail_path_for(config: &AppConfig, filename: &str) -> PathBuf {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    thumbnails_dir_for(config).join(format!("{}_thumb.jpg", stem))
}

/// Save raw image bytes to the originals directory and create a thumbnail.
pub fn save_image_from_bytes(bytes: &[u8], filename: &str) -> Result<()> {
    save_image_from_bytes_for(bytes, filename, &originals_dir(), &thumbnails_dir())
}

/// Save raw image bytes using a specific config's directories.
pub fn save_image_from_bytes_with_config(
    config: &AppConfig,
    bytes: &[u8],
    filename: &str,
) -> Result<()> {
    let orig_dir = originals_dir_for(config);
    let thumb_dir = thumbnails_dir_for(config);
    save_image_from_bytes_for(bytes, filename, &orig_dir, &thumb_dir)
}

fn save_image_from_bytes_for(
    bytes: &[u8],
    filename: &str,
    orig_dir: &Path,
    thumb_dir: &Path,
) -> Result<()> {
    std::fs::create_dir_all(orig_dir)
        .with_context(|| format!("Failed to create originals dir {}", orig_dir.display()))?;
    std::fs::create_dir_all(thumb_dir)
        .with_context(|| format!("Failed to create thumbnails dir {}", thumb_dir.display()))?;

    let orig_path = orig_dir.join(filename);
    std::fs::write(&orig_path, bytes)
        .with_context(|| format!("Failed to write image to {}", orig_path.display()))?;

    create_thumbnail_to(&orig_path, filename, thumb_dir)?;
    Ok(())
}

/// Create a 256px thumbnail from an original image file.
pub fn create_thumbnail(original_path: &Path, filename: &str) -> Result<()> {
    create_thumbnail_to(original_path, filename, &thumbnails_dir())
}

fn create_thumbnail_to(original_path: &Path, filename: &str, thumb_dir: &Path) -> Result<()> {
    let img = image::open(original_path)
        .with_context(|| format!("Failed to open image {}", original_path.display()))?;

    let thumb = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let thumb_path = thumb_dir.join(format!("{}_thumb.jpg", stem));

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

/// Delete image files using config-aware paths. Falls back to default paths.
pub fn delete_image_files_for(config: &AppConfig, filename: &str) -> Result<()> {
    // Try config paths first
    let orig = get_image_path_for(config, filename);
    if orig.exists() {
        std::fs::remove_file(&orig)
            .with_context(|| format!("Failed to delete image {}", orig.display()))?;
    } else {
        // Fallback to default path
        let fallback = get_image_path(filename);
        if fallback.exists() {
            std::fs::remove_file(&fallback)
                .with_context(|| format!("Failed to delete image {}", fallback.display()))?;
        }
    }

    let thumb = get_thumbnail_path_for(config, filename);
    if thumb.exists() {
        std::fs::remove_file(&thumb)
            .with_context(|| format!("Failed to delete thumbnail {}", thumb.display()))?;
    } else {
        let fallback = get_thumbnail_path(filename);
        if fallback.exists() {
            std::fs::remove_file(&fallback)
                .with_context(|| format!("Failed to delete thumbnail {}", fallback.display()))?;
        }
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

    #[test]
    fn test_custom_image_dir() {
        let mut config = AppConfig::default();
        config.storage.image_directory = "/tmp/my-images".to_string();
        assert_eq!(
            originals_dir_for(&config),
            PathBuf::from("/tmp/my-images/originals")
        );
        assert_eq!(
            thumbnails_dir_for(&config),
            PathBuf::from("/tmp/my-images/thumbnails")
        );
    }

    #[test]
    fn test_empty_image_dir_uses_default() {
        let config = AppConfig::default();
        assert!(originals_dir_for(&config)
            .to_str()
            .unwrap()
            .contains(".visionforge"));
    }
}
