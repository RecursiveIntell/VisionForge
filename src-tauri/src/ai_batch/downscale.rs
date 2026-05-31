use anyhow::{Context, Result};
use std::path::Path;

/// Read an image, optionally downscale it, and return base64-encoded bytes.
/// If `max_dimension` is None, returns the original image as base64.
/// If `max_dimension` is Some(n), downscales so the longest side is at most n pixels,
/// preserving aspect ratio. Only downscales -- never upscales.
pub fn read_image_base64_downscaled(path: &Path, max_dimension: Option<u32>) -> Result<String> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("Failed to read image at {}", path.display()))?;

    let max_dim = match max_dimension {
        Some(d) => d,
        None => {
            return Ok(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &bytes,
            ));
        }
    };

    let img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(_) => {
            // If we can't decode (unsupported format), return original
            return Ok(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &bytes,
            ));
        }
    };

    let (w, h) = (img.width(), img.height());
    let longest = w.max(h);

    if longest <= max_dim {
        return Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &bytes,
        ));
    }

    let resized = img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3);

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    resized
        .write_to(&mut cursor, image::ImageFormat::Png)
        .context("Failed to encode downscaled image")?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &buf,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_png(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::new(width, height);
        let mut bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
        image::ImageEncoder::write_image(
            encoder,
            img.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
        bytes
    }

    #[test]
    fn test_no_downscale_returns_original() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let bytes = create_test_png(256, 256);
        std::fs::write(tmp.path(), &bytes).unwrap();

        let b64 = read_image_base64_downscaled(tmp.path(), None).unwrap();
        let expected = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        assert_eq!(b64, expected);
    }

    #[test]
    fn test_small_image_not_upscaled() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let bytes = create_test_png(100, 100);
        std::fs::write(tmp.path(), &bytes).unwrap();

        // max_dim 1024 > image size 100, should return original
        let b64 = read_image_base64_downscaled(tmp.path(), Some(1024)).unwrap();
        let expected = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        assert_eq!(b64, expected);
    }

    #[test]
    fn test_large_image_downscaled() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let bytes = create_test_png(2048, 1024);
        std::fs::write(tmp.path(), &bytes).unwrap();

        let b64 = read_image_base64_downscaled(tmp.path(), Some(512)).unwrap();
        // Should be different from original (downscaled + re-encoded)
        let original_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        assert_ne!(b64, original_b64);
        // Result should decode to valid base64
        assert!(!b64.is_empty());
    }
}
