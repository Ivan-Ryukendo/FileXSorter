# Development Plan

## Overview

**FileXSorter** is a lightweight Windows GUI application for detecting duplicate files using SHA-256 hash verification. Built with Rust and egui for optimal performance and minimal binary size.

## Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust | Memory safety, performance, small binaries |
| GUI | eframe/egui | Immediate-mode, cross-platform, fast |
| Hashing | SHA-256 | Cryptographic strength, collision resistance |
| Threading | Rayon | Safe parallel processing |

---

## Completed Versions

| Version | Key Features |
|---------|--------------|
| v0.3.2 | Fixed status bar anchoring, proper panel-based layout |
| v0.3.1 | Preview panel fixes, file selection in Explorer |
| v0.3.0 | Image preview, file type icons, media detection |
| v0.2.0 | Multi-folder scanning, file preview panel, space analysis |
| v0.1.0 | Core duplicate detection, delete/move operations |

---

## Future Development Phases

### Phase A: AI-Powered Duplicate Detection (Priority: High)

Transform duplicate detection from exact-match to intelligent similarity detection using AI/ML techniques.

#### A1. Perceptual Image Hashing
- **Goal**: Detect visually similar images even with different resolutions, crops, or compression
- **Implementation**: 
  - Integrate `image_hasher` crate for pHash/dHash algorithms
  - Compare perceptual hash distance (Hamming distance threshold)
  - UI: Similarity slider (0-100%) for match sensitivity
- **Use Cases**: Find resized copies, screenshots of same content, re-encoded images

#### A2. Lightweight Local AI Models
- **Goal**: On-device image similarity using efficient neural networks
- **Implementation Options**:
  - **ONNX Runtime**: Run pre-trained models (MobileNet, EfficientNet-Lite) for feature extraction
  - **Tract**: Pure Rust inference engine, no Python dependency
  - Model options: CLIP embeddings, ResNet feature vectors
- **Workflow**: 
  1. Extract feature vectors from images
  2. Store in local vector database
  3. Find nearest neighbors for similarity matching
- **Performance Target**: <100ms per image on CPU

#### A3. Cloud AI API Integration (Optional)
- **Goal**: Advanced similarity detection via external APIs for users who prefer accuracy over privacy
- **Supported APIs**:
  - Google Cloud Vision API
  - Azure Computer Vision
  - OpenAI CLIP API
- **Features**:
  - API key configuration in settings
  - Batch processing with rate limiting
  - Fallback to local detection if API unavailable
- **Privacy**: Opt-in only, clear data handling disclosure

#### A4. Audio Fingerprinting
- **Goal**: Detect duplicate audio files regardless of format, bitrate, or metadata
- **Implementation**:
  - Chromaprint/AcoustID integration
  - FFT-based audio fingerprinting
- **Use Cases**: Find MP3/FLAC duplicates, different encodings of same song

---

### Phase B: Database & Tagging System (Priority: High)

Add persistent storage for file metadata, tags, and scan history.

#### B1. SQLite Database Integration
- **Goal**: Persistent local storage for all file metadata
- **Schema Design**:
  ```sql
  files (id, path, name, size, hash, created_at, modified_at)
  tags (id, name, color, icon)
  file_tags (file_id, tag_id)
  scans (id, folders, timestamp, duplicate_count, wasted_space)
  scan_results (scan_id, file_id, group_id)
  ```
- **Benefits**: 
  - Instant repeat scans (hash caching)
  - Persistent tag assignments
  - Scan history and statistics

#### B2. File Tagging System
- **Goal**: Organize files with custom tags for better management
- **Features**:
  - Create/edit/delete custom tags with colors
  - Bulk tagging: Select multiple files, apply tags
  - Tag-based filtering in results view
  - Auto-tag rules: "All .psd files get 'Design' tag"
- **UI**: Tag chips in file list, tag filter sidebar

#### B3. Smart Collections
- **Goal**: Dynamic file groups based on rules
- **Examples**:
  - "Large duplicates (>100MB)"
  - "Old files not accessed in 1 year"
  - "Photos from 2023"
- **Implementation**: Saved filter presets with live updating

#### B4. Hash Caching for Fast Rescans
- **Goal**: Skip re-hashing unchanged files
- **Implementation**:
  - Store file path + size + modified_date + hash
  - On rescan, only hash files with changed metadata
- **Performance**: 10x+ speedup on repeat scans

---

### Phase C: Advanced Duplicate Detection (Priority: Medium)

#### C1. Fuzzy Filename Matching
- **Goal**: Detect files with similar names
- **Examples**: "photo1.jpg" vs "photo_1.jpg" vs "photo (1).jpg"
- **Algorithm**: Levenshtein distance, configurable threshold

#### C2. Cross-Extension Detection
- **Goal**: Find same content in different formats
- **Examples**: "document.doc" vs "document.docx" vs "document.pdf"
- **Implementation**: Content-based comparison for known format pairs

#### C3. Duplicate Age Priority
- **Goal**: Smart suggestions for which duplicate to keep
- **Options**: Keep oldest, keep newest, keep largest, keep from preferred folder
- **UI**: Highlight recommended file with reason

---

### Phase D: User Experience Enhancements (Priority: Medium)

#### D1. Dark/Light Theme Toggle
- **Implementation**: egui theme switching, persist preference
- **Themes**: System default, Dark, Light, High Contrast

#### D2. Keyboard Shortcuts
- **Shortcuts**:
  - `Ctrl+O`: Add folder
  - `Ctrl+S`: Start scan
  - `Delete`: Delete selected
  - `Space`: Toggle selection
  - `P`: Toggle preview panel

#### D3. Drag & Drop Support
- **Feature**: Drop folders directly onto window to add to scan
- **Implementation**: eframe drag-drop handling

#### D4. Scan History & Statistics
- **Features**:
  - List of previous scans with date, folders, results
  - Statistics: Total space recovered, files deleted
  - Charts: Duplicate trends over time

#### D5. Undo/Recycle Bin Integration
- **Feature**: Move deleted files to Recycle Bin instead of permanent delete
- **Option**: User toggle between permanent delete and recycle

---

### Phase E: Performance & Scale (Priority: Medium)

#### E1. Pause/Resume Scanning
- **Feature**: Interrupt long scans, resume later
- **Implementation**: Serialize scan state to disk

#### E2. Exclusion Patterns
- **Feature**: Ignore specific folders, file types, or patterns
- **Examples**: Ignore `.git`, `node_modules`, `*.tmp`
- **UI**: Pattern list in settings

#### E3. Network Drive Support
- **Feature**: Scan mapped network drives and UNC paths
- **Challenges**: Handle timeouts, offline drives gracefully

#### E4. Memory-Efficient Mode
- **Feature**: Stream processing for directories with millions of files
- **Implementation**: Chunked processing, disk-based intermediate storage

---

### Phase F: Export & Reporting (Priority: Low)

#### F1. CSV/JSON Export
- **Feature**: Export duplicate list for external processing
- **Fields**: Path, size, hash, group ID, tags

#### F2. HTML Report Generation
- **Feature**: Shareable visual report with charts
- **Includes**: Summary stats, duplicate groups, space analysis

#### F3. Statistics Dashboard
- **Feature**: Visual charts in-app
- **Charts**: Storage by type, duplicate count by folder, trends

---

### Phase G: Integration & Automation (Priority: Low)

#### G1. Command-Line Interface
- **Feature**: CLI mode for scripting
- **Commands**: `filexsorter scan <path>`, `filexsorter delete-dupes`

#### G2. Windows Context Menu
- **Feature**: Right-click folder -> "Scan for Duplicates"
- **Implementation**: Windows registry integration

#### G3. Scheduled Scans
- **Feature**: Automatic periodic scanning
- **Implementation**: Windows Task Scheduler integration

#### G4. System Tray Mode
- **Feature**: Run in background, notify on new duplicates
- **Implementation**: Minimize to system tray, folder watching

---

## System Requirements

| Requirement | Specification |
|-------------|---------------|
| OS | Windows 10/11 (64-bit) |
| RAM | 4 GB minimum, 8 GB recommended |
| Disk | ~5 MB for executable |
| Runtime | None (standalone) |

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| eframe | 0.29 | GUI framework |
| egui | 0.29 | UI library |
| image | 0.25 | Image processing |
| sha2 | 0.10 | SHA-256 hashing |
| rayon | 1.10 | Parallel processing |
| walkdir | 2.5 | Directory traversal |
| rfd | 0.15 | File dialogs |
| rusqlite | TBD | Database (planned) |
| ort | TBD | ONNX Runtime (planned) |
