# File X Sorter

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![License](https://img.shields.io/badge/license-MIT-green)

A lightweight, fast Windows application for detecting and managing duplicate files using SHA-256 hash verification with a clean graphical interface.

## Features

### Core Features
- **Fast Duplicate Detection** - Two-stage detection (size pre-filter + SHA-256 hash) for optimal performance
- **Multi-threaded Scanning** - Parallel file hashing using Rayon for speed
- **Recursive/Non-recursive Scans** - Toggle subfolder scanning as needed
- **Clean GUI** - Built with egui for a modern, responsive interface
- **Standalone Executable** - No runtime dependencies, single ~4 MB exe

### New in v0.2.0
- **Multi-Folder Scanning** - Add multiple directories to scan in one session
- **File Preview Panel** - View file details, size, and content preview before taking action
- **Space Analysis** - See which folders contain the most duplicate data
- **Text File Preview** - Preview contents of text files (txt, md, rs, py, js, etc.)
- **Improved Status Bar** - Shows selected file count and total size

### File Management
- **Delete Duplicates** - Remove selected duplicate files with confirmation
- **Move Duplicates** - Move files to a specified folder instead of deleting
- **Open in Explorer** - Quick access to file locations
- **Smart Selection** - Select all duplicates with one click (keeps first file)

## Screenshots

![Main Interface](docs/images/main.png)

## Installation

### Download the Executable

1. Download the latest `File X Sorter.exe` from the [Releases](../../releases) page
2. Place it anywhere on your computer
3. Double-click to run - no installation required

### Build from Source

If you have Rust installed, you can build from source:

```bash
# Clone the repository
git clone https://github.com/Ivan-Ryukendo/FileXSorter.git
cd FileXSorter

# Build release version
cargo build --release

# The executable will be at:
# target/release/file-x-sorter.exe
```

## Usage

### Basic Workflow

1. **Add Folders**: Click "Add Folder" to select directories to scan
   - Add multiple folders to compare files across different locations
   - Remove folders using the "X" button next to each path
2. **Configure Scan**: Toggle "Scan subfolders" on or off as needed
3. **Start Scan**: Click "Scan for Duplicates" to begin
4. **Review Results**: Duplicate groups are displayed, sorted by wasted space
5. **Preview Files**: Click "Preview" to see file details in the right panel
6. **Select Files**: Check the duplicates you want to remove (first file marked [KEEP])
7. **Take Action**: 
   - Click "Delete Selected" to permanently remove duplicates
   - Click "Move Selected" to move duplicates to another folder

### Preview Panel

The preview panel shows:
- File name, size, and type
- Last modified date
- Text content preview (for supported file types)
- Full file path
- Quick actions (Open File, Open Folder)

When no file is selected, it displays a breakdown of space usage by folder.

### Tips

- Use "Select All Duplicates" to quickly select all but one copy from each group
- Toggle "Show Preview" in the header to hide/show the preview panel
- Click "Open" next to any file to view it in Windows Explorer
- Cancel a long-running scan using the "Cancel" button
- The status bar shows total selected files and their combined size

## How It Works

File X Sorter uses a multi-stage detection algorithm:

1. **File Collection**: Walk through all selected directories (recursive optional)
2. **Size Filtering**: Group files by size - files with unique sizes cannot be duplicates
3. **Hash Computation**: SHA-256 hash computed in parallel for same-sized files
4. **Duplicate Grouping**: Files with identical hashes are grouped together
5. **Results Display**: Groups sorted by wasted space for easy prioritization

## Requirements

- Windows 10 or Windows 11 (64-bit)
- Approximately 4 MB disk space

## Changelog

### v0.2.0
- Added multi-folder scanning support
- Added file preview panel with size and content display
- Added space breakdown by folder
- Added text file content preview
- Improved status bar with selection info
- Updated UI layout with resizable panels

### v0.1.0
- Initial release
- Duplicate detection with SHA-256 hashing
- Delete and move operations
- Recursive scanning toggle

## Roadmap

Future planned features include:

- Tagging system for file organization
- SQLite database for cached scans and metadata
- Fuzzy name matching
- Image similarity detection
- Audio fingerprinting
- Batch rename functionality
- Scheduled scans
- Dark/light theme toggle
- Command-line interface mode

See [`roadmap.md`](roadmap.md) for complete details.

## Development

### Project Structure

```
FileXSorter/
├── src/
│   ├── main.rs       # Entry point, window setup
│   ├── app.rs        # GUI application logic (egui)
│   ├── scanner.rs    # Duplicate detection engine
│   └── file_ops.rs   # File delete/move operations
├── Cargo.toml        # Project configuration
├── plan.md          # Development plan
└── roadmap.md       # Future features
```

### Dependencies

- `eframe`/`egui` - GUI framework
- `rfd` - Native file dialogs
- `sha2` - SHA-256 hashing
- `rayon` - Parallel processing
- `walkdir` - Directory traversal
- `chrono` - Date/time formatting
- `open` - Open files in system explorer

### Building

```bash
# Debug build (faster compile, larger file)
cargo build

# Release build (optimized, small file)
cargo build --release
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui) - a simple, fast, and portable GUI library
- File dialogs via [rfd](https://github.com/PolyMeilex/rfd)
- Parallel processing with [rayon](https://github.com/rayon-rs/rayon)

---

Made with Rust | [Report Bug](../../issues) | [Request Feature](../../issues/new)
