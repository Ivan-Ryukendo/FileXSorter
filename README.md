# FileXSorter

![Version](https://img.shields.io/badge/version-0.3.2-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![License](https://img.shields.io/badge/license-MIT-green)

A lightweight, fast Windows application for detecting and managing duplicate files using SHA-256 hash verification with a clean graphical interface.

## Features

### Current Features
- **Fast Duplicate Detection** - Two-stage detection (size pre-filter + SHA-256 hash)
- **Multi-threaded Scanning** - Parallel file hashing using Rayon
- **Multi-Folder Scanning** - Scan multiple directories in one session
- **Image Preview** - View PNG, JPG, GIF, BMP, WEBP images directly
- **File Type Icons** - Visual indicators for images, video, audio, text
- **Delete/Move Duplicates** - Remove or relocate selected files
- **Open in Explorer** - Quick access to file locations with file selection
- **Standalone Executable** - No runtime dependencies, single ~4.5 MB exe

## Screenshots

![Main Interface](docs/images/main.png)

## Installation

### Download
1. Download `FileXSorter.exe` from [Releases](../../releases)
2. Run it - no installation required

### Build from Source
```bash
git clone https://github.com/Ivan-Ryukendo/FileXSorter.git
cd FileXSorter
cargo build --release
```

## Usage

1. **Add Folders** - Click "Add" to select directories
2. **Configure** - Toggle "Subfolders" as needed
3. **Scan** - Click "Scan" to find duplicates
4. **Preview** - Click the eye icon on any file
5. **Select** - Check duplicates to remove (first file marked [KEEP])
6. **Action** - Delete or Move selected files

---

## Planned Features

### High Priority

- [ ] **Perceptual Image Hashing** - Find visually similar images (resized, cropped, re-encoded)
- [ ] **SQLite Database** - Persistent storage for metadata, tags, and scan history
- [ ] **File Tagging System** - Organize files with custom color-coded tags
- [ ] **Hash Caching** - 10x faster repeat scans by caching file hashes

### AI-Powered Detection

- [ ] **Local AI Models** - On-device neural networks for image similarity (ONNX/Tract)
- [ ] **Cloud AI Integration** - Optional Google Vision, Azure, OpenAI CLIP support
- [ ] **Audio Fingerprinting** - Detect audio duplicates regardless of format

### Advanced Detection

- [ ] **Fuzzy Filename Matching** - "photo1.jpg" vs "photo_1.jpg" detection
- [ ] **Cross-Extension Detection** - Find same content in different formats
- [ ] **Duplicate Age Priority** - Smart suggestions for which file to keep

### User Experience

- [ ] **Dark/Light Themes** - User-selectable color schemes
- [ ] **Keyboard Shortcuts** - Power-user navigation (Ctrl+O, Delete, Space)
- [ ] **Drag & Drop** - Drop folders onto window to scan
- [ ] **Scan History** - Track previous scans and space recovered
- [ ] **Undo/Recycle Bin** - Move to Recycle Bin instead of permanent delete

### Performance

- [ ] **Pause/Resume Scanning** - Interrupt and continue long scans
- [ ] **Exclusion Patterns** - Ignore .git, node_modules, *.tmp
- [ ] **Network Drive Support** - Scan mapped drives and UNC paths

### Export & Integration

- [ ] **CSV/JSON Export** - Export duplicate lists for external processing
- [ ] **HTML Reports** - Shareable visual reports with charts
- [ ] **Command-Line Interface** - CLI mode for scripting
- [ ] **Windows Context Menu** - Right-click "Scan for Duplicates"
- [ ] **System Tray Mode** - Background monitoring with notifications

---

## Changelog

### v0.3.2
- **Resizable Preview Panel** - Drag the edge to resize preview panel width (150-400px)
- **Fixed Preview Panel** - Now always anchored to right side using proper SidePanel
- **Fixed Status Bar** - Permanently anchored at bottom, never overlaps content
- **Full-Height File List** - Uses all available vertical space, no wasted black area
- **Responsive Layout** - See more files on larger monitors
- **Scalable Image Preview** - Images scale with panel size for better viewing

### v0.3.1
- Fixed preview panel layout (compact, max 25% width)
- Fixed "Open" button to select file in Explorer
- Added "Open with default app" button

### v0.3.0
- Image preview (PNG, JPG, GIF, BMP, WEBP)
- File type icons in duplicate list
- Video/audio file indicators

### v0.2.0
- Multi-folder scanning
- File preview panel
- Text content preview
- Space analysis

### v0.1.0
- Initial release
- SHA-256 duplicate detection
- Delete and move operations

## Requirements

- Windows 10/11 (64-bit)
- ~5 MB disk space

## Project Structure

```
FileXSorter/
├── src/
│   ├── main.rs       # Entry point
│   ├── app.rs        # GUI application
│   ├── scanner.rs    # Duplicate detection
│   └── file_ops.rs   # File operations
├── Cargo.toml
├── plan.md           # Development plan
└── roadmap.md        # Feature roadmap
```

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/Name`)
3. Commit changes
4. Push and open a Pull Request

See [roadmap.md](roadmap.md) for planned features!

## License

MIT License - see LICENSE file.

---

Made with Rust | [Report Bug](../../issues) | [Request Feature](../../issues/new)
