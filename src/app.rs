//! GUI module - Application state and UI rendering
//!
//! This module contains the main application state and egui-based UI.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use eframe::egui;
use rfd::FileDialog;

use crate::file_ops::{FileOperations, OperationResult};
use crate::scanner::{format_size, DuplicateGroup, FileEntry, ScanResult, Scanner, ScannerConfig};

/// Shared state for background scanning
struct ScanState {
    result: Mutex<Option<ScanResult>>,
    is_complete: AtomicBool,
    progress_current: AtomicUsize,
    progress_total: AtomicUsize,
    cancel_flag: AtomicBool,
}

impl ScanState {
    fn new() -> Self {
        Self {
            result: Mutex::new(None),
            is_complete: AtomicBool::new(false),
            progress_current: AtomicUsize::new(0),
            progress_total: AtomicUsize::new(0),
            cancel_flag: AtomicBool::new(false),
        }
    }
}

/// File type for preview purposes
#[derive(Clone, PartialEq)]
enum FileType {
    Image,
    Video,
    Audio,
    Gif,
    Text,
    Other,
}

/// File preview information
#[derive(Clone)]
struct FilePreview {
    path: PathBuf,
    name: String,
    size: u64,
    extension: String,
    modified: String,
    file_type: FileType,
    preview_text: Option<String>,
    dimensions: Option<(u32, u32)>, // For images
    duration_info: Option<String>,  // For audio/video
}

/// Application state
pub struct FileXSorterApp {
    // Scan settings - supports multiple folders
    selected_folders: Vec<PathBuf>,
    recursive_scan: bool,

    // Scan state
    is_scanning: bool,
    scan_result: Option<ScanResult>,
    scan_state: Arc<ScanState>,
    scan_handle: Option<JoinHandle<()>>,

    // Selection state (which files are selected for action)
    selected_files: Vec<(usize, usize)>, // (group_index, file_index)

    // File preview state
    preview_file: Option<FilePreview>,
    show_preview_panel: bool,
    loaded_images: HashMap<PathBuf, egui::TextureHandle>,

    // File operations
    file_ops: FileOperations,

    // UI state
    show_confirmation_dialog: Option<ConfirmationDialog>,
    status_message: Option<(String, MessageType)>,
}

#[derive(Clone)]
enum ConfirmationDialog {
    DeleteFiles(Vec<PathBuf>),
    MoveFiles(Vec<PathBuf>, PathBuf),
}

#[derive(Clone)]
enum MessageType {
    Info,
    Success,
    Error,
}

impl Default for FileXSorterApp {
    fn default() -> Self {
        Self {
            selected_folders: Vec::new(),
            recursive_scan: true,
            is_scanning: false,
            scan_result: None,
            scan_state: Arc::new(ScanState::new()),
            scan_handle: None,
            selected_files: Vec::new(),
            preview_file: None,
            show_preview_panel: true,
            loaded_images: HashMap::new(),
            file_ops: FileOperations::new(),
            show_confirmation_dialog: None,
            status_message: None,
        }
    }
}

impl FileXSorterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn get_file_type(extension: &str) -> FileType {
        match extension.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "ico" | "webp" | "tiff" | "tif" => FileType::Image,
            "gif" => FileType::Gif,
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpeg" | "mpg" => {
                FileType::Video
            }
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => FileType::Audio,
            "txt" | "md" | "rs" | "py" | "js" | "ts" | "json" | "xml" | "html" | "css" | "toml"
            | "yaml" | "yml" | "ini" | "cfg" | "log" | "csv" | "c" | "cpp" | "h" | "java"
            | "go" | "rb" | "php" | "sh" | "bat" | "ps1" => FileType::Text,
            _ => FileType::Other,
        }
    }

    fn start_scan(&mut self) {
        if self.selected_folders.is_empty() {
            self.status_message = Some((
                "Please add at least one folder to scan.".to_string(),
                MessageType::Error,
            ));
            return;
        }

        // Reset state
        self.is_scanning = true;
        self.scan_result = None;
        self.selected_files.clear();
        self.preview_file = None;
        self.loaded_images.clear();

        // Create new scan state
        self.scan_state = Arc::new(ScanState::new());

        let folders = self.selected_folders.clone();
        let recursive = self.recursive_scan;
        let scan_state = Arc::clone(&self.scan_state);

        // Spawn background thread
        let handle = thread::spawn(move || {
            let config = ScannerConfig {
                recursive,
                min_size: 1,
            };
            let scanner = Scanner::new(config);

            let progress_current = &scan_state.progress_current;
            let progress_total = &scan_state.progress_total;
            let cancel_flag = &scan_state.cancel_flag;

            let result = scanner.scan_directories_with_progress(
                &folders,
                progress_current,
                progress_total,
                cancel_flag,
            );

            if let Ok(mut guard) = scan_state.result.lock() {
                *guard = Some(result);
            }
            scan_state.is_complete.store(true, Ordering::SeqCst);
        });

        self.scan_handle = Some(handle);
        self.status_message = Some((
            format!("Scanning {} folder(s)...", self.selected_folders.len()),
            MessageType::Info,
        ));
    }

    fn check_scan_complete(&mut self) {
        if !self.is_scanning {
            return;
        }

        if self.scan_state.is_complete.load(Ordering::SeqCst) {
            if let Ok(mut guard) = self.scan_state.result.lock() {
                self.scan_result = guard.take();
            }

            self.is_scanning = false;

            if let Some(handle) = self.scan_handle.take() {
                let _ = handle.join();
            }

            if let Some(ref result) = self.scan_result {
                if result.duplicate_groups.is_empty() {
                    self.status_message =
                        Some(("No duplicates found.".to_string(), MessageType::Success));
                } else {
                    self.status_message = Some((
                        format!(
                            "Found {} duplicate groups ({} files, {} wasted)",
                            result.duplicate_groups.len(),
                            result.total_duplicates,
                            format_size(result.wasted_space)
                        ),
                        MessageType::Success,
                    ));
                }
            }
        }
    }

    fn cancel_scan(&mut self) {
        self.scan_state.cancel_flag.store(true, Ordering::SeqCst);
        self.is_scanning = false;
        self.status_message = Some(("Scan cancelled.".to_string(), MessageType::Info));
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(ref result) = self.scan_result {
            for (group_idx, file_idx) in &self.selected_files {
                if let Some(group) = result.duplicate_groups.get(*group_idx) {
                    if let Some(file) = group.files.get(*file_idx) {
                        paths.push(file.path.clone());
                    }
                }
            }
        }
        paths
    }

    fn load_file_preview(&mut self, file: &FileEntry) {
        let extension = file
            .path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let file_type = Self::get_file_type(&extension);

        let modified = fs::metadata(&file.path)
            .and_then(|m| m.modified())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_string());

        // Text preview for text files
        let preview_text = if file_type == FileType::Text && file.size < 1024 * 100 {
            fs::read_to_string(&file.path)
                .ok()
                .map(|s| s.chars().take(2000).collect())
        } else {
            None
        };

        // Get image dimensions
        let dimensions = if file_type == FileType::Image || file_type == FileType::Gif {
            image::image_dimensions(&file.path).ok()
        } else {
            None
        };

        // Duration info placeholder for audio/video
        let duration_info = match file_type {
            FileType::Video => Some(format!("Video file - {}", extension.to_uppercase())),
            FileType::Audio => Some(format!("Audio file - {}", extension.to_uppercase())),
            _ => None,
        };

        self.preview_file = Some(FilePreview {
            path: file.path.clone(),
            name: file.name.clone(),
            size: file.size,
            extension,
            modified,
            file_type,
            preview_text,
            dimensions,
            duration_info,
        });
    }

    fn load_image_texture(
        &mut self,
        ctx: &egui::Context,
        path: &PathBuf,
    ) -> Option<egui::TextureHandle> {
        if let Some(texture) = self.loaded_images.get(path) {
            return Some(texture.clone());
        }

        // Try to load image
        if let Ok(img) = image::open(path) {
            let img = img.to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            let pixels = img.into_raw();

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            let texture = ctx.load_texture(
                path.to_string_lossy(),
                color_image,
                egui::TextureOptions::LINEAR,
            );

            self.loaded_images.insert(path.clone(), texture.clone());
            return Some(texture);
        }

        None
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("File X Sorter");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("v0.3.0");
                ui.separator();
                ui.checkbox(&mut self.show_preview_panel, "Show Preview");
            });
        });
        ui.separator();
    }

    fn render_folder_selection(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Folders to scan:");
            ui.add_space(10.0);

            if ui.button("Add Folder").clicked() && !self.is_scanning {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    if !self.selected_folders.contains(&folder) {
                        self.selected_folders.push(folder);
                    }
                }
            }

            if ui
                .add_enabled(
                    !self.selected_folders.is_empty(),
                    egui::Button::new("Clear All"),
                )
                .clicked()
            {
                self.selected_folders.clear();
                self.scan_result = None;
                self.selected_files.clear();
            }
        });

        // Display selected folders
        if !self.selected_folders.is_empty() {
            ui.group(|ui| {
                egui::ScrollArea::horizontal()
                    .max_height(40.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut to_remove: Option<usize> = None;
                            for (idx, folder) in self.selected_folders.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui.small_button("X").clicked() && !self.is_scanning {
                                            to_remove = Some(idx);
                                        }
                                        ui.label(format!("{}", folder.display()));
                                    });
                                });
                            }
                            if let Some(idx) = to_remove {
                                self.selected_folders.remove(idx);
                            }
                        });
                    });
            });
        }

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.recursive_scan, "Scan subfolders");
            ui.add_space(20.0);

            if self.is_scanning {
                if ui.button("Cancel").clicked() {
                    self.cancel_scan();
                }
                ui.spinner();

                let current = self.scan_state.progress_current.load(Ordering::Relaxed);
                let total = self.scan_state.progress_total.load(Ordering::Relaxed);
                if total > 0 {
                    ui.label(format!("Hashing: {}/{} files", current, total));
                } else {
                    ui.label("Collecting files...");
                }
            } else if ui.button("Scan for Duplicates").clicked() {
                self.start_scan();
            }
        });
    }

    fn render_results(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(ref result) = self.scan_result.clone() {
            ui.separator();

            // Summary line
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Scanned {} files ({}) | {} groups | {} duplicates | {} wasted",
                    result.total_files,
                    format_size(result.total_size),
                    result.duplicate_groups.len(),
                    result.total_duplicates,
                    format_size(result.wasted_space)
                ));
            });

            ui.separator();

            // Action buttons
            ui.horizontal(|ui| {
                let selected_count = self.selected_files.len();

                if ui
                    .add_enabled(
                        selected_count > 0,
                        egui::Button::new(format!("Delete ({})", selected_count)),
                    )
                    .clicked()
                {
                    let paths = self.get_selected_paths();
                    self.show_confirmation_dialog = Some(ConfirmationDialog::DeleteFiles(paths));
                }

                if ui
                    .add_enabled(
                        selected_count > 0,
                        egui::Button::new(format!("Move ({})", selected_count)),
                    )
                    .clicked()
                {
                    if let Some(dest) = FileDialog::new().pick_folder() {
                        let paths = self.get_selected_paths();
                        self.show_confirmation_dialog =
                            Some(ConfirmationDialog::MoveFiles(paths, dest));
                    }
                }

                if ui.button("Select All Duplicates").clicked() {
                    self.selected_files.clear();
                    for (g_idx, group) in result.duplicate_groups.iter().enumerate() {
                        for f_idx in 1..group.files.len() {
                            self.selected_files.push((g_idx, f_idx));
                        }
                    }
                }

                if ui.button("Clear Selection").clicked() {
                    self.selected_files.clear();
                }
            });

            ui.separator();

            // Main content: Preview on right side, file list in remaining space
            let available = ui.available_size();
            let preview_width = if self.show_preview_panel { 280.0 } else { 0.0 };
            let list_width = available.x - preview_width - 10.0;

            ui.horizontal(|ui| {
                // Left side: Duplicate groups list (fills most space)
                ui.vertical(|ui| {
                    ui.set_width(list_width);

                    // Results summary in green
                    ui.label(
                        egui::RichText::new(format!(
                            "Found {} duplicate groups ({} files, {} wasted)",
                            result.duplicate_groups.len(),
                            result.total_duplicates,
                            format_size(result.wasted_space)
                        ))
                        .color(egui::Color32::from_rgb(100, 255, 100)),
                    );

                    ui.separator();

                    // Scrollable list of duplicate groups
                    egui::ScrollArea::vertical()
                        .id_salt("file_list_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (group_idx, group) in result.duplicate_groups.iter().enumerate() {
                                self.render_duplicate_group(ui, group_idx, group);
                            }
                        });
                });

                // Right side: Preview panel (compact)
                if self.show_preview_panel {
                    ui.separator();
                    self.render_preview_panel(ui, ctx);
                }
            });

            // Show errors if any
            if !result.errors.is_empty() {
                ui.separator();
                ui.collapsing(format!("Errors ({})", result.errors.len()), |ui| {
                    for error in &result.errors {
                        ui.label(egui::RichText::new(error).color(egui::Color32::RED).small());
                    }
                });
            }
        } else if !self.is_scanning {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Add folders and click 'Scan for Duplicates' to begin.");
            });
        }
    }

    fn render_preview_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            ui.set_width(270.0);
            ui.heading("File Preview");
            ui.separator();

            if let Some(ref preview) = self.preview_file.clone() {
                // File name
                ui.label(egui::RichText::new(&preview.name).strong().size(14.0));

                // File info grid
                egui::Grid::new("preview_info")
                    .num_columns(2)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        ui.label("Size:");
                        ui.label(format_size(preview.size));
                        ui.end_row();

                        ui.label("Type:");
                        let type_str = match preview.file_type {
                            FileType::Image => "Image",
                            FileType::Gif => "GIF",
                            FileType::Video => "Video",
                            FileType::Audio => "Audio",
                            FileType::Text => "Text",
                            FileType::Other => "File",
                        };
                        ui.label(format!(
                            "{} ({})",
                            type_str,
                            preview.extension.to_uppercase()
                        ));
                        ui.end_row();

                        if let Some((w, h)) = preview.dimensions {
                            ui.label("Dimensions:");
                            ui.label(format!("{}x{}", w, h));
                            ui.end_row();
                        }
                    });

                ui.add_space(5.0);

                // Media preview area
                match preview.file_type {
                    FileType::Image | FileType::Gif => {
                        ui.group(|ui| {
                            ui.set_max_height(200.0);
                            if let Some(texture) = self.load_image_texture(ctx, &preview.path) {
                                let size = texture.size_vec2();
                                let max_size = egui::vec2(250.0, 180.0);
                                let scale = (max_size.x / size.x).min(max_size.y / size.y).min(1.0);
                                let display_size = size * scale;
                                ui.image(&texture);
                                let _ = display_size; // Used for sizing
                            } else {
                                ui.label("Loading image...");
                            }
                        });
                    }
                    FileType::Video => {
                        ui.group(|ui| {
                            ui.set_min_height(80.0);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("🎬").size(32.0));
                                ui.label("Video File");
                                if let Some(ref info) = preview.duration_info {
                                    ui.label(
                                        egui::RichText::new(info)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                    );
                                }
                            });
                        });
                    }
                    FileType::Audio => {
                        ui.group(|ui| {
                            ui.set_min_height(80.0);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("🎵").size(32.0));
                                ui.label("Audio File");
                                if let Some(ref info) = preview.duration_info {
                                    ui.label(
                                        egui::RichText::new(info)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                    );
                                }
                            });
                        });
                    }
                    FileType::Text => {
                        if let Some(ref text) = preview.preview_text {
                            ui.group(|ui| {
                                egui::ScrollArea::vertical()
                                    .id_salt("text_preview")
                                    .max_height(150.0)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut text.as_str())
                                                .font(egui::TextStyle::Monospace)
                                                .desired_width(f32::INFINITY)
                                                .interactive(false),
                                        );
                                    });
                            });
                        }
                    }
                    FileType::Other => {
                        ui.group(|ui| {
                            ui.set_min_height(60.0);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("📄").size(24.0));
                                ui.label("No preview available");
                            });
                        });
                    }
                }

                ui.add_space(5.0);

                // Path
                ui.label(egui::RichText::new("Path:").small());
                ui.label(
                    egui::RichText::new(preview.path.display().to_string())
                        .small()
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(5.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.small_button("Open File").clicked() {
                        let _ = open::that(&preview.path);
                    }
                    if ui.small_button("Open Folder").clicked() {
                        if let Some(parent) = preview.path.parent() {
                            let _ = open::that(parent);
                        }
                    }
                });
            } else {
                ui.label(
                    egui::RichText::new("Click 'Preview' on a file")
                        .italics()
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Supported previews:").small());
                ui.label(
                    egui::RichText::new("• Images: PNG, JPG, GIF, BMP, WEBP")
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.label(
                    egui::RichText::new("• Video: MP4, AVI, MKV, MOV...")
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.label(
                    egui::RichText::new("• Audio: MP3, WAV, FLAC, AAC...")
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.label(
                    egui::RichText::new("• Text: TXT, MD, JSON, code...")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            }
        });
    }

    fn render_duplicate_group(
        &mut self,
        ui: &mut egui::Ui,
        group_idx: usize,
        group: &DuplicateGroup,
    ) {
        let header = format!(
            "Group: {} files | {} each | {} wasted | Hash: {}...",
            group.files.len(),
            format_size(group.files.first().map(|f| f.size).unwrap_or(0)),
            format_size(group.wasted_size),
            &group.hash[..8.min(group.hash.len())]
        );

        egui::CollapsingHeader::new(header)
            .default_open(group.files.len() <= 5)
            .show(ui, |ui| {
                for (file_idx, file) in group.files.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let is_selected = self.selected_files.contains(&(group_idx, file_idx));
                        let mut selected = is_selected;

                        if ui.checkbox(&mut selected, "").changed() {
                            if selected {
                                self.selected_files.push((group_idx, file_idx));
                            } else {
                                self.selected_files
                                    .retain(|&(g, f)| !(g == group_idx && f == file_idx));
                            }
                        }

                        if file_idx == 0 {
                            ui.label(
                                egui::RichText::new("[KEEP]")
                                    .color(egui::Color32::GREEN)
                                    .strong(),
                            );
                        }

                        // File type icon
                        let ext = file
                            .path
                            .extension()
                            .map(|e| e.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        let icon = match Self::get_file_type(&ext) {
                            FileType::Image => "🖼",
                            FileType::Gif => "🎞",
                            FileType::Video => "🎬",
                            FileType::Audio => "🎵",
                            FileType::Text => "📄",
                            FileType::Other => "📁",
                        };
                        ui.label(icon);

                        ui.label(&file.name);
                        ui.label(format_size(file.size));

                        if ui.small_button("Preview").clicked() {
                            self.load_file_preview(file);
                        }

                        if ui.small_button("Open").clicked() {
                            if let Some(parent) = file.path.parent() {
                                let _ = open::that(parent);
                            }
                        }
                    });
                }
            });
    }

    fn render_confirmation_dialog(&mut self, ctx: &egui::Context) {
        if let Some(dialog) = self.show_confirmation_dialog.clone() {
            egui::Window::new("Confirm Action")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| match &dialog {
                    ConfirmationDialog::DeleteFiles(paths) => {
                        ui.label(format!("Delete {} file(s)?", paths.len()));
                        ui.label(
                            egui::RichText::new("This cannot be undone!").color(egui::Color32::RED),
                        );
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Delete").clicked() {
                                let results = self.file_ops.delete_files(paths);
                                let success = results
                                    .iter()
                                    .filter(|r| matches!(r, OperationResult::Success(_)))
                                    .count();
                                self.status_message = Some((
                                    format!("Deleted {} of {} files.", success, paths.len()),
                                    if success == paths.len() {
                                        MessageType::Success
                                    } else {
                                        MessageType::Error
                                    },
                                ));
                                self.selected_files.clear();
                                self.preview_file = None;
                                self.show_confirmation_dialog = None;
                                if !self.selected_folders.is_empty() {
                                    self.start_scan();
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_confirmation_dialog = None;
                            }
                        });
                    }
                    ConfirmationDialog::MoveFiles(paths, dest) => {
                        ui.label(format!("Move {} file(s) to:", paths.len()));
                        ui.label(dest.display().to_string());
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Move").clicked() {
                                let results = self.file_ops.move_files(paths, dest);
                                let success = results
                                    .iter()
                                    .filter(|r| matches!(r, OperationResult::Success(_)))
                                    .count();
                                self.status_message = Some((
                                    format!("Moved {} of {} files.", success, paths.len()),
                                    if success == paths.len() {
                                        MessageType::Success
                                    } else {
                                        MessageType::Error
                                    },
                                ));
                                self.selected_files.clear();
                                self.preview_file = None;
                                self.show_confirmation_dialog = None;
                                if !self.selected_folders.is_empty() {
                                    self.start_scan();
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_confirmation_dialog = None;
                            }
                        });
                    }
                });
        }
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            if let Some((msg, msg_type)) = &self.status_message {
                let color = match msg_type {
                    MessageType::Info => egui::Color32::GRAY,
                    MessageType::Success => egui::Color32::GREEN,
                    MessageType::Error => egui::Color32::RED,
                };
                ui.label(egui::RichText::new(msg).color(color));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.selected_files.is_empty() {
                    let selected_size: u64 = self
                        .get_selected_paths()
                        .iter()
                        .filter_map(|p| fs::metadata(p).ok())
                        .map(|m| m.len())
                        .sum();
                    ui.label(format!(
                        "Selected: {} ({})",
                        self.selected_files.len(),
                        format_size(selected_size)
                    ));
                }
            });
        });
    }
}

impl eframe::App for FileXSorterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_scan_complete();

        if self.is_scanning {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            self.render_folder_selection(ui);
            self.render_results(ui, ctx);
            self.render_status_bar(ui);
        });

        self.render_confirmation_dialog(ctx);
    }
}
