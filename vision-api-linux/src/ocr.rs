use std::process::Command;

use axum::extract::Multipart;
use serde::Serialize;

use crate::error::AppError;

#[derive(Serialize)]
pub struct RecognizeTextResponse {
    pub text: String,
}

pub async fn recognize_text(mut multipart: Multipart) -> Result<String, AppError> {
    let form = parse_multipart(&mut multipart).await?;
    let languages = map_languages(form.recognition_languages.as_deref());
    let fast = form.recognition_level == Some(1);
    let image_bytes = form.image_bytes;

    let text = tokio::task::spawn_blocking(move || run_tesseract_cli(&image_bytes, &languages, fast))
        .await
        .map_err(|e| AppError::Internal(format!("ocr task failed: {e}")))?
        .map_err(|e| AppError::Internal(e))?;

    Ok(text)
}

/// Runs the system `tesseract` binary (no libtesseract link-time dependency).
fn run_tesseract_cli(image: &[u8], languages: &str, fast: bool) -> Result<String, String> {
    let ext = image_file_extension(image);
    let temp_dir = std::env::temp_dir();
    let image_path = temp_dir.join(format!(
        "vision-api-ocr-{}-{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        ext
    ));

    std::fs::write(&image_path, image).map_err(|e| format!("write temp image: {e}"))?;

    let result = (|| {
        let mut cmd = Command::new("tesseract");
        cmd.arg(&image_path)
            .arg("stdout")
            .arg("-l")
            .arg(languages);

        if fast {
            cmd.arg("--oem").arg("1");
        }

        let output = cmd
            .output()
            .map_err(|e| format!("failed to run tesseract (install tesseract-ocr / brew install tesseract): {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tesseract exited with {}: {stderr}", output.status));
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
    })();

    let _ = std::fs::remove_file(&image_path);
    result
}

fn image_file_extension(image: &[u8]) -> &'static str {
    if image.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if image.starts_with(b"\x89PNG\r\n\x1a\n") {
        "png"
    } else if image.starts_with(b"GIF87a") || image.starts_with(b"GIF89a") {
        "gif"
    } else if image.len() > 12 && &image[0..4] == b"RIFF" && &image[8..12] == b"WEBP" {
        "webp"
    } else if image.starts_with(b"BM") {
        "bmp"
    } else if image.starts_with(b"II*\0") || image.starts_with(b"MM\0*") {
        "tiff"
    } else {
        "png"
    }
}

/// Maps BCP-47 style codes (e.g. `fr-FR`, `en-US`) to Tesseract language codes.
fn map_languages(raw: Option<&str>) -> String {
    let Some(raw) = raw else {
        return "eng".to_string();
    };

    let codes: Vec<String> = raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|tag| map_language_tag(tag))
        .collect();

    if codes.is_empty() {
        "eng".into()
    } else {
        codes.join("+")
    }
}

fn map_language_tag(tag: &str) -> String {
    let lower = tag.to_lowercase();
    match lower.as_str() {
        "fr" | "fr-fr" | "fra" => "fra".into(),
        "en" | "en-us" | "eng" => "eng".into(),
        "de" | "de-de" | "deu" => "deu".into(),
        "es" | "es-es" | "spa" => "spa".into(),
        "it" | "it-it" | "ita" => "ita".into(),
        "pt" | "pt-pt" | "por" => "por".into(),
        "nl" | "nl-nl" | "nld" => "nld".into(),
        "zh-hans" | "chi_sim" => "chi_sim".into(),
        "zh-hant" | "chi_tra" => "chi_tra".into(),
        "ja" | "ja-jp" | "jpn" => "jpn".into(),
        other if other.len() == 3 => other.to_string(),
        other => match other.split('-').next().unwrap_or("eng") {
            "fr" => "fra".into(),
            "en" => "eng".into(),
            "de" => "deu".into(),
            "es" => "spa".into(),
            _ => "eng".into(),
        },
    }
}

struct MultipartForm {
    image_bytes: Vec<u8>,
    recognition_languages: Option<String>,
    recognition_level: Option<i32>,
}

async fn parse_multipart(multipart: &mut Multipart) -> Result<MultipartForm, AppError> {
    let mut image_bytes: Option<Vec<u8>> = None;
    let mut recognition_languages: Option<String> = None;
    let mut recognition_level: Option<i32> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "imageFile" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("read imageFile: {e}")))?;
                image_bytes = Some(data.to_vec());
            }
            "recognitionLanguages" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("read recognitionLanguages: {e}")))?;
                if !text.is_empty() {
                    recognition_languages = Some(text);
                }
            }
            "recognitionLevel" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("read recognitionLevel: {e}")))?;
                recognition_level = text.trim().parse().ok();
            }
            _ => {}
        }
    }

    let image_bytes =
        image_bytes.ok_or_else(|| AppError::BadRequest("missing form field: imageFile".into()))?;

    Ok(MultipartForm {
        image_bytes,
        recognition_languages,
        recognition_level,
    })
}
