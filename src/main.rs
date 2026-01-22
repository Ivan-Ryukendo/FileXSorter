//! File X Sorter - A lightweight Windows file duplicate detection tool
//!
//! This application helps users find and manage duplicate files using
//! name and hash-based detection with a clean GUI interface.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod file_ops;
mod scanner;

use app::FileXSorterApp;

fn main() -> eframe::Result<()> {
    // Initialize logging in debug mode
    #[cfg(debug_assertions)]
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("File X Sorter"),
        ..Default::default()
    };

    eframe::run_native(
        "File X Sorter",
        options,
        Box::new(|cc| Ok(Box::new(FileXSorterApp::new(cc)))),
    )
}
