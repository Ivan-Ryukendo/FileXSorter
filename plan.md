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
1. User selects a folder to scan
2. User toggles recursive scanning on/off
3. Application scans and identifies duplicates
4. Results displayed in a clear, grouped list
5. User can select duplicates to delete or move to a specified folder

## Step-by-Step Implementation

### Phase 1: Project Setup
1. Initialize Rust project with Cargo
2. Configure dependencies in `Cargo.toml`
3. Set up project structure (src/main.rs, modules)
4. Configure Windows-specific build settings for `.exe` output

### Phase 2: Core Duplicate Detection Engine
5. Implement directory walker with optional recursion toggle
6. Create file metadata collector (name, size, path)
7. Implement size-based pre-filtering (files with unique sizes cannot be duplicates)
8. Implement SHA-256 hashing with chunked reading for large files
9. Build duplicate grouping logic (group files by hash)
10. Add progress tracking for scan operations

### Phase 3: File Operations
11. Implement safe file deletion with confirmation
12. Implement file move operation to user-specified directory
13. Add error handling for locked/protected files
14. Create operation logging for user reference

### Phase 4: GUI Development
15. Set up `eframe`/`egui` for native Windows GUI
16. Design main window layout:
    - Folder selection button with path display
    - Recursive scan toggle checkbox
    - Scan button
    - Progress bar
    - Results table/list
17. Implement folder browser dialog (native Windows)
18. Create results view with duplicate groups
19. Add selection checkboxes for duplicate files
20. Implement action buttons (Delete Selected, Move Selected)
21. Add move destination folder picker
22. Create confirmation dialogs for destructive actions

### Phase 5: Performance Optimization
23. Implement multi-threaded file hashing using `rayon`
24. Add early-exit optimization (skip hashing unique-sized files)
25. Implement incremental UI updates during scanning
26. Optimize memory usage for large directory scans

### Phase 6: Packaging & Distribution
27. Configure release build optimizations in `Cargo.toml`
28. Add Windows application manifest and icon
29. Build final `FileXSorter.exe`
30. Test on clean Windows system (no Rust installed)

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
| `eframe` | latest | Cross-platform GUI framework (egui backend) |
| `egui` | latest | Immediate-mode GUI library |
| `rfd` | latest | Native file/folder dialogs |
| `sha2` | latest | SHA-256 hashing |
| `rayon` | latest | Parallel processing |
| `walkdir` | latest | Recursive directory traversal |
| `dirs` | latest | Common directory paths |

### System Requirements
- Windows 10/11 (64-bit)
- No runtime dependencies for end users
- Estimated binary size: ~3-5 MB
