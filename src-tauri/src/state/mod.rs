use std::collections::HashMap;
use uuid::Uuid;
use self::window::{WindowInstance, WindowState, CompositorSlot};

pub mod window;

pub struct StateManager {
    pub windows: HashMap<String, WindowInstance>,
    pub window_stack: Vec<String>, // Ordered list of IDs for focus history
    pub left_slot: Option<String>,  // Window ID in left slot
    pub right_slot: Option<String>, // Window ID in right slot
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            window_stack: Vec::new(),
            left_slot: None,
            right_slot: None,
        }
    }

    /// Spawn a new window in the first available slot
    /// Returns None if both slots are occupied
    pub fn spawn_window(&mut self, content_key: String, source_element_id: Option<String>, source_domain_id: Option<String>) -> Option<WindowInstance> {
        // Find first available slot (left first, then right)
        let slot = if self.left_slot.is_none() {
            CompositorSlot::Left
        } else if self.right_slot.is_none() {
            CompositorSlot::Right
        } else {
            // Both slots occupied - cannot spawn
            return None;
        };

        let id = Uuid::new_v4().to_string();
        let title = format!("Window - {}", content_key);
        let z_order = (self.window_stack.len() as u32) + 1;

        let window = WindowInstance {
            id: id.clone(),
            content_key,
            title,
            state: WindowState::Minimized, // Default to half-size (Minimized)
            slot,
            z_order,
            source_element_id,
            source_domain_id,
        };

        // Assign to slot
        match slot {
            CompositorSlot::Left => self.left_slot = Some(id.clone()),
            CompositorSlot::Right => self.right_slot = Some(id.clone()),
        }

        self.windows.insert(id.clone(), window.clone());
        self.window_stack.push(id);
        
        Some(window)
    }

    /// Close a window and free its slot
    /// Returns the window instance that was closed (useful for retrieving source_id)
    pub fn close_window(&mut self, id: &str) -> Option<WindowInstance> {
        // Free the slot
        if self.left_slot.as_deref() == Some(id) {
            self.left_slot = None;
        } else if self.right_slot.as_deref() == Some(id) {
            self.right_slot = None;
        }

        // Remove from stack
        if let Some(index) = self.window_stack.iter().position(|x| x == id) {
            self.window_stack.remove(index);
        }
        
        let removed_window = self.windows.remove(id);
        self.normalize_stack();
        
        removed_window
    }

    /// Set window state (Minimized = half, Maximized = full, Hidden = not shown)
    pub fn set_window_state(&mut self, id: &str, new_state: WindowState) -> Option<WindowInstance> {
        if let Some(win) = self.windows.get_mut(id) {
            win.state = new_state;
            Some(win.clone())
        } else {
            None
        }
    }

    /// Check if a slot is available
    pub fn is_slot_available(&self, slot: CompositorSlot) -> bool {
        match slot {
            CompositorSlot::Left => self.left_slot.is_none(),
            CompositorSlot::Right => self.right_slot.is_none(),
        }
    }

    /// Get window in a specific slot
    pub fn get_window_in_slot(&self, slot: CompositorSlot) -> Option<&WindowInstance> {
        let id = match slot {
            CompositorSlot::Left => self.left_slot.as_ref(),
            CompositorSlot::Right => self.right_slot.as_ref(),
        };
        id.and_then(|id| self.windows.get(id))
    }

    pub fn get_all_windows(&self) -> Vec<WindowInstance> {
        self.windows.values().cloned().collect()
    }

    fn normalize_stack(&mut self) {
        for (i, win_id) in self.window_stack.iter().enumerate() {
            if let Some(win) = self.windows.get_mut(win_id) {
                win.z_order = (i as u32) + 1;
            }
        }
    }
}
