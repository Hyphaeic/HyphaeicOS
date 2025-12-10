// Asset loader module for downloading and caching remote assets
#[path = "assetLoader/mod.rs"]
mod asset_loader;

use asset_loader::{clear_asset_cache, get_asset_cache_path, is_asset_cached, load_asset};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            load_asset,
            clear_asset_cache,
            is_asset_cached,
            get_asset_cache_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
