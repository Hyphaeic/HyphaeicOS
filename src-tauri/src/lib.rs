// Asset loader module for downloading and caching remote assets
#[path = "assetLoader/mod.rs"]
mod asset_loader;

// Input handler module for domain-based keyboard navigation
#[path = "inputHandler/mod.rs"]
mod input_handler;

// State management module
mod state;

use asset_loader::{clear_asset_cache, get_asset_cache_path, is_asset_cached, load_asset};
use input_handler::{DomainNavigator, LayoutMode, ListDirection, GateDirection, Rect, WASDKey, NavigationResult, ElementType};
use state::StateManager;
use state::window::{WindowInstance, WindowState};
use std::sync::{Mutex, Arc};
use tauri::{State, Manager, AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use serde::Serialize;

// Event payload types for frontend communication
#[derive(Clone, Serialize)]
struct CursorMovedPayload {
    domain_id: String,
    element_id: String,
    element_type: String,
}

#[derive(Clone, Serialize)]
struct AtGatePayload {
    gate_id: String,
    target_domain: String,
}

#[derive(Clone, Serialize)]
struct DomainSwitchedPayload {
    from_domain: String,
    to_domain: String,
    new_element_id: String,
}

#[derive(Clone, Serialize)]
struct BoundaryReachedPayload {
    direction: String,
}

// Global state for domain navigator (Arc for sharing with shortcut handlers)
struct AppState {
    domain_navigator: Arc<Mutex<DomainNavigator>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ===== Window Management Commands =====

#[tauri::command]
fn spawn_window(
    content_key: String,
    source_element_id: Option<String>,
    source_domain_id: Option<String>,
    app: AppHandle,
    state: State<Mutex<StateManager>>,
) -> Result<WindowInstance, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    
    match manager.spawn_window(content_key, source_element_id, source_domain_id) {
        Some(window) => {
            // Emit event
            app.emit("window-created", window.clone())
                .map_err(|e| e.to_string())?;
            Ok(window)
        }
        None => Err("No available slots - both compositor slots are occupied".to_string())
    }
}

#[tauri::command]
fn close_window(
    id: String,
    app: AppHandle,
    state: State<Mutex<StateManager>>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    
    // First, set window state to Closing (triggers animation)
    if let Some(window) = manager.set_window_state(&id, WindowState::Closing) {
        // Emit state change event so frontend updates
        app.emit("window-state-changed", window)
            .map_err(|e| e.to_string())?;
    } else {
        return Err(format!("Window {} not found", id));
    }
    
    Ok(())
}

#[tauri::command]
fn remove_window(
    id: String,
    app: AppHandle,
    state: State<Mutex<StateManager>>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    let closed_window = manager.close_window(&id);
    
    // Emit event
    app.emit("window-closed", id)
        .map_err(|e| e.to_string())?;
    
    // If window had a source element, try to return focus to it
    if let Some(win) = closed_window {
        if let (Some(source_domain), Some(source_element)) = (win.source_domain_id, win.source_element_id) {
            // We use a small delay to ensure the DOM update has processed the window removal
            // This is handled via event emission to the frontend controller
            app.emit("return-focus", CursorMovedPayload {
                domain_id: source_domain,
                element_id: source_element,
                element_type: "Button".to_string(), // Assuming button triggered it
            }).map_err(|e| e.to_string())?;
        }
    }
        
    Ok(())
}

#[tauri::command]
fn set_window_state(
    id: String,
    window_state: String,
    app: AppHandle,
    state: State<Mutex<StateManager>>,
) -> Result<(), String> {
    let new_state = match window_state.as_str() {
        "Minimized" => WindowState::Minimized,
        "Maximized" => WindowState::Maximized,
        "Hidden" => WindowState::Hidden,
        "Closing" => WindowState::Closing,
        _ => return Err(format!("Invalid window state: {}", window_state)),
    };

    let mut manager = state.lock().map_err(|e| e.to_string())?;
    
    if let Some(window) = manager.set_window_state(&id, new_state) {
        app.emit("window-state-changed", window)
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Window not found: {}", id))
    }
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
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    println!("[TAURI CMD] register_button called: domain={}, button={}, order={}", domain_id, button_id, order);
    
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    // Get cursor position before registration
    let cursor_before = navigator.get_cursor_position();
    println!("[TAURI CMD] Cursor before: {:?}", cursor_before);

    // Register the button
    navigator.register_button(domain_id.clone(), button_id.clone(), bounds, order)?;

    // Check if cursor was restored (position changed to this button)
    let cursor_after = navigator.get_cursor_position();
    println!("[TAURI CMD] Cursor after: {:?}", cursor_after);
    
    if let Some(cursor) = &cursor_after {
        // If cursor changed and is now on this button, emit event
        let cursor_changed = match &cursor_before {
            Some(before) => before.element_id != cursor.element_id || before.domain_id != cursor.domain_id,
            None => true,
        };
        
        println!("[TAURI CMD] Cursor changed: {}, matches button: {}", cursor_changed, cursor.element_id == button_id);
        
        if cursor_changed && cursor.element_id == button_id && cursor.domain_id == domain_id {
            println!("[TAURI CMD] ✓ EMITTING cursor-moved event for {}", button_id);
            let type_str = match cursor.element_type {
                ElementType::Button => "Button",
                ElementType::Gate => "Gate",
            };
            let _ = app.emit("cursor-moved", CursorMovedPayload {
                domain_id: cursor.domain_id.clone(),
                element_id: cursor.element_id.clone(),
                element_type: type_str.to_string(),
            });
        }
    }

    Ok(())
}

/// Unregister a button
#[tauri::command]
fn unregister_button(
    domain_id: String,
    button_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    println!("[TAURI CMD] unregister_button called: domain={}, button={}", domain_id, button_id);
    
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.unregister_button(&domain_id, &button_id)
}

/// Update button bounds without unregistering (used during resize)
#[tauri::command]
fn update_button_bounds(
    domain_id: String,
    button_id: String,
    bounds: Option<Rect>,
    state: State<AppState>,
) -> Result<(), String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    navigator.update_button_bounds(&domain_id, &button_id, bounds)
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

/// Handle WASD keyboard input - processes navigation and emits events to frontend
#[tauri::command]
fn handle_wasd_input(key: String, app: AppHandle, state: State<AppState>) -> Result<NavigationResult, String> {
    let wasd_key = WASDKey::from_str(&key)
        .ok_or_else(|| format!("Invalid WASD key: {}", key))?;

    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    let result = navigator.handle_wasd_input(wasd_key.clone());
    
    // Emit appropriate event based on navigation result
    match &result {
        NavigationResult::CursorMoved { domain_id, element_id, element_type } => {
            let type_str = match element_type {
                ElementType::Button => "Button",
                ElementType::Gate => "Gate",
            };
            let _ = app.emit("cursor-moved", CursorMovedPayload {
                domain_id: domain_id.clone(),
                element_id: element_id.clone(),
                element_type: type_str.to_string(),
            });
        }
        NavigationResult::AtGate { gate_id, target_domain } => {
            let _ = app.emit("at-gate", AtGatePayload {
                gate_id: gate_id.clone(),
                target_domain: target_domain.clone(),
            });
        }
        NavigationResult::BoundaryReached => {
            let direction = match wasd_key {
                WASDKey::W => "up",
                WASDKey::A => "left",
                WASDKey::S => "down",
                WASDKey::D => "right",
            };
            let _ = app.emit("boundary-reached", BoundaryReachedPayload {
                direction: direction.to_string(),
            });
        }
        NavigationResult::NoActiveDomain => {
            // No event needed - this is a state issue
        }
        NavigationResult::DomainSwitched { from_domain, to_domain, new_element_id } => {
            let _ = app.emit("domain-switched", DomainSwitchedPayload {
                from_domain: from_domain.clone(),
                to_domain: to_domain.clone(),
                new_element_id: new_element_id.clone(),
            });
        }
        NavigationResult::Error { message: _ } => {
            // Errors are returned, not emitted
        }
    }
    
    Ok(result)
}

/// Toggle fullscreen mode (F11)
#[tauri::command]
fn toggle_fullscreen(app: tauri::AppHandle) -> Result<bool, String> {
    let window = app.get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    
    let is_fullscreen = window
        .is_fullscreen()
        .map_err(|e| format!("Failed to check fullscreen state: {}", e))?;

    if is_fullscreen {
        window
            .set_fullscreen(false)
            .map_err(|e| format!("Failed to exit fullscreen: {}", e))?;
        Ok(false)
    } else {
        window
            .set_fullscreen(true)
            .map_err(|e| format!("Failed to enter fullscreen: {}", e))?;
        Ok(true)
    }
}

/// Switch to the domain at the current gate - emits domain-switched event
#[tauri::command]
fn switch_domain(app: AppHandle, state: State<AppState>) -> Result<NavigationResult, String> {
    let mut navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    let result = navigator.switch_domain();
    
    // Emit event on successful domain switch
    if let NavigationResult::DomainSwitched { from_domain, to_domain, new_element_id } = &result {
        let _ = app.emit("domain-switched", DomainSwitchedPayload {
            from_domain: from_domain.clone(),
            to_domain: to_domain.clone(),
            new_element_id: new_element_id.clone(),
        });
        // Also emit cursor-moved for the new position
        let _ = app.emit("cursor-moved", CursorMovedPayload {
            domain_id: to_domain.clone(),
            element_id: new_element_id.clone(),
            element_type: "Button".to_string(),
        });
    }

    Ok(result)
}

/// Emit the current cursor position - useful for initial setup
#[tauri::command]
fn emit_cursor_position(app: AppHandle, state: State<AppState>) -> Result<bool, String> {
    let navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    if let Some(cursor) = navigator.get_cursor_position() {
        let type_str = match cursor.element_type {
            ElementType::Button => "Button",
            ElementType::Gate => "Gate",
        };
        let _ = app.emit("cursor-moved", CursorMovedPayload {
            domain_id: cursor.domain_id,
            element_id: cursor.element_id,
            element_type: type_str.to_string(),
        });
        Ok(true)
    } else {
        Ok(false)
    }
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

/// Update domain layout mode
#[tauri::command]
fn update_domain_layout(
    domain_id: String,
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

    navigator.update_layout_mode(&domain_id, layout)
}

/// Get all domain IDs (for debugging)
#[tauri::command]
fn get_all_domains(state: State<AppState>) -> Result<Vec<String>, String> {
    let navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    Ok(navigator.get_all_domain_ids())
}

/// Get detailed domain info for debugging
#[tauri::command]
fn debug_domain(domain_id: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    let navigator = state.domain_navigator.lock()
        .map_err(|e| format!("Failed to lock navigator: {}", e))?;

    match navigator.get_domain_info(&domain_id) {
        Some(domain) => serde_json::to_value(domain)
            .map_err(|e| format!("Serialization error: {}", e)),
        None => Err(format!("Domain '{}' not found", domain_id)),
    }
}

/// Helper function to process WASD navigation and emit events
fn process_wasd_navigation(app: &AppHandle, navigator: &Arc<Mutex<DomainNavigator>>, key: WASDKey) {
    let mut nav = match navigator.lock() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to lock navigator: {}", e);
            return;
        }
    };

    let result = nav.handle_wasd_input(key.clone());
    
    // Emit appropriate event based on navigation result
    match &result {
        NavigationResult::CursorMoved { domain_id, element_id, element_type } => {
            let type_str = match element_type {
                ElementType::Button => "Button",
                ElementType::Gate => "Gate",
            };
            let _ = app.emit("cursor-moved", CursorMovedPayload {
                domain_id: domain_id.clone(),
                element_id: element_id.clone(),
                element_type: type_str.to_string(),
            });
        }
        NavigationResult::AtGate { gate_id, target_domain } => {
            let _ = app.emit("at-gate", AtGatePayload {
                gate_id: gate_id.clone(),
                target_domain: target_domain.clone(),
            });
        }
        NavigationResult::BoundaryReached => {
            let direction = match key {
                WASDKey::W => "up",
                WASDKey::A => "left",
                WASDKey::S => "down",
                WASDKey::D => "right",
            };
            let _ = app.emit("boundary-reached", BoundaryReachedPayload {
                direction: direction.to_string(),
            });
        }
        _ => {}
    }
}

/// Helper function to process Enter/Space for domain switching
fn process_activate(app: &AppHandle, navigator: &Arc<Mutex<DomainNavigator>>) {
    let mut nav = match navigator.lock() {
        Ok(n) => n,
        Err(_) => return,
    };

    // Check if we're at a gate
    if let Some(cursor) = nav.get_cursor_position() {
        if cursor.element_type == ElementType::Gate {
            let result = nav.switch_domain();
            
            if let NavigationResult::DomainSwitched { from_domain, to_domain, new_element_id } = &result {
                let _ = app.emit("domain-switched", DomainSwitchedPayload {
                    from_domain: from_domain.clone(),
                    to_domain: to_domain.clone(),
                    new_element_id: new_element_id.clone(),
                });
                let _ = app.emit("cursor-moved", CursorMovedPayload {
                    domain_id: to_domain.clone(),
                    element_id: new_element_id.clone(),
                    element_type: "Button".to_string(),
                });
            }
        } else {
            // Not at a gate - emit button activation event
            let _ = app.emit("button-activate", CursorMovedPayload {
                domain_id: cursor.domain_id,
                element_id: cursor.element_id,
                element_type: "Button".to_string(),
            });
        }
    }
}

/// Default shortcuts we want registered for navigation/activation
fn default_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut::new(Some(Modifiers::empty()), Code::KeyW),
        Shortcut::new(Some(Modifiers::empty()), Code::KeyA),
        Shortcut::new(Some(Modifiers::empty()), Code::KeyS),
        Shortcut::new(Some(Modifiers::empty()), Code::KeyD),
        Shortcut::new(Some(Modifiers::empty()), Code::Enter),
        Shortcut::new(Some(Modifiers::empty()), Code::Space),
    ]
}

/// Enable or disable global shortcuts (used to release bindings when window unfocused)
#[tauri::command]
fn set_global_shortcuts_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    if enabled {
        // First unregister all shortcuts to avoid "already registered" errors
        let _ = app.global_shortcut().unregister_all();
        
        let mut success_count = 0;
        let mut last_error = None;
        
        for shortcut in default_shortcuts() {
            match app.global_shortcut().register(shortcut.clone()) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("Failed to register shortcut {:?}: {}", shortcut, e);
                    last_error = Some(e);
                }
            }
        }
        
        if success_count > 0 {
            println!("Global shortcuts enabled ({} keys registered)", success_count);
            Ok(())
        } else if let Some(e) = last_error {
            Err(format!("Failed to register any shortcuts: {}", e))
        } else {
            Err("Failed to register shortcuts for unknown reason".to_string())
        }
    } else {
        // Immediately unregister all shortcuts when window loses focus
        println!("Global shortcuts disabled");
        app.global_shortcut()
            .unregister_all()
            .map_err(|e| format!("Failed to unregister shortcuts: {}", e))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize domain navigator with Arc for sharing with shortcut handlers
    let navigator = Arc::new(Mutex::new(DomainNavigator::new()));

    // Initialize application state
    let app_state = AppState {
        domain_navigator: navigator.clone(),
    };

    // Define shortcuts (no modifiers)
    let [shortcut_w, shortcut_a, shortcut_s, shortcut_d, shortcut_enter, shortcut_space] = {
        let mut list = default_shortcuts();
        let mut iter = list.drain(..);
        [
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ]
    };

    // Clone navigator for the shortcut handler closure
    let nav_for_handler = navigator.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    // Only process on key press, not release
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    // Match shortcut and process navigation
                    if shortcut == &shortcut_w {
                        process_wasd_navigation(app, &nav_for_handler, WASDKey::W);
                    } else if shortcut == &shortcut_a {
                        process_wasd_navigation(app, &nav_for_handler, WASDKey::A);
                    } else if shortcut == &shortcut_s {
                        process_wasd_navigation(app, &nav_for_handler, WASDKey::S);
                    } else if shortcut == &shortcut_d {
                        process_wasd_navigation(app, &nav_for_handler, WASDKey::D);
                    } else if shortcut == &shortcut_enter || shortcut == &shortcut_space {
                        process_activate(app, &nav_for_handler);
                    }
                })
                .build(),
        )
        .manage(app_state)
        .manage(Mutex::new(StateManager::new()))
        .setup(|_app| {
            // NOTE: Shortcuts are NOT registered here anymore.
            // Frontend controls registration via set_global_shortcuts_enabled()
            // This prevents duplicate registrations and allows proper focus/blur handling.
            println!("WASD navigation system initialized (shortcuts will register on window focus)");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Original commands
            greet,
            load_asset,
            clear_asset_cache,
            is_asset_cached,
            get_asset_cache_path,
            // Window management commands
            spawn_window,
            close_window,
            remove_window,
            set_window_state,
            // Domain navigation commands
            register_domain,
            unregister_domain,
            register_button,
            unregister_button,
            update_button_bounds,
            register_gate,
            unregister_gate,
            set_active_domain,
            get_active_domain,
            handle_wasd_input,
            switch_domain,
            get_cursor_position,
            emit_cursor_position,
            get_all_domains,
            debug_domain,
            update_domain_layout,
            toggle_fullscreen,
            set_global_shortcuts_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
