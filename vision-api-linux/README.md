# vision-api-linux

Linux-compatible companion to [`vision-api-macos`](../vision-api-macos/README.md), written in **Rust**. Same HTTP paths and JSON shapes for OCR and barcode detection; macOS-only Vision features return **501 Not Implemented**.

Runs on **CPU only** (no GPU required).

## Implemented endpoints

| Method | Path | Backend |
|--------|------|---------|
| `POST` | `/text-detection/recognize-text` | [Tesseract](https://github.com/tesseract-ocr/tesseract) |
| `POST` | `/barcode-detection/detect` | [rxing](https://github.com/rxing-rust/rxing) |
| `GET` | `/barcode-detection/symbologies` | Static list (Vision-compatible names) |
| `GET` | `/health` | Service info |

**Not on Linux** (501): `/image-classification/*`, `/image-feature/*` (Apple Vision only — use macOS build).

Default port: **9493** (`PORT` env overrides).

## Prerequisites

Debian/Ubuntu:

```bash
sudo apt-get update
sudo apt-get install -y tesseract-ocr tesseract-ocr-eng tesseract-ocr-fra
```

macOS (local `cargo build` / `npm run build:linux`):

```bash
brew install tesseract tesseract-lang
```

Fedora:

```bash
sudo dnf install tesseract tesseract-langpack-eng tesseract-langpack-fra
```

OCR calls the **`tesseract` CLI** at runtime (no libtesseract at link time).

Install [Rust](https://rustup.rs/), then:

```bash
cd vision-api-linux
cargo build --release
PORT=9493 ./target/release/vision-api-linux
```

## Try it (same curl as macOS)

```bash
curl -X POST http://localhost:9493/text-detection/recognize-text \
  -F "imageFile=@/path/to/receipt.jpg" \
  -F "recognitionLanguages=fr-FR"

curl -X POST http://localhost:9493/barcode-detection/detect \
  -F "imageFile=@/path/to/qr.png"
```

Optional OCR fields (match macOS API):

- `recognitionLanguages` — e.g. `fr-FR`, `en-US`, or `fra+eng`
- `recognitionLevel` — `0` = default, `1` = faster (lighter Tesseract settings)

## Docker

```bash
docker build -t vision-api-linux .
docker run --rm -p 9493:9493 vision-api-linux
```

## Notes

- OCR quality differs from Apple Vision; receipts often need `recognitionLanguages=fr-FR` and good lighting.
- Barcode symbology strings mirror Vision names (e.g. `VNBarcodeSymbologyQR`) for easier client reuse.
- For production, put the service behind a reverse proxy and do not expose it publicly without auth.
