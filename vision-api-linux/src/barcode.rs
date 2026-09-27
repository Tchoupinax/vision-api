use std::collections::HashSet;

use axum::extract::Multipart;
use rxing::common::HybridBinarizer;
use rxing::multi::GenericMultipleBarcodeReader;
use rxing::multi::MultipleBarcodeReader;
use rxing::BarcodeFormat;
use rxing::BinaryBitmap;
use rxing::DecodeHints;
use rxing::Luma8LuminanceSource;
use rxing::MultiFormatReader;
use serde::Serialize;

use crate::error::AppError;

#[derive(Serialize)]
pub struct BarcodeResult {
    pub payload: String,
    pub symbology: String,
}

pub async fn detect_barcodes(mut multipart: Multipart) -> Result<Vec<BarcodeResult>, AppError> {
    let (image_bytes, symbologies) = parse_barcode_multipart(&mut multipart).await?;

    let results = tokio::task::spawn_blocking(move || decode_barcodes(&image_bytes, symbologies))
        .await
        .map_err(|e| AppError::Internal(format!("barcode task failed: {e}")))?
        .map_err(AppError::Internal)?;

    Ok(results)
}

fn decode_barcodes(
    image_bytes: &[u8],
    symbology_filter: Option<Vec<BarcodeFormat>>,
) -> Result<Vec<BarcodeResult>, String> {
    let img = image::load_from_memory(image_bytes).map_err(|e| format!("decode image: {e}"))?;
    let luma = img.to_luma8();
    let (width, height) = luma.dimensions();

    let source = Luma8LuminanceSource::new(luma.into_raw(), width, height);
    let mut bitmap = BinaryBitmap::new(HybridBinarizer::new(source));

    let mut hints = DecodeHints::default();
    hints.TryHarder = Some(true);
    if let Some(formats) = symbology_filter {
        hints.PossibleFormats = Some(formats.into_iter().collect::<HashSet<_>>());
    }

    let reader = MultiFormatReader::default();
    let mut multi = GenericMultipleBarcodeReader::new(reader);

    let decoded = multi
        .decode_multiple_with_hints(&mut bitmap, &hints)
        .unwrap_or_default();

    Ok(decoded
        .into_iter()
        .map(|result| BarcodeResult {
            payload: result.getText().to_string(),
            symbology: vision_symbology_name(*result.getBarcodeFormat()),
        })
        .collect())
}

fn vision_symbology_name(format: BarcodeFormat) -> String {
    match format {
        BarcodeFormat::QR_CODE => "VNBarcodeSymbologyQR".into(),
        BarcodeFormat::EAN_13 => "VNBarcodeSymbologyEAN13".into(),
        BarcodeFormat::EAN_8 => "VNBarcodeSymbologyEAN8".into(),
        BarcodeFormat::UPC_E => "VNBarcodeSymbologyUPCE".into(),
        BarcodeFormat::CODE_128 => "VNBarcodeSymbologyCode128".into(),
        BarcodeFormat::CODE_39 => "VNBarcodeSymbologyCode39".into(),
        BarcodeFormat::CODE_93 => "VNBarcodeSymbologyCode93".into(),
        BarcodeFormat::ITF => "VNBarcodeSymbologyITF14".into(),
        BarcodeFormat::PDF_417 => "VNBarcodeSymbologyPDF417".into(),
        BarcodeFormat::DATA_MATRIX => "VNBarcodeSymbologyDataMatrix".into(),
        BarcodeFormat::AZTEC => "VNBarcodeSymbologyAztec".into(),
        BarcodeFormat::CODABAR => "VNBarcodeSymbologyCodabar".into(),
        other => format!("RXing_{other:?}"),
    }
}

pub fn supported_symbologies() -> Vec<String> {
    vec![
        "VNBarcodeSymbologyQR".into(),
        "VNBarcodeSymbologyEAN13".into(),
        "VNBarcodeSymbologyEAN8".into(),
        "VNBarcodeSymbologyUPCE".into(),
        "VNBarcodeSymbologyCode128".into(),
        "VNBarcodeSymbologyCode39".into(),
        "VNBarcodeSymbologyCode93".into(),
        "VNBarcodeSymbologyITF14".into(),
        "VNBarcodeSymbologyPDF417".into(),
        "VNBarcodeSymbologyDataMatrix".into(),
        "VNBarcodeSymbologyAztec".into(),
        "VNBarcodeSymbologyCodabar".into(),
    ]
}

fn parse_symbology_token(token: &str) -> Option<BarcodeFormat> {
    let t = token.trim();
    match t {
        "VNBarcodeSymbologyQR" | "QR" => Some(BarcodeFormat::QR_CODE),
        "VNBarcodeSymbologyEAN13" | "EAN13" => Some(BarcodeFormat::EAN_13),
        "VNBarcodeSymbologyEAN8" | "EAN8" => Some(BarcodeFormat::EAN_8),
        "VNBarcodeSymbologyUPCE" | "UPCE" => Some(BarcodeFormat::UPC_E),
        "VNBarcodeSymbologyCode128" | "Code128" => Some(BarcodeFormat::CODE_128),
        "VNBarcodeSymbologyCode39" | "Code39" => Some(BarcodeFormat::CODE_39),
        "VNBarcodeSymbologyCode93" | "Code93" => Some(BarcodeFormat::CODE_93),
        "VNBarcodeSymbologyITF14" | "ITF14" => Some(BarcodeFormat::ITF),
        "VNBarcodeSymbologyPDF417" | "PDF417" => Some(BarcodeFormat::PDF_417),
        "VNBarcodeSymbologyDataMatrix" | "DataMatrix" => Some(BarcodeFormat::DATA_MATRIX),
        "VNBarcodeSymbologyAztec" | "Aztec" => Some(BarcodeFormat::AZTEC),
        "VNBarcodeSymbologyCodabar" | "Codabar" => Some(BarcodeFormat::CODABAR),
        _ => None,
    }
}

async fn parse_barcode_multipart(
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, Option<Vec<BarcodeFormat>>), AppError> {
    let mut image_bytes: Option<Vec<u8>> = None;
    let mut symbologies: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        match field.name().unwrap_or("") {
            "imageFile" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("read imageFile: {e}")))?;
                image_bytes = Some(data.to_vec());
            }
            "symbologies" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("read symbologies: {e}")))?;
                if !text.is_empty() {
                    symbologies = Some(text);
                }
            }
            _ => {}
        }
    }

    let image_bytes =
        image_bytes.ok_or_else(|| AppError::BadRequest("missing form field: imageFile".into()))?;

    let filter = symbologies.map(|s| {
        s.split(',')
            .filter_map(|part| parse_symbology_token(part))
            .collect::<Vec<_>>()
    });

    let filter = filter.filter(|v| !v.is_empty());

    Ok((image_bytes, filter))
}
