// Main domain navigation logic

use super::spatial::{find_nearest_in_direction, navigate_grid, navigate_list};
use super::types::*;
use std::collections::HashMap;

/// Main domain navigation state manager
pub struct DomainNavigator {
    domains: HashMap<String, Domain>,
    active_domain_id: Option<String>,
    cursor_position: Option<CursorPosition>,
    /// Saved cursor positions for domains that were unregistered
    /// Used to restore cursor when domain re-registers (e.g., window state change)
    saved_cursor_positions: HashMap<String, CursorPosition>,
    /// Saved active domain ID when it gets unregistered
    saved_active_domain: Option<String>,
}

impl DomainNavigator {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
            active_domain_id: None,
            cursor_position: None,
            saved_cursor_positions: HashMap::new(),
            saved_active_domain: None,
        }
    }

    /// Register a new domain
    pub fn register_domain(
        &mut self,
        domain_id: String,
        parent_id: Option<String>,
        layout_mode: LayoutMode,
    ) -> Result<(), String> {
        if self.domains.contains_key(&domain_id) {
            return Err(format!("Domain '{}' already exists", domain_id));
        }

        let domain = Domain::new(domain_id.clone(), parent_id, layout_mode);
        self.domains.insert(domain_id.clone(), domain);

        // Check if this domain was previously active and restore it
        if self.saved_active_domain.as_ref() == Some(&domain_id) {
            self.active_domain_id = Some(domain_id.clone());
            self.saved_active_domain = None;
            
            // Restore cursor position if we have one saved
            if let Some(saved_cursor) = self.saved_cursor_positions.remove(&domain_id) {
                // Don't set cursor yet - wait for buttons to register
                // Store it back temporarily until buttons are registered
                self.saved_cursor_positions.insert(domain_id.clone(), saved_cursor);
            }
        }
        // If this is the first domain and no active domain, make it active
        else if self.active_domain_id.is_none() {
            self.active_domain_id = Some(domain_id);
        }

        Ok(())
    }

    /// Unregister a domain
    pub fn unregister_domain(&mut self, domain_id: &str) -> Result<(), String> {
        println!("[UNREGISTER_DOMAIN] domain: {}", domain_id);
        
        if !self.domains.contains_key(domain_id) {
            return Err(format!("Domain '{}' not found", domain_id));
        }

        // If cursor was in this domain, save it for restoration
        if let Some(cursor) = &self.cursor_position {
            if cursor.domain_id == domain_id {
                self.saved_cursor_positions.insert(domain_id.to_string(), cursor.clone());
            }
        }

        // If this was the active domain, save it and clear active state
        if self.active_domain_id.as_ref() == Some(&domain_id.to_string()) {
            self.saved_active_domain = Some(domain_id.to_string());
            self.active_domain_id = None;
            self.cursor_position = None;
        }

        self.domains.remove(domain_id);
        
        // Clean up saved cursor for this domain since it no longer exists
        // This prevents stale entries from causing issues
        self.saved_cursor_positions.remove(domain_id);
        println!("[UNREGISTER_DOMAIN] Cleaned up saved cursor, remaining: {:?}", self.saved_cursor_positions.keys().collect::<Vec<_>>());
        
        Ok(())
    }

    /// Register a button within a domain
    pub fn register_button(
        &mut self,
        domain_id: String,
        button_id: String,
        bounds: Option<Rect>,
        order: usize,
    ) -> Result<(), String> {
        println!("[REGISTER_BUTTON] domain: {}, button: {}, order: {}", domain_id, button_id, order);
        println!("[REGISTER_BUTTON] Active domain: {:?}", self.active_domain_id);
        println!("[REGISTER_BUTTON] Current cursor: {:?}", self.cursor_position);
        println!("[REGISTER_BUTTON] Saved cursors: {:?}", self.saved_cursor_positions);
        
        let domain = self.domains.get_mut(&domain_id)
            .ok_or_else(|| format!("Domain '{}' not found", domain_id))?;

        // Check if button already exists
        if domain.buttons.iter().any(|b| b.id == button_id) {
            return Err(format!("Button '{}' already exists in domain '{}'", button_id, domain_id));
        }

        let button = ButtonElement {
            id: button_id.clone(),
            bounds,
            enabled: true,
            order,
        };

        domain.buttons.push(button);

        // Sort buttons by order
        domain.buttons.sort_by_key(|b| b.order);

        println!("[REGISTER_BUTTON] Domain now has {} buttons", domain.buttons.len());

        // Check if we have a saved cursor position for this domain
        if self.active_domain_id.as_ref() == Some(&domain_id) {
            if let Some(saved_cursor) = self.saved_cursor_positions.get(&domain_id) {
                println!("[REGISTER_BUTTON] Found saved cursor: {:?}", saved_cursor);
                // If this is the button we were on, restore cursor
                if saved_cursor.element_id == button_id {
                    println!("[REGISTER_BUTTON] ✓ RESTORING cursor to {}", button_id);
                    self.cursor_position = Some(CursorPosition {
                        domain_id: domain_id.clone(),
                        element_id: button_id.clone(),
                        element_type: ElementType::Button,
                    });
                    // Remove saved cursor since we've restored it
                    self.saved_cursor_positions.remove(&domain_id);
                    return Ok(());
                } else {
                    // There's a saved cursor waiting for a different button
                    // Don't set cursor to first element - wait for the correct button to register
                    println!("[REGISTER_BUTTON] Saved cursor exists for different button, waiting...");
                    return Ok(());
                }
            }
            
            // If no cursor position and no saved cursor and this is the first element, set cursor to it
            if self.cursor_position.is_none() && domain.element_count() == 1 {
                println!("[REGISTER_BUTTON] ✓ Setting cursor to first element: {}", button_id);
                self.cursor_position = Some(CursorPosition {
                    domain_id: domain_id.clone(),
                    element_id: button_id,
                    element_type: ElementType::Button,
                });
            }
        }

        println!("[REGISTER_BUTTON] Final cursor: {:?}", self.cursor_position);
        Ok(())
    }

    /// Unregister a button
    pub fn unregister_button(&mut self, domain_id: &str, button_id: &str) -> Result<(), String> {
        println!("[UNREGISTER_BUTTON] domain: {}, button: {}", domain_id, button_id);
        println!("[UNREGISTER_BUTTON] Current cursor: {:?}", self.cursor_position);
        
        let domain = self.domains.get_mut(domain_id)
            .ok_or_else(|| format!("Domain '{}' not found", domain_id))?;

        let index = domain.buttons.iter().position(|b| b.id == button_id)
            .ok_or_else(|| format!("Button '{}' not found in domain '{}'", button_id, domain_id))?;

        // If cursor was on this button, save it for restoration when button re-registers
        // (e.g., during resize, window state change, etc.)
        if let Some(cursor) = &self.cursor_position {
            if cursor.domain_id == domain_id && cursor.element_id == button_id {
                println!("[UNREGISTER_BUTTON] ✓ SAVING cursor position for {}", button_id);
                // Save cursor position for this domain
                self.saved_cursor_positions.insert(domain_id.to_string(), cursor.clone());
                // Clear current cursor since button no longer exists
                // It will be restored when button re-registers
                self.cursor_position = None;
            }
        }

        domain.buttons.remove(index);
        println!("[UNREGISTER_BUTTON] Domain now has {} buttons", domain.buttons.len());
        println!("[UNREGISTER_BUTTON] Saved cursors: {:?}", self.saved_cursor_positions);

        Ok(())
    }

    /// Update button bounds without unregistering (used during resize)
    /// This avoids the cursor save/restore dance and is much simpler
    pub fn update_button_bounds(
        &mut self,
        domain_id: &str,
        button_id: &str,
        bounds: Option<Rect>,
    ) -> Result<(), String> {
        let domain = self.domains.get_mut(domain_id)
            .ok_or_else(|| format!("Domain '{}' not found", domain_id))?;

        let button = domain.buttons.iter_mut()
            .find(|b| b.id == button_id)
            .ok_or_else(|| format!("Button '{}' not found in domain '{}'", button_id, domain_id))?;

        button.bounds = bounds;
        Ok(())
    }

    /// Register a gate within a domain
    pub fn register_gate(
        &mut self,
        gate_id: String,
        source_domain: String,
        target_domain: String,
        direction: GateDirection,
        entry_point: Option<usize>,
    ) -> Result<(), String> {
        let domain = self.domains.get_mut(&source_domain)
            .ok_or_else(|| format!("Source domain '{}' not found", source_domain))?;

        // Check if gate already exists
        if domain.gates.iter().any(|g| g.id == gate_id) {
            return Err(format!("Gate '{}' already exists in domain '{}'", gate_id, source_domain));
        }

        let gate = GateElement {
            id: gate_id,
            bounds: None,
            target_domain,
            direction,
            entry_point,
        };

        domain.gates.push(gate);

        Ok(())
    }

    /// Unregister a gate
    pub fn unregister_gate(&mut self, domain_id: &str, gate_id: &str) -> Result<(), String> {
        let domain = self.domains.get_mut(domain_id)
            .ok_or_else(|| format!("Domain '{}' not found", domain_id))?;

        let index = domain.gates.iter().position(|g| g.id == gate_id)
            .ok_or_else(|| format!("Gate '{}' not found in domain '{}'", gate_id, domain_id))?;

        domain.gates.remove(index);

        Ok(())
    }

    /// Set the active domain
    pub fn set_active_domain(&mut self, domain_id: String) -> Result<(), String> {
        if !self.domains.contains_key(&domain_id) {
            return Err(format!("Domain '{}' not found", domain_id));
        }

        self.active_domain_id = Some(domain_id.clone());

        // Set cursor to first element if available
        if let Some(domain) = self.domains.get(&domain_id) {
            if let Some((element_type, element_id)) = domain.get_element_at_index(0) {
                self.cursor_position = Some(CursorPosition {
                    domain_id,
                    element_id,
                    element_type,
                });
            }
        }

        Ok(())
    }

    /// Get current cursor position
    pub fn get_cursor_position(&self) -> Option<CursorPosition> {
        self.cursor_position.clone()
    }

    /// Get active domain ID
    pub fn get_active_domain_id(&self) -> Option<String> {
        self.active_domain_id.clone()
    }

    /// Explicitly set the cursor position (e.g. from mouse hover)
    pub fn set_cursor_position(&mut self, domain_id: &str, element_id: &str) -> Result<ElementType, String> {
        // Verify domain exists
        let domain = self.domains.get(domain_id)
            .ok_or_else(|| format!("Domain '{}' not found", domain_id))?;

        // Verify element exists and get its type
        let element_type = if domain.buttons.iter().any(|b| b.id == element_id) {
            ElementType::Button
        } else if domain.gates.iter().any(|g| g.id == element_id) {
            ElementType::Gate
        } else {
            return Err(format!("Element '{}' not found in domain '{}'", element_id, domain_id));
        };

        // Update active domain
        self.active_domain_id = Some(domain_id.to_string());

        // Update cursor position
        self.cursor_position = Some(CursorPosition {
            domain_id: domain_id.to_string(),
            element_id: element_id.to_string(),
            element_type: element_type.clone(),
        });

        Ok(element_type)
    }

    /// Handle WASD input and navigate
    pub fn handle_wasd_input(&mut self, key: WASDKey) -> NavigationResult {
        let Some(active_domain_id) = self.active_domain_id.clone() else {
            return NavigationResult::NoActiveDomain;
        };

        // First, calculate the next index without holding a borrow
        let (element_count, current_index, layout_mode) = {
            let Some(domain) = self.domains.get(&active_domain_id) else {
                return NavigationResult::Error {
                    message: format!("Active domain '{}' not found", active_domain_id),
                };
            };

            if domain.element_count() == 0 {
                return NavigationResult::BoundaryReached;
            }

            let current_index = if let Some(cursor) = &self.cursor_position {
                domain.find_element_index(&cursor.element_id).unwrap_or(0)
            } else {
                0
            };

            (domain.element_count(), current_index, domain.layout_mode.clone())
        };

        // Navigate based on layout mode
        let next_index = match &layout_mode {
            LayoutMode::Grid { columns } => {
                navigate_grid(current_index, element_count, *columns, key)
            }
            LayoutMode::List { direction } => {
                let is_vertical = matches!(direction, ListDirection::Vertical);
                navigate_list(current_index, element_count, is_vertical, key)
            }
            LayoutMode::Spatial => {
                // For spatial, we need to access the domain again
                let domain = self.domains.get(&active_domain_id).unwrap();
                self.navigate_spatial(domain, current_index, key)
            }
        };

        // Update cursor position
        if let Some(new_index) = next_index {
            // Get element info and gate info before updating
            let element_info = {
                let domain = self.domains.get(&active_domain_id).unwrap();
                domain.get_element_at_index(new_index)
            };

            if let Some((element_type, element_id)) = element_info {
                // Get gate target if this is a gate
                let gate_target = if element_type == ElementType::Gate {
                    let domain = self.domains.get(&active_domain_id).unwrap();
                    domain.gates.iter()
                        .find(|g| g.id == element_id)
                        .map(|gate| gate.target_domain.clone())
                } else {
                    None
                };

                // Now update the domain's current index
                if let Some(domain_mut) = self.domains.get_mut(&active_domain_id) {
                    domain_mut.current_index = new_index;
                }

                // Update cursor position
                self.cursor_position = Some(CursorPosition {
                    domain_id: active_domain_id.clone(),
                    element_id: element_id.clone(),
                    element_type: element_type.clone(),
                });

                // Check if we're at a gate
                if let Some(target_domain) = gate_target {
                    return NavigationResult::AtGate {
                        gate_id: element_id,
                        target_domain,
                    };
                }

                return NavigationResult::CursorMoved {
                    domain_id: active_domain_id,
                    element_id,
                    element_type,
                };
            }
        }

        NavigationResult::BoundaryReached
    }

    /// Navigate using spatial positioning
    fn navigate_spatial(&self, domain: &Domain, current_index: usize, direction: WASDKey) -> Option<usize> {
        // Get current element bounds
        let current_element = if current_index < domain.buttons.len() {
            domain.buttons[current_index].bounds?
        } else {
            let gate_index = current_index - domain.buttons.len();
            domain.gates.get(gate_index)?.bounds?
        };

        // Collect all candidate elements with bounds
        let mut candidates: Vec<(String, Rect)> = Vec::new();
        
        for (idx, button) in domain.buttons.iter().enumerate() {
            if idx != current_index {
                if let Some(bounds) = button.bounds {
                    candidates.push((button.id.clone(), bounds));
                }
            }
        }

        for (idx, gate) in domain.gates.iter().enumerate() {
            let gate_idx = domain.buttons.len() + idx;
            if gate_idx != current_index {
                if let Some(bounds) = gate.bounds {
                    candidates.push((gate.id.clone(), bounds));
                }
            }
        }

        // Find nearest element in direction
        let nearest_id = find_nearest_in_direction(&current_element, &candidates, direction)?;

        // Find the index of this element
        domain.find_element_index(&nearest_id)
    }

    /// Switch to the domain at the current gate
    pub fn switch_domain(&mut self) -> NavigationResult {
        let Some(cursor) = &self.cursor_position.clone() else {
            return NavigationResult::Error {
                message: "No cursor position".to_string(),
            };
        };

        // Must be at a gate to switch
        if cursor.element_type != ElementType::Gate {
            return NavigationResult::Error {
                message: "Not at a gate".to_string(),
            };
        }

        // Find the gate
        let domain = match self.domains.get(&cursor.domain_id) {
            Some(d) => d,
            None => return NavigationResult::Error {
                message: format!("Domain '{}' not found", cursor.domain_id),
            },
        };

        let gate = match domain.gates.iter().find(|g| g.id == cursor.element_id) {
            Some(g) => g,
            None => return NavigationResult::Error {
                message: format!("Gate '{}' not found", cursor.element_id),
            },
        };

        let target_domain_id = gate.target_domain.clone();
        let entry_point = gate.entry_point.unwrap_or(0);

        // Check target domain exists
        if !self.domains.contains_key(&target_domain_id) {
            return NavigationResult::Error {
                message: format!("Target domain '{}' not found", target_domain_id),
            };
        }

        // Get entry element in target domain
        let target_domain = self.domains.get(&target_domain_id).unwrap();
        let (element_type, element_id) = match target_domain.get_element_at_index(entry_point) {
            Some(e) => e,
            None => return NavigationResult::Error {
                message: format!("No element at entry point {} in domain '{}'", entry_point, target_domain_id),
            },
        };

        // Switch!
        let from_domain = cursor.domain_id.clone();
        self.active_domain_id = Some(target_domain_id.clone());
        self.cursor_position = Some(CursorPosition {
            domain_id: target_domain_id.clone(),
            element_id: element_id.clone(),
            element_type: element_type.clone(),
        });

        NavigationResult::DomainSwitched {
            from_domain,
            to_domain: target_domain_id,
            new_element_id: element_id,
        }
    }

    /// Get domain information for debugging
    pub fn get_domain_info(&self, domain_id: &str) -> Option<Domain> {
        self.domains.get(domain_id).cloned()
    }

    /// Get all domain IDs
    pub fn get_all_domain_ids(&self) -> Vec<String> {
        self.domains.keys().cloned().collect()
    }

    /// Update the layout mode of a domain
    pub fn update_layout_mode(&mut self, domain_id: &str, layout_mode: LayoutMode) -> Result<(), String> {
        let domain = self.domains.get_mut(domain_id)
            .ok_or_else(|| format!("Domain '{}' not found", domain_id))?;
        
        domain.layout_mode = layout_mode;
        Ok(())
    }
}

impl Default for DomainNavigator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_registration() {
        let mut nav = DomainNavigator::new();
        
        nav.register_domain(
            "test-domain".to_string(),
            None,
            LayoutMode::List { direction: ListDirection::Vertical },
        ).unwrap();

        assert_eq!(nav.get_active_domain_id(), Some("test-domain".to_string()));
    }

    #[test]
    fn test_button_registration() {
        let mut nav = DomainNavigator::new();
        
        nav.register_domain(
            "test-domain".to_string(),
            None,
            LayoutMode::List { direction: ListDirection::Vertical },
        ).unwrap();

        nav.register_button(
            "test-domain".to_string(),
            "btn-1".to_string(),
            None,
            0,
        ).unwrap();

        let cursor = nav.get_cursor_position().unwrap();
        assert_eq!(cursor.element_id, "btn-1");
        assert_eq!(cursor.element_type, ElementType::Button);
    }

    #[test]
    fn test_list_navigation() {
        let mut nav = DomainNavigator::new();
        
        nav.register_domain(
            "test-domain".to_string(),
            None,
            LayoutMode::List { direction: ListDirection::Vertical },
        ).unwrap();

        // Add 3 buttons
        for i in 0..3 {
            nav.register_button(
                "test-domain".to_string(),
                format!("btn-{}", i),
                None,
                i,
            ).unwrap();
        }

        // Should start at first button
        let cursor = nav.get_cursor_position().unwrap();
        assert_eq!(cursor.element_id, "btn-0");

        // Navigate down
        let result = nav.handle_wasd_input(WASDKey::S);
        if let NavigationResult::CursorMoved { element_id, .. } = result {
            assert_eq!(element_id, "btn-1");
        } else {
            panic!("Expected CursorMoved");
        }

        // Navigate down again
        let result = nav.handle_wasd_input(WASDKey::S);
        if let NavigationResult::CursorMoved { element_id, .. } = result {
            assert_eq!(element_id, "btn-2");
        } else {
            panic!("Expected CursorMoved");
        }

        // Try to go past end - should hit boundary
        let result = nav.handle_wasd_input(WASDKey::S);
        assert!(matches!(result, NavigationResult::BoundaryReached));
    }

    #[test]
    fn test_button_unregister_reregister_preserves_cursor() {
        let mut nav = DomainNavigator::new();
        
        nav.register_domain(
            "test-domain".to_string(),
            None,
            LayoutMode::List { direction: ListDirection::Horizontal },
        ).unwrap();

        // Add 3 buttons (like window header buttons: minimize, maximize, close)
        nav.register_button("test-domain".to_string(), "btn-min".to_string(), None, 0).unwrap();
        nav.register_button("test-domain".to_string(), "btn-max".to_string(), None, 1).unwrap();
        nav.register_button("test-domain".to_string(), "btn-close".to_string(), None, 2).unwrap();

        // Navigate to middle button (maximize)
        nav.handle_wasd_input(WASDKey::D);
        let cursor = nav.get_cursor_position().unwrap();
        assert_eq!(cursor.element_id, "btn-max");

        // Simulate resize: unregister all buttons
        nav.unregister_button("test-domain", "btn-min").unwrap();
        nav.unregister_button("test-domain", "btn-max").unwrap();
        nav.unregister_button("test-domain", "btn-close").unwrap();

        // Cursor should be cleared after unregistering the focused button
        assert!(nav.get_cursor_position().is_none());

        // Re-register all buttons (simulating re-registration after resize)
        nav.register_button("test-domain".to_string(), "btn-min".to_string(), None, 0).unwrap();
        nav.register_button("test-domain".to_string(), "btn-max".to_string(), None, 1).unwrap();
        nav.register_button("test-domain".to_string(), "btn-close".to_string(), None, 2).unwrap();

        // Cursor should be restored to the maximize button
        let cursor = nav.get_cursor_position().unwrap();
        assert_eq!(cursor.element_id, "btn-max", "Cursor should be restored to the same button after re-registration");
    }
}

