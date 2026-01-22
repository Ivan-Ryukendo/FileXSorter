# File X Sorter

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![License](https://img.shields.io/badge/license-MIT-green)

A lightweight, fast Windows application for detecting and managing duplicate files using SHA-256 hash verification with a clean graphical interface.

## Features

- **Fast Duplicate Detection** - Two-stage detection (size pre-filter + SHA-256 hash) for optimal performance
- **Multi-threaded Scanning** - Parallel file hashing using Rayon for speed
- **Recursive/Non-recursive Scans** - Toggle subfolder scanning as needed
- **Clean GUI** - Built with egui for a modern, responsive interface
- **File Management** - Delete or move duplicate files directly from the app
- **Progress Tracking** - Real-time progress indicator during scans
- **Intelligent Grouping** - Duplicates sorted by wasted space (largest first)
- **Windows Explorer Integration** - Open folder location with one click
- **Standalone Executable** - No runtime dependencies, single 3.9 MB exe

## Screenshots

![Main Interface](docs/images/main.png)

## Installation

### Download the Executable

1. Download the latest `File X Sorter.exe` from the [Releases](releases) page
2. Place it anywhere on your computer
3. Double-click to run - no installation required

### Build from Source

If you have Rust installed, you can build from source:

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/FileXSorter.git
cd FileXSorter

# Build release version
cargo build --release

# The executable will be at:
# target/release/file-x-sorter.exe
```

## Usage

1. **Select Folder**: Click "Browse..." to choose the directory you want to scan
2. **Configure Scan**: Toggle "Scan subfolders" on or off as needed
3. **Start Scan**: Click "Scan for Duplicates" to begin
4. **Review Results**: Duplicate groups are displayed, sorted by wasted space
5. **Select Files**: Check the duplicates you want to remove (first file in each group is marked [KEEP])
6. **Take Action**: 
   - Click "Delete Selected" to permanently remove duplicates
   - Click "Move Selected" to move duplicates to another folder
7. **Confirmation**: Confirm the action in the dialog

### Tips

- Use "Select All Duplicates" to quickly select all but one copy from each group
- Click "Open Folder" next to any file to view it in Windows Explorer
- Cancel a long-running scan using the "Cancel" button
- Results show total files scanned, duplicate groups found, and wasted space

## How It Works

File X Sorter uses a multi-stage detection algorithm:

1. **File Collection**: Walk through directories (recursive or non-recursive)
2. **Size Filtering**: Group files by size - files with unique sizes cannot be duplicates
3. **Hash Computation**: SHA-256 hash computed in parallel for same-sized files
4. **Duplicate Grouping**: Files with identical hashes are grouped together
5. **Results Display**: Groups sorted by wasted space for easy prioritization

## Requirements

- Windows 10 or Windows 11 (64-bit)
- Approximately 3.9 MB disk space

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
