//! GUI module - Application state and UI rendering
//!
//! This module contains the main application state and egui-based UI.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use eframe::egui;
use rfd::FileDialog;

use crate::file_ops::{FileOperations, OperationResult};
use crate::scanner::{format_size, DuplicateGroup, ScanResult, Scanner, ScannerConfig};

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

/// Application state
pub struct FileXSorterApp {
    // Scan settings
    selected_folder: Option<PathBuf>,
    recursive_scan: bool,

    // Scan state
    is_scanning: bool,
    scan_result: Option<ScanResult>,
    scan_state: Arc<ScanState>,
    scan_handle: Option<JoinHandle<()>>,

    // Selection state (which files are selected for action)
    selected_files: Vec<(usize, usize)>, // (group_index, file_index)

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
            selected_folder: None,
            recursive_scan: true,
            is_scanning: false,
            scan_result: None,
            scan_state: Arc::new(ScanState::new()),
            scan_handle: None,
            selected_files: Vec::new(),
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
        if self.selected_folder.is_none() {
            self.status_message = Some((
                "Please select a folder first.".to_string(),
                MessageType::Error,
            ));
            return;
        }

        // Reset state
        self.is_scanning = true;
        self.scan_result = None;
        self.selected_files.clear();

        // Create new scan state
        self.scan_state = Arc::new(ScanState::new());

        let folder = self.selected_folder.clone().unwrap();
        let recursive = self.recursive_scan;
        let scan_state = Arc::clone(&self.scan_state);

        // Spawn background thread
        let handle = thread::spawn(move || {
            let config = ScannerConfig {
                recursive,
                min_size: 1,
            };
            let scanner = Scanner::new(config);

            // Set up progress callback-like polling mechanism
            let progress_current = &scan_state.progress_current;
            let progress_total = &scan_state.progress_total;
            let cancel_flag = &scan_state.cancel_flag;

            // We'll poll the scanner's progress in a simpler way
            // Just run the scan - the scanner handles progress internally
            let result = scanner.scan_directory_with_progress(
                &folder,
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
        self.status_message = Some(("Scanning...".to_string(), MessageType::Info));
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

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("File X Sorter");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("v0.1.0");
            });
        });
        ui.separator();
    }

    fn render_folder_selection(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Folder:");

            let folder_text = self
                .selected_folder
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "No folder selected".to_string());

            ui.add(
                egui::TextEdit::singleline(&mut folder_text.as_str())
                    .desired_width(400.0)
                    .interactive(false),
            );

            if ui.button("Browse...").clicked() && !self.is_scanning {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    self.selected_folder = Some(folder);
                    self.scan_result = None;
                    self.selected_files.clear();
                }
            }
        });

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

            // Duplicate groups list
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (group_idx, group) in result.duplicate_groups.iter().enumerate() {
                        self.render_duplicate_group(ui, group_idx, group);
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
                ui.label("Select a folder and click 'Scan for Duplicates' to begin.");
            });
        }
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

                        if ui.small_button("Open Folder").clicked() {
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
                                    self.show_confirmation_dialog = None;

                                    // Trigger rescan
                                    if self.selected_folder.is_some() {
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
                                    self.show_confirmation_dialog = None;

                                    // Trigger rescan
                                    if self.selected_folder.is_some() {
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
