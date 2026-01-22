# Development Plan

## Preferred Language

**Rust**

Rust is selected for the following reasons:
- **Performance**: Near C-level speed with zero-cost abstractions, ideal for file I/O operations
- **Memory Safety**: Prevents common bugs without garbage collection overhead
- **Small Binaries**: Compiles to a single lightweight `.exe` with no runtime dependencies
- **Cross-platform**: Easy Windows native development with excellent filesystem APIs
- **Concurrency**: Safe multi-threading for parallel file hashing (significant speed boost)

## Overview

**File X Sorter** is a lightweight Windows GUI application for detecting duplicate files. The tool scans user-selected directories, identifies duplicates using a two-stage detection system (filename matching + cryptographic hash verification), and provides options to delete or move duplicate files. The application features an optional recursive subfolder scan toggle, ensuring flexibility for different use cases.

### Core Workflow
1. User adds one or more folders to scan
2. User toggles recursive scanning on/off
3. Application scans and identifies duplicates across all folders
4. Results displayed in a clear, grouped list with preview panel
5. User can preview files and select duplicates to delete or move

## Version History

### v0.3.1 (Current)
- Fixed preview panel layout (compact, max 25% width)
- Fixed "Open" button to select file in Explorer
- Added "Open with default app" in preview panel
- Removed duplicate status text display
- Improved responsive window layout
- Optimized image preview with thumbnails

### v0.3.0
- Image preview support (PNG, JPG, GIF, BMP, WEBP)
- Video/audio file type indicators
- File type icons in duplicate list

### v0.2.0
- Multi-folder scanning support
- File preview panel with content preview
- Space analysis by folder
- Improved UI with collapsible preview

### v0.1.0
- Initial release with core duplicate detection
- Single folder scanning
- Delete/move operations

## Step-by-Step Implementation

### Phase 1: Project Setup [COMPLETED]
1. Initialize Rust project with Cargo
2. Configure dependencies in `Cargo.toml`
3. Set up project structure (src/main.rs, modules)
4. Configure Windows-specific build settings for `.exe` output

### Phase 2: Core Duplicate Detection Engine [COMPLETED]
5. Implement directory walker with optional recursion toggle
6. Create file metadata collector (name, size, path)
7. Implement size-based pre-filtering (files with unique sizes cannot be duplicates)
8. Implement SHA-256 hashing with chunked reading for large files
9. Build duplicate grouping logic (group files by hash)
10. Add progress tracking for scan operations

### Phase 3: File Operations [COMPLETED]
11. Implement safe file deletion with confirmation
12. Implement file move operation to user-specified directory
13. Add error handling for locked/protected files
14. Create operation logging for user reference

### Phase 4: GUI Development [COMPLETED]
15. Set up `eframe`/`egui` for native Windows GUI
16. Design main window layout:
    - Multi-folder selection with add/remove
    - Recursive scan toggle checkbox
    - Scan button with progress
    - Results table/list with preview panel
17. Implement folder browser dialog (native Windows)
18. Create results view with duplicate groups
19. Add selection checkboxes for duplicate files
20. Implement action buttons (Delete Selected, Move Selected)
21. Add move destination folder picker
22. Create confirmation dialogs for destructive actions

### Phase 5: Performance Optimization [COMPLETED]
23. Implement multi-threaded file hashing using `rayon`
24. Add early-exit optimization (skip hashing unique-sized files)
25. Implement incremental UI updates during scanning
26. Optimize memory usage for large directory scans

### Phase 6: Enhanced Features [COMPLETED - v0.2.0]
27. Multi-folder scanning support
28. File preview panel with metadata display
29. Text file content preview
30. Space analysis by folder breakdown
31. Improved status bar with selection info

### Phase 7: Packaging & Distribution [COMPLETED]
32. Configure release build optimizations in `Cargo.toml`
33. Add Windows application manifest and icon
34. Build final `FileXSorter.exe`
35. Test on clean Windows system (no Rust installed)

## Requirements

### Development Tools
| Tool | Purpose |
|------|---------|
| Rust (stable) | Compiler and toolchain |
| Cargo | Package manager and build system |
| Visual Studio Build Tools | Windows linker (MSVC) |

### Dependencies (Crates)
| Crate | Version | Purpose |
|-------|---------|---------|
| `eframe` | 0.29 | Cross-platform GUI framework (egui backend) |
| `egui` | 0.29 | Immediate-mode GUI library |
| `rfd` | 0.15 | Native file/folder dialogs |
| `sha2` | 0.10 | SHA-256 hashing |
| `rayon` | 1.10 | Parallel processing |
| `walkdir` | 2.5 | Recursive directory traversal |
| `chrono` | 0.4 | Date/time formatting |
| `open` | 5.0 | Open files in system explorer |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON support |

### System Requirements
- Windows 10/11 (64-bit)
- No runtime dependencies for end users
- Binary size: ~4 MB
