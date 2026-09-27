use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::barcode::{detect_barcodes, supported_symbologies};
use crate::error::AppError;
use crate::ocr::{recognize_text, RecognizeTextResponse};

pub fn api_router() -> Router {
    Router::new()
        .route("/", get(redirect_root))
        .route("/health", get(health))
        .route(
            "/text-detection/recognize-text",
            post(recognize_text_handler),
        )
        .route("/barcode-detection/detect", post(detect_barcodes_handler))
        .route(
            "/barcode-detection/symbologies",
            get(symbologies_handler),
        )
        .route(
            "/image-classification/classify",
            post(image_classification_stub),
        )
        .route(
            "/image-classification/supported-identifiers",
            get(image_classification_identifiers_stub),
        )
        .route(
            "/image-feature/background-removal",
            post(image_feature_stub),
        )
        .route(
            "/image-feature/aesthetics-scoring",
            post(image_feature_stub),
        )
}

async fn redirect_root() -> Json<Health> {
    health().await
}

#[derive(Serialize)]
struct Health {
    service: &'static str,
    platform: &'static str,
    endpoints: &'static [&'static str],
}

async fn health() -> Json<Health> {
    Json(Health {
        service: "vision-api-linux",
        platform: "linux",
        endpoints: &[
            "POST /text-detection/recognize-text",
            "POST /barcode-detection/detect",
            "GET /barcode-detection/symbologies",
        ],
    })
}

async fn recognize_text_handler(
    multipart: axum::extract::Multipart,
) -> Result<Json<RecognizeTextResponse>, AppError> {
    let text = recognize_text(multipart).await?;
    Ok(Json(RecognizeTextResponse { text }))
}

async fn detect_barcodes_handler(
    multipart: axum::extract::Multipart,
) -> Result<Json<Vec<crate::barcode::BarcodeResult>>, AppError> {
    Ok(Json(detect_barcodes(multipart).await?))
}

async fn symbologies_handler() -> Json<Vec<String>> {
    Json(supported_symbologies())
}

async fn image_classification_stub() -> Result<(), AppError> {
    Err(AppError::NotImplemented(
        "Image classification uses Apple VNClassifyImageRequest on macOS. \
         On Linux, use a CLIP or ONNX model service, or call the macOS vision-api."
            .into(),
    ))
}

#[derive(Serialize)]
struct SupportedIdentifiersStub {
    identifiers: Vec<String>,
    count: usize,
    note: &'static str,
}

async fn image_classification_identifiers_stub() -> Json<SupportedIdentifiersStub> {
    Json(SupportedIdentifiersStub {
        identifiers: vec![],
        count: 0,
        note: "Not available on vision-api-linux; macOS Vision only.",
    })
}

async fn image_feature_stub() -> Result<(), AppError> {
    Err(AppError::NotImplemented(
        "Background removal and aesthetics scoring require Apple Vision (macOS 12+ / 15+). \
         On Linux, consider rembg or a custom model behind a separate service."
            .into(),
    ))
}
