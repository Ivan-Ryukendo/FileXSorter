//! GUI module - Application state and UI rendering
//!
//! This module contains the main application state and egui-based UI.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
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
    file_type: FileType,
    preview_text: Option<String>,
    dimensions: Option<(u32, u32)>,
}

/// Application state
pub struct FileXSorterApp {
    selected_folders: Vec<PathBuf>,
    recursive_scan: bool,
    is_scanning: bool,
    scan_result: Option<ScanResult>,
    scan_state: Arc<ScanState>,
    scan_handle: Option<JoinHandle<()>>,
    selected_files: Vec<(usize, usize)>,
    preview_file: Option<FilePreview>,
    show_preview_panel: bool,
    preview_panel_width: f32,
    loaded_images: HashMap<PathBuf, egui::TextureHandle>,
    file_ops: FileOperations,
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
            preview_panel_width: 220.0,
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

    /// Open file with default system application
    fn open_file_with_default(path: &PathBuf) {
        let _ = open::that(path);
    }

    /// Open folder and select the specific file in Windows Explorer
    fn open_folder_and_select_file(path: &PathBuf) {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("explorer").arg("/select,").arg(path).spawn();
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Some(parent) = path.parent() {
                let _ = open::that(parent);
            }
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

        self.is_scanning = true;
        self.scan_result = None;
        self.selected_files.clear();
        self.preview_file = None;
        self.loaded_images.clear();
        self.scan_state = Arc::new(ScanState::new());

        let folders = self.selected_folders.clone();
        let recursive = self.recursive_scan;
        let scan_state = Arc::clone(&self.scan_state);

        let handle = thread::spawn(move || {
            let config = ScannerConfig {
                recursive,
                min_size: 1,
            };
            let scanner = Scanner::new(config);
            let result = scanner.scan_directories_with_progress(
                &folders,
                &scan_state.progress_current,
                &scan_state.progress_total,
                &scan_state.cancel_flag,
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
        if !self.is_scanning || !self.scan_state.is_complete.load(Ordering::SeqCst) {
            return;
        }

        if let Ok(mut guard) = self.scan_state.result.lock() {
            self.scan_result = guard.take();
        }
        self.is_scanning = false;

        if let Some(handle) = self.scan_handle.take() {
            let _ = handle.join();
        }

        if let Some(ref result) = self.scan_result {
            self.status_message = if result.duplicate_groups.is_empty() {
                Some(("No duplicates found.".to_string(), MessageType::Success))
            } else {
                Some((
                    format!(
                        "Found {} groups ({} files, {} wasted)",
                        result.duplicate_groups.len(),
                        result.total_duplicates,
                        format_size(result.wasted_space)
                    ),
                    MessageType::Success,
                ))
            };
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

        let preview_text = if file_type == FileType::Text && file.size < 50 * 1024 {
            fs::read_to_string(&file.path)
                .ok()
                .map(|s| s.chars().take(1000).collect())
        } else {
            None
        };

        let dimensions = if file_type == FileType::Image || file_type == FileType::Gif {
            image::image_dimensions(&file.path).ok()
        } else {
            None
        };

        self.preview_file = Some(FilePreview {
            path: file.path.clone(),
            name: file.name.clone(),
            size: file.size,
            extension,
            file_type,
            preview_text,
            dimensions,
        });
    }

    fn load_image_texture(
        &mut self,
        ctx: &egui::Context,
        path: &PathBuf,
        max_size: f32,
    ) -> Option<egui::TextureHandle> {
        if let Some(texture) = self.loaded_images.get(path) {
            return Some(texture.clone());
        }

        if let Ok(img) = image::open(path) {
            // Resize for preview to save memory
            let img = img.thumbnail(max_size as u32, max_size as u32);
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
            ui.heading("FileXSorter");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("v0.3.2");
                ui.separator();
                ui.checkbox(&mut self.show_preview_panel, "Preview");
            });
        });
        ui.separator();
    }

    fn render_folder_selection(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Folders:");
            if ui.button("Add").clicked() && !self.is_scanning {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    if !self.selected_folders.contains(&folder) {
                        self.selected_folders.push(folder);
                    }
                }
            }
            if ui
                .add_enabled(
                    !self.selected_folders.is_empty(),
                    egui::Button::new("Clear"),
                )
                .clicked()
            {
                self.selected_folders.clear();
                self.scan_result = None;
                self.selected_files.clear();
            }
        });

        if !self.selected_folders.is_empty() {
            egui::ScrollArea::horizontal()
                .max_height(35.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut to_remove = None;
                        for (idx, folder) in self.selected_folders.iter().enumerate() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.small_button("X").clicked() && !self.is_scanning {
                                        to_remove = Some(idx);
                                    }
                                    ui.label(folder.display().to_string());
                                });
                            });
                        }
                        if let Some(idx) = to_remove {
                            self.selected_folders.remove(idx);
                        }
                    });
                });
        }

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.recursive_scan, "Subfolders");
            if self.is_scanning {
                if ui.button("Cancel").clicked() {
                    self.cancel_scan();
                }
                ui.spinner();
                let current = self.scan_state.progress_current.load(Ordering::Relaxed);
                let total = self.scan_state.progress_total.load(Ordering::Relaxed);
                ui.label(if total > 0 {
                    format!("{}/{}", current, total)
                } else {
                    "Scanning...".into()
                });
            } else if ui.button("Scan").clicked() {
                self.start_scan();
            }
        });
    }

    fn render_results_only(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let result = match self.scan_result.clone() {
            Some(r) => r,
            None => {
                if !self.is_scanning {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label("Add folders and click 'Scan' to find duplicates.");
                    });
                }
                return;
            }
        };

        ui.separator();

        // Compact summary line
        ui.label(format!(
            "Scanned {} files ({}) | {} groups | {} duplicates | {} wasted",
            result.total_files,
            format_size(result.total_size),
            result.duplicate_groups.len(),
            result.total_duplicates,
            format_size(result.wasted_space)
        ));

        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            let count = self.selected_files.len();
            if ui
                .add_enabled(count > 0, egui::Button::new(format!("Delete ({})", count)))
                .clicked()
            {
                self.show_confirmation_dialog =
                    Some(ConfirmationDialog::DeleteFiles(self.get_selected_paths()));
            }
            if ui
                .add_enabled(count > 0, egui::Button::new(format!("Move ({})", count)))
                .clicked()
            {
                if let Some(dest) = FileDialog::new().pick_folder() {
                    self.show_confirmation_dialog = Some(ConfirmationDialog::MoveFiles(
                        self.get_selected_paths(),
                        dest,
                    ));
                }
            }
            if ui.button("Select All").clicked() {
                self.selected_files.clear();
                for (g, group) in result.duplicate_groups.iter().enumerate() {
                    for f in 1..group.files.len() {
                        self.selected_files.push((g, f));
                    }
                }
            }
            if ui.button("Clear").clicked() {
                self.selected_files.clear();
            }
        });

        ui.separator();

        // File list - uses ALL available space
        let available = ui.available_size();
        egui::ScrollArea::vertical()
            .id_salt("main_list")
            .auto_shrink([false, false])
            .max_height(available.y)
            .show(ui, |ui| {
                for (group_idx, group) in result.duplicate_groups.iter().enumerate() {
                    self.render_group(ui, group_idx, group);
                }
            });
    }

    fn render_preview_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let available_height = ui.available_height();
        let width = ui.available_width();

        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Preview").strong());
            ui.separator();

            let preview = match self.preview_file.clone() {
                Some(p) => p,
                None => {
                    ui.label(
                        egui::RichText::new("Click 👁 to preview a file")
                            .small()
                            .italics(),
                    );
                    return;
                }
            };

            // File info
            ui.label(egui::RichText::new(&preview.name).strong().size(11.0));
            ui.label(format!(
                "{} | {}",
                format_size(preview.size),
                preview.extension.to_uppercase()
            ));

            if let Some((w, h)) = preview.dimensions {
                ui.label(egui::RichText::new(format!("{}x{}", w, h)).small());
            }

            ui.add_space(8.0);

            // Calculate available space for content
            let header_used = 90.0;
            let button_height = 35.0;
            let content_height = (available_height - header_used - button_height).max(80.0);

            // Content preview - scales with panel size
            match preview.file_type {
                FileType::Image | FileType::Gif => {
                    if let Some(texture) = self.load_image_texture(ctx, &preview.path, width * 2.0)
                    {
                        let size = texture.size_vec2();
                        let scale_w = (width - 10.0) / size.x;
                        let scale_h = content_height / size.y;
                        let scale = scale_w.min(scale_h).min(1.0);
                        ui.image(egui::load::SizedTexture::new(texture.id(), size * scale));
                    }
                }
                FileType::Video => {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("🎬").size(64.0));
                        ui.label("Video File");
                    });
                }
                FileType::Audio => {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("🎵").size(64.0));
                        ui.label("Audio File");
                    });
                }
                FileType::Text => {
                    if let Some(ref text) = preview.preview_text {
                        egui::ScrollArea::vertical()
                            .max_height(content_height)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(text).monospace().size(10.0));
                            });
                    }
                }
                FileType::Other => {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("📄").size(64.0));
                        ui.label("File");
                    });
                }
            }

            ui.add_space(8.0);

            // Action buttons at bottom
            ui.horizontal(|ui| {
                if ui
                    .button("Open")
                    .on_hover_text("Open with default app")
                    .clicked()
                {
                    Self::open_file_with_default(&preview.path);
                }
                if ui
                    .button("Folder")
                    .on_hover_text("Show in Explorer")
                    .clicked()
                {
                    Self::open_folder_and_select_file(&preview.path);
                }
            });
        });
    }

    fn render_group(&mut self, ui: &mut egui::Ui, group_idx: usize, group: &DuplicateGroup) {
        let header = format!(
            "{} files | {} each | {} wasted",
            group.files.len(),
            format_size(group.files.first().map(|f| f.size).unwrap_or(0)),
            format_size(group.wasted_size)
        );

        egui::CollapsingHeader::new(header)
            .default_open(group.files.len() <= 3)
            .show(ui, |ui| {
                for (file_idx, file) in group.files.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let mut selected = self.selected_files.contains(&(group_idx, file_idx));
                        if ui.checkbox(&mut selected, "").changed() {
                            if selected {
                                self.selected_files.push((group_idx, file_idx));
                            } else {
                                self.selected_files
                                    .retain(|&(g, f)| g != group_idx || f != file_idx);
                            }
                        }

                        if file_idx == 0 {
                            ui.label(
                                egui::RichText::new("[KEEP]")
                                    .color(egui::Color32::GREEN)
                                    .strong(),
                            );
                        }

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

                        if ui.small_button("👁").on_hover_text("Preview").clicked() {
                            self.load_file_preview(file);
                        }
                        if ui
                            .small_button("📂")
                            .on_hover_text("Open folder & select file")
                            .clicked()
                        {
                            Self::open_folder_and_select_file(&file.path);
                        }
                    });
                }
            });
    }

    fn render_confirmation_dialog(&mut self, ctx: &egui::Context) {
        let dialog = match self.show_confirmation_dialog.clone() {
            Some(d) => d,
            None => return,
        };

        egui::Window::new("Confirm")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| match &dialog {
                ConfirmationDialog::DeleteFiles(paths) => {
                    ui.label(format!("Delete {} file(s)?", paths.len()));
                    ui.label(
                        egui::RichText::new("Cannot be undone!")
                            .color(egui::Color32::RED)
                            .small(),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            let results = self.file_ops.delete_files(paths);
                            let success = results
                                .iter()
                                .filter(|r| matches!(r, OperationResult::Success(_)))
                                .count();
                            self.status_message = Some((
                                format!("Deleted {}/{}", success, paths.len()),
                                MessageType::Success,
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
                    ui.label(format!("Move {} file(s)?", paths.len()));
                    ui.label(egui::RichText::new(dest.display().to_string()).small());
                    ui.horizontal(|ui| {
                        if ui.button("Move").clicked() {
                            let results = self.file_ops.move_files(paths, dest);
                            let success = results
                                .iter()
                                .filter(|r| matches!(r, OperationResult::Success(_)))
                                .count();
                            self.status_message = Some((
                                format!("Moved {}/{}", success, paths.len()),
                                MessageType::Success,
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

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Status message on left
            if let Some((msg, msg_type)) = &self.status_message {
                let color = match msg_type {
                    MessageType::Info => egui::Color32::GRAY,
                    MessageType::Success => egui::Color32::from_rgb(100, 255, 100),
                    MessageType::Error => egui::Color32::RED,
                };
                ui.label(egui::RichText::new(msg).color(color));
            }

            // Selection info on right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.selected_files.is_empty() {
                    let size: u64 = self
                        .get_selected_paths()
                        .iter()
                        .filter_map(|p| fs::metadata(p).ok())
                        .map(|m| m.len())
                        .sum();
                    ui.label(format!(
                        "Selected: {} ({})",
                        self.selected_files.len(),
                        format_size(size)
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

        // Bottom panel for status bar - always anchored at bottom
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(28.0)
            .show(ctx, |ui| {
                self.render_status_bar(ui);
            });

        // Right panel for preview - resizable, always anchored to right
        if self.show_preview_panel {
            egui::SidePanel::right("preview_panel")
                .resizable(true)
                .default_width(self.preview_panel_width)
                .width_range(150.0..=400.0)
                .show(ctx, |ui| {
                    self.preview_panel_width = ui.available_width();
                    self.render_preview_panel(ui, ctx);
                });
        }

        // Central panel for main content - fills remaining space
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            self.render_folder_selection(ui);
            self.render_results_only(ui, ctx);
        });

        self.render_confirmation_dialog(ctx);
    }
}
