# vision-api-macos

macOS server using **Swift**, **Vapor**, and Apple **Vision**. Full feature set: OCR, barcodes, image classification, background removal, aesthetics scoring.

Requires **macOS 15+** (see `Package.swift`).

## Quick start

```bash
cd vision-api-macos
swift package resolve
swift run App
# → http://localhost:9493
```

Release build:

```bash
swift build -c release
.build/release/App
```

Port: default **9493**; override with `PORT=9493` or `swift run App serve --port 9493`.

## API

| Feature | Method & path |
|---------|----------------|
| OCR | `POST /text-detection/recognize-text` |
| Barcodes | `POST /barcode-detection/detect` |
| Barcode symbologies | `GET /barcode-detection/symbologies` |
| Classification | `POST /image-classification/classify` |
| Supported labels | `GET /image-classification/supported-identifiers` |
| Background removal | `POST /image-feature/background-removal` |
| Aesthetics | `POST /image-feature/aesthetics-scoring` |

Swagger UI: `http://localhost:9493/Swagger/index.html`

All image endpoints use `multipart/form-data` with field **`imageFile`**.

Example:

```bash
curl -X POST http://localhost:9493/text-detection/recognize-text \
  -F "imageFile=@/path/to/image.png" \
  -F "recognitionLanguages=fr-FR" \
  -F "recognitionLevel=1"
```

Optional OCR fields: `recognitionLanguages` (comma-separated BCP-47), `recognitionLevel` (`0` accurate, `1` fast).

## Linux alternative

CPU-only OCR + barcodes on Linux: [`../vision-api-linux/`](../vision-api-linux/README.md).

## Legal

Uses Apple Vision on macOS. See the [root legal notice](../README.md#-legal-notice) before commercial or hosted use.
