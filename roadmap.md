# Feature Roadmap

## Completed Features

| Version | Features |
|---------|----------|
| v0.3.2 | Status bar fixed to bottom, panel-based UI layout |
| v0.3.1 | Preview panel layout fix, file selection in Explorer, open with default app |
| v0.3.0 | Image preview (PNG/JPG/GIF/BMP/WEBP), file type icons, video/audio indicators |
| v0.2.0 | Multi-folder scanning, file preview panel, text content preview, space analysis |
| v0.1.0 | SHA-256 duplicate detection, delete/move operations, recursive scanning |

---

## Planned Features

### AI-Powered Detection (Phase A)

- [ ] **A1. Perceptual Image Hashing**
  - Find visually similar images using pHash/dHash algorithms
  - Configurable similarity threshold (0-100%)
  - Detect resized, cropped, or re-encoded duplicates

- [ ] **A2. Local AI Image Models**
  - On-device neural network inference (ONNX Runtime or Tract)
  - Feature extraction using MobileNet/EfficientNet-Lite
  - No cloud dependency, full privacy

- [ ] **A3. Cloud AI Integration (Optional)**
  - Support for Google Vision, Azure, OpenAI CLIP APIs
  - Opt-in with API key configuration
  - Batch processing with rate limiting

- [ ] **A4. Audio Fingerprinting**
  - Detect audio duplicates regardless of format/bitrate
  - Chromaprint/AcoustID integration
  - Find MP3/FLAC copies of same audio

---

### Database & Tagging (Phase B)

- [ ] **B1. SQLite Database**
  - Persistent storage for file metadata
  - Scan history with statistics
  - Fast query-based file lookup

- [ ] **B2. File Tagging System**
  - Create custom tags with colors
  - Bulk tagging for selected files
  - Tag-based filtering in results
  - Auto-tag rules by extension/folder

- [ ] **B3. Smart Collections**
  - Dynamic groups: "Large duplicates >100MB", "Old files"
  - Saved filter presets
  - Live updating results

- [ ] **B4. Hash Caching**
  - Skip re-hashing unchanged files
  - 10x+ speedup on repeat scans
  - Invalidation by modified date/size

---

### Advanced Detection (Phase C)

- [ ] **C1. Fuzzy Filename Matching**
  - Detect similar names: "photo1.jpg" vs "photo_1.jpg"
  - Levenshtein distance with configurable threshold

- [ ] **C2. Cross-Extension Detection**
  - Find same content in different formats
  - "document.doc" vs "document.docx"

- [ ] **C3. Duplicate Age Priority**
  - Suggest which duplicate to keep
  - Options: oldest, newest, largest, preferred folder

---

### User Experience (Phase D)

- [ ] **D1. Dark/Light Theme**
  - System default, Dark, Light themes
  - Persistent preference

- [ ] **D2. Keyboard Shortcuts**
  - Ctrl+O: Add folder | Ctrl+S: Scan
  - Delete: Remove selected | Space: Toggle selection

- [ ] **D3. Drag & Drop**
  - Drop folders onto window to add

- [ ] **D4. Scan History**
  - List of previous scans with results
  - Statistics: total space recovered

- [ ] **D5. Undo/Recycle Bin**
  - Move to Recycle Bin option
  - Recover accidentally deleted files

---

### Performance & Scale (Phase E)

- [ ] **E1. Pause/Resume Scanning**
  - Interrupt long scans, resume later
  - Save scan state to disk

- [ ] **E2. Exclusion Patterns**
  - Ignore folders: .git, node_modules
  - Ignore file types: *.tmp, *.log

- [ ] **E3. Network Drive Support**
  - Scan mapped drives and UNC paths
  - Handle timeouts gracefully

- [ ] **E4. Memory-Efficient Mode**
  - Stream processing for huge directories
  - Chunked processing, disk-based temp storage

---

### Export & Reporting (Phase F)

- [ ] **F1. CSV/JSON Export**
  - Export duplicate list with all metadata
  - Compatible with Excel, scripts

- [ ] **F2. HTML Report**
  - Shareable visual report
  - Summary, charts, duplicate groups

- [ ] **F3. Statistics Dashboard**
  - In-app charts and graphs
  - Storage by type, trends over time

---

### Integration & Automation (Phase G)

- [ ] **G1. Command-Line Interface**
  - CLI mode for scripting
  - `filexsorter scan`, `filexsorter delete-dupes`

- [ ] **G2. Windows Context Menu**
  - Right-click folder -> "Scan for Duplicates"
  - Registry integration

- [ ] **G3. Scheduled Scans**
  - Windows Task Scheduler integration
  - Automatic periodic scanning

- [ ] **G4. System Tray Mode**
  - Background folder monitoring
  - Notification on new duplicates

---

## Priority Matrix

| Priority | Features |
|----------|----------|
| **High** | A1 (Perceptual Hash), B1 (Database), B2 (Tagging), B4 (Hash Cache) |
| **Medium** | A2 (Local AI), C1 (Fuzzy Names), D1 (Themes), D2 (Shortcuts), E2 (Exclusions) |
| **Low** | A3 (Cloud AI), F1-F3 (Export), G1-G4 (Integration) |

## Contribution

Want to help implement a feature? Check the [GitHub Issues](../../issues) or submit a PR!
