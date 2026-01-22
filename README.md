# File X Sorter

![Version](https://img.shields.io/badge/version-0.3.0-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![License](https://img.shields.io/badge/license-MIT-green)

A lightweight, fast Windows application for detecting and managing duplicate files using SHA-256 hash verification with a clean graphical interface.

## Features

### Core Features
- **Fast Duplicate Detection** - Two-stage detection (size pre-filter + SHA-256 hash)
- **Multi-threaded Scanning** - Parallel file hashing using Rayon
- **Multi-Folder Scanning** - Scan multiple directories in one session
- **Recursive/Non-recursive Scans** - Toggle subfolder scanning
- **Standalone Executable** - No runtime dependencies, single ~4.5 MB exe

### File Preview (New in v0.3.0)
- **Image Preview** - View PNG, JPG, GIF, BMP, WEBP images directly
- **Video Info** - Display video file type and format info
- **Audio Info** - Display audio file type and format info
- **Text Preview** - Preview contents of text/code files
- **File Icons** - Visual file type indicators in the list

### File Management
- **Delete Duplicates** - Remove selected files with confirmation
- **Move Duplicates** - Relocate files to a specified folder
- **Open in Explorer** - Quick access to file locations
- **Smart Selection** - Select all duplicates with one click

## Screenshots

![Main Interface](docs/images/main.png)

## Installation

### Download the Executable

1. Download the latest `File X Sorter.exe` from [Releases](../../releases)
2. Place it anywhere on your computer
3. Double-click to run - no installation required

### Build from Source

```bash
git clone https://github.com/Ivan-Ryukendo/FileXSorter.git
cd FileXSorter
cargo build --release
# Output: target/release/file-x-sorter.exe
```

## Usage

1. **Add Folders**: Click "Add Folder" to select directories
2. **Configure**: Toggle "Scan subfolders" as needed
3. **Scan**: Click "Scan for Duplicates"
4. **Preview**: Click "Preview" on any file to see details
5. **Select**: Check duplicates to remove (first file marked [KEEP])
6. **Action**: Delete or Move selected files

### Supported Preview Types

| Type | Extensions | Preview |
|------|------------|---------|
| Images | PNG, JPG, GIF, BMP, WEBP, ICO | Full image display |
| Video | MP4, AVI, MKV, MOV, WMV | File info + icon |
| Audio | MP3, WAV, FLAC, AAC, OGG | File info + icon |
| Text | TXT, MD, JSON, code files | Content preview |

## Changelog

### v0.3.0
- Added image preview support (PNG, JPG, GIF, BMP, WEBP)
- Added video/audio file type indicators
- Added file type icons in duplicate list
- Improved UI layout with compact preview panel
- Better space utilization for file list

### v0.2.0
- Added multi-folder scanning
- Added file preview panel
- Added text file content preview
- Added space analysis by folder

### v0.1.0
- Initial release
- Duplicate detection with SHA-256
- Delete and move operations

## Requirements

- Windows 10 or Windows 11 (64-bit)
- ~4.5 MB disk space

## Development

### Project Structure

```
FileXSorter/
├── src/
│   ├── main.rs       # Entry point
│   ├── app.rs        # GUI application (egui)
│   ├── scanner.rs    # Duplicate detection
│   └── file_ops.rs   # File operations
├── Cargo.toml        # Dependencies
├── plan.md           # Development plan
└── roadmap.md        # Future features
```

### Key Dependencies

- `eframe`/`egui` - GUI framework
- `image` - Image loading and preview
- `sha2` - SHA-256 hashing
- `rayon` - Parallel processing
- `walkdir` - Directory traversal
- `rfd` - Native file dialogs

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/Name`)
3. Commit changes (`git commit -m 'Add feature'`)
4. Push to branch (`git push origin feature/Name`)
5. Open a Pull Request

## License

MIT License - see LICENSE file for details.

---

Made with Rust | [Report Bug](../../issues) | [Request Feature](../../issues/new)
