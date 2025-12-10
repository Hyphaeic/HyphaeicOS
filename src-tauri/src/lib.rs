// Asset loader module for downloading and caching remote assets
#[path = "assetLoader/mod.rs"]
mod asset_loader;

// Input handler module for domain-based keyboard navigation
#[path = "inputHandler/mod.rs"]
mod input_handler;

use asset_loader::{clear_asset_cache, get_asset_cache_path, is_asset_cached, load_asset};
use input_handler::{DomainNavigator, LayoutMode, ListDirection, GateDirection, Rect, WASDKey, NavigationResult};
use std::sync::Mutex;
use tauri::State;

// Global state for domain navigator
struct AppState {
    domain_navigator: Mutex<DomainNavigator>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ===== Domain Navigation Commands =====

/// Register a new domain
#[tauri::command]
fn register_domain(
    domain_id: String,
    parent_domain: Option<String>,
    layout_mode: String,
    grid_columns: Option<usize>,
    state: State<AppState>,
) -> Result<(), String> {
    let layout = match layout_mode.as_str() {
        "grid" => LayoutMode::Grid {
            columns: grid_columns.unwrap_or(3),
        },
        "list-vertical" => LayoutMode::List {
            direction: ListDirection::Vertical,
        },
        "list-horizontal" => LayoutMode::List {
            direction: ListDirection::Horizontal,
        },
        "spatial" => LayoutMode::Spatial,
        _ => return Err(format!("Unknown layout mode: {}", layout_mode)),
    };

    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.register_domain(domain_id, parent_domain, layout)
}

/// Unregister a domain
#[tauri::command]
fn unregister_domain(domain_id: String, state: State<AppState>) -> Result<(), String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.unregister_domain(&domain_id)
}

/// Register a button within a domain
#[tauri::command]
fn register_button(
    domain_id: String,
    button_id: String,
    bounds: Option<Rect>,
    order: usize,
    state: State<AppState>,
) -> Result<(), String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.register_button(domain_id, button_id, bounds, order)
}

/// Unregister a button
#[tauri::command]
fn unregister_button(
    domain_id: String,
    button_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.unregister_button(&domain_id, &button_id)
}

/// Register a gate within a domain
#[tauri::command]
fn register_gate(
    gate_id: String,
    source_domain: String,
    target_domain: String,
    direction: String,
    entry_point: Option<usize>,
    state: State<AppState>,
) -> Result<(), String> {
    let gate_dir = GateDirection::from_str(&direction)
        .ok_or_else(|| format!("Invalid gate direction: {}", direction))?;

    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.register_gate(gate_id, source_domain, target_domain, gate_dir, entry_point)
}

/// Unregister a gate
#[tauri::command]
fn unregister_gate(
    domain_id: String,
    gate_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.unregister_gate(&domain_id, &gate_id)
}

/// Set the active domain
#[tauri::command]
fn set_active_domain(domain_id: String, state: State<AppState>) -> Result<(), String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.set_active_domain(domain_id)
}

/// Get the current active domain ID
#[tauri::command]
fn get_active_domain(state: State<AppState>) -> Result<Option<String>, String> {
    let navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    Ok(navigator.get_active_domain_id())
}

/// Handle WASD keyboard input
#[tauri::command]
fn handle_wasd_input(key: String, state: State<AppState>) -> Result<NavigationResult, String> {
    let wasd_key = WASDKey::from_str(&key)
        .ok_or_else(|| format!("Invalid WASD key: {}", key))?;

    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    Ok(navigator.handle_wasd_input(wasd_key))
}

/// Switch to the domain at the current gate
#[tauri::command]
fn switch_domain(state: State<AppState>) -> Result<NavigationResult, String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    Ok(navigator.switch_domain())
}

/// Get current cursor position
#[tauri::command]
fn get_cursor_position(state: State<AppState>) -> Result<serde_json::Value, String> {
    let navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    match navigator.get_cursor_position() {
        Some(pos) => serde_json::to_value(pos)
            .map_err(|e| format!("Serialization error: {}", e)),
        None => Ok(serde_json::Value::Null),
    }
}

/// Get all domain IDs (for debugging)
#[tauri::command]
fn get_all_domains(state: State<AppState>) -> Result<Vec<String>, String> {
    let navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    Ok(navigator.get_all_domain_ids())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize application state
    let app_state = AppState {
        domain_navigator: Mutex::new(DomainNavigator::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Original commands
            greet,
            load_asset,
            clear_asset_cache,
            is_asset_cached,
            get_asset_cache_path,
            // Domain navigation commands
            register_domain,
            unregister_domain,
            register_button,
            unregister_button,
            register_gate,
            unregister_gate,
            set_active_domain,
            get_active_domain,
            handle_wasd_input,
            switch_domain,
            get_cursor_position,
            get_all_domains,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
