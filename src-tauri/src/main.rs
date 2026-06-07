#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;
mod compress;
mod pdf_tools;
mod png;
mod svg;

use app_state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_png_inputs,
            commands::resolve_dropped_input,
            commands::start_compress_job,
            commands::cancel_compress_job,
            commands::open_path,
            commands::export_svg_image,
            commands::export_svg_icon_set,
            commands::images_to_pdf,
            commands::pdf_to_images,
            commands::merge_pdfs,
            commands::split_pdf,
            commands::watermark_pdf
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").expect("main window");
            let _ = main_window.center();
            main_window.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
