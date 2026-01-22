//! GUI module - Application state and UI rendering
//!
//! This module contains the main application state and egui-based UI.

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

/// File preview information
#[derive(Clone, Default)]
struct FilePreview {
    path: PathBuf,
    name: String,
    size: u64,
    extension: String,
    modified: String,
    preview_text: Option<String>,
    is_image: bool,
}

/// Application state
pub struct FileXSorterApp {
    // Scan settings - now supports multiple folders
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

            // Scan all directories
            let result = scanner.scan_directories_with_progress(
                &folders,
                progress_current,
                progress_total,
                cancel_flag,
            );

            // Store result
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
            // Scan is done, get result
            if let Ok(mut guard) = self.scan_state.result.lock() {
                self.scan_result = guard.take();
            }

            self.is_scanning = false;

            // Clean up thread handle
            if let Some(handle) = self.scan_handle.take() {
                let _ = handle.join();
            }

            // Update status
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

        let is_image = matches!(
            extension.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp"
        );

        let is_text = matches!(
            extension.as_str(),
            "txt"
                | "md"
                | "rs"
                | "py"
                | "js"
                | "ts"
                | "json"
                | "xml"
                | "html"
                | "css"
                | "toml"
                | "yaml"
                | "yml"
                | "ini"
                | "cfg"
                | "log"
                | "csv"
        );

        // Get file modified time
        let modified = fs::metadata(&file.path)
            .and_then(|m| m.modified())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_string());

        // Try to read text preview for text files
        let preview_text = if is_text && file.size < 1024 * 100 {
            // Only preview files < 100KB
            fs::read_to_string(&file.path)
                .ok()
                .map(|s| s.chars().take(2000).collect()) // Limit preview to 2000 chars
        } else {
            None
        };

        self.preview_file = Some(FilePreview {
            path: file.path.clone(),
            name: file.name.clone(),
            size: file.size,
            extension,
            modified,
            preview_text,
            is_image,
        });
    }

    fn calculate_folder_sizes(&self) -> Vec<(PathBuf, u64, usize)> {
        let mut folder_stats: Vec<(PathBuf, u64, usize)> = Vec::new();

        if let Some(ref result) = self.scan_result {
            for group in &result.duplicate_groups {
                for file in &group.files {
                    if let Some(parent) = file.path.parent() {
                        let parent_path = parent.to_path_buf();
                        if let Some(entry) =
                            folder_stats.iter_mut().find(|(p, _, _)| *p == parent_path)
                        {
                            entry.1 += file.size;
                            entry.2 += 1;
                        } else {
                            folder_stats.push((parent_path, file.size, 1));
                        }
                    }
                }
            }
        }

        // Sort by size (largest first)
        folder_stats.sort_by(|a, b| b.1.cmp(&a.1));
        folder_stats
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("File X Sorter");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("v0.2.0");
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

        // Display selected folders with remove buttons
        if !self.selected_folders.is_empty() {
            ui.group(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(80.0)
                    .show(ui, |ui| {
                        let mut to_remove: Option<usize> = None;
                        for (idx, folder) in self.selected_folders.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.small_button("X").clicked() && !self.is_scanning {
                                    to_remove = Some(idx);
                                }
                                ui.label(format!("{}", folder.display()));
                            });
                        }
                        if let Some(idx) = to_remove {
                            self.selected_folders.remove(idx);
                        }
                    });
            });
        } else {
            ui.label(
                egui::RichText::new("No folders selected. Click 'Add Folder' to begin.")
                    .italics()
                    .color(egui::Color32::GRAY),
            );
        }

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.recursive_scan, "Scan subfolders");
            ui.add_space(20.0);

            if self.is_scanning {
                if ui.button("Cancel").clicked() {
                    self.cancel_scan();
                }
                ui.spinner();

                // Show progress
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

    fn render_results(&mut self, ui: &mut egui::Ui) {
        if let Some(ref result) = self.scan_result.clone() {
            ui.separator();

            // Summary
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Scanned {} files ({}) | Found {} duplicate groups | {} duplicates | {} wasted",
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
                        egui::Button::new(format!("Delete Selected ({})", selected_count)),
                    )
                    .clicked()
                {
                    let paths = self.get_selected_paths();
                    self.show_confirmation_dialog = Some(ConfirmationDialog::DeleteFiles(paths));
                }

                if ui
                    .add_enabled(
                        selected_count > 0,
                        egui::Button::new(format!("Move Selected ({})", selected_count)),
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
                        // Select all but the first file in each group
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

            // Main content area with optional preview panel
            let available_width = ui.available_width();
            let results_width = if self.show_preview_panel {
                available_width * 0.6
            } else {
                available_width
            };

            ui.horizontal(|ui| {
                // Duplicate groups list
                ui.vertical(|ui| {
                    ui.set_width(results_width);
                    egui::ScrollArea::vertical()
                        .id_salt("results_scroll")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            for (group_idx, group) in result.duplicate_groups.iter().enumerate() {
                                self.render_duplicate_group(ui, group_idx, group);
                            }
                        });
                });

                // Preview panel
                if self.show_preview_panel {
                    ui.separator();
                    self.render_preview_panel(ui);
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

    fn render_preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.set_min_width(250.0);
            ui.heading("File Preview");
            ui.separator();

            if let Some(ref preview) = self.preview_file.clone() {
                // File info
                ui.group(|ui| {
                    ui.label(egui::RichText::new(&preview.name).strong());
                    ui.separator();

                    egui::Grid::new("preview_info")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Size:");
                            ui.label(format_size(preview.size));
                            ui.end_row();

                            ui.label("Type:");
                            ui.label(if preview.extension.is_empty() {
                                "Unknown".to_string()
                            } else {
                                preview.extension.to_uppercase()
                            });
                            ui.end_row();

                            ui.label("Modified:");
                            ui.label(&preview.modified);
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Content preview
                if let Some(ref text) = preview.preview_text {
                    ui.label(egui::RichText::new("Content Preview:").strong());
                    egui::ScrollArea::vertical()
                        .id_salt("preview_text_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut text.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false),
                            );
                        });
                } else if preview.is_image {
                    ui.label(
                        egui::RichText::new("Image file - Open folder to view")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("No preview available for this file type")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                }

                ui.add_space(10.0);

                // Full path
                ui.label(egui::RichText::new("Path:").strong());
                ui.label(
                    egui::RichText::new(preview.path.display().to_string())
                        .small()
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(10.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("Open File").clicked() {
                        let _ = open::that(&preview.path);
                    }
                    if ui.button("Open Folder").clicked() {
                        if let Some(parent) = preview.path.parent() {
                            let _ = open::that(parent);
                        }
                    }
                });
            } else {
                ui.label(
                    egui::RichText::new("Click 'Preview' on a file to see details here.")
                        .italics()
                        .color(egui::Color32::GRAY),
                );

                // Show folder space breakdown
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Space by Folder:").strong());
                ui.separator();

                let folder_stats = self.calculate_folder_sizes();
                egui::ScrollArea::vertical()
                    .id_salt("folder_stats_scroll")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (path, size, count) in folder_stats.iter().take(20) {
                            ui.horizontal(|ui| {
                                ui.label(format!("{} ({} files)", format_size(*size), count));
                            });
                            ui.label(
                                egui::RichText::new(path.display().to_string())
                                    .small()
                                    .color(egui::Color32::GRAY),
                            );
                            ui.add_space(5.0);
                        }
                    });
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

                        // First file indicator (original)
                        if file_idx == 0 {
                            ui.label(
                                egui::RichText::new("[KEEP]")
                                    .color(egui::Color32::GREEN)
                                    .strong(),
                            );
                        }

                        ui.label(&file.name);
                        ui.label(format_size(file.size));

                        // Show path on hover
                        ui.label("|").on_hover_text(file.path.display().to_string());

                        // Preview button
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
                .show(ctx, |ui| {
                    match &dialog {
                        ConfirmationDialog::DeleteFiles(paths) => {
                            ui.label(format!(
                                "Are you sure you want to delete {} file(s)?",
                                paths.len()
                            ));
                            ui.label("This action cannot be undone!");

                            ui.separator();

                            ui.horizontal(|ui| {
                                if ui.button("Delete").clicked() {
                                    let results = self.file_ops.delete_files(paths);
                                    let success_count = results
                                        .iter()
                                        .filter(|r| matches!(r, OperationResult::Success(_)))
                                        .count();
                                    self.status_message = Some((
                                        format!(
                                            "Deleted {} of {} files.",
                                            success_count,
                                            paths.len()
                                        ),
                                        if success_count == paths.len() {
                                            MessageType::Success
                                        } else {
                                            MessageType::Error
                                        },
                                    ));
                                    self.selected_files.clear();
                                    self.preview_file = None;
                                    self.show_confirmation_dialog = None;

                                    // Trigger rescan
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
                                    let success_count = results
                                        .iter()
                                        .filter(|r| matches!(r, OperationResult::Success(_)))
                                        .count();
                                    self.status_message = Some((
                                        format!(
                                            "Moved {} of {} files.",
                                            success_count,
                                            paths.len()
                                        ),
                                        if success_count == paths.len() {
                                            MessageType::Success
                                        } else {
                                            MessageType::Error
                                        },
                                    ));
                                    self.selected_files.clear();
                                    self.preview_file = None;
                                    self.show_confirmation_dialog = None;

                                    // Trigger rescan
                                    if !self.selected_folders.is_empty() {
                                        self.start_scan();
                                    }
                                }

                                if ui.button("Cancel").clicked() {
                                    self.show_confirmation_dialog = None;
                                }
                            });
                        }
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

            // Show selected file count
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let selected_size: u64 = self
                    .get_selected_paths()
                    .iter()
                    .filter_map(|p| fs::metadata(p).ok())
                    .map(|m| m.len())
                    .sum();
                if !self.selected_files.is_empty() {
                    ui.label(format!(
                        "Selected: {} files ({})",
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
        // Check if background scan completed
        self.check_scan_complete();

        // Request repaint while scanning for progress updates
        if self.is_scanning {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            self.render_folder_selection(ui);
            self.render_results(ui);
            self.render_status_bar(ui);
        });

        self.render_confirmation_dialog(ctx);
    }
}
