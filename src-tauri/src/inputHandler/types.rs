// Core data structures for domain navigation system

use serde::{Deserialize, Serialize};

/// Represents a spatial rectangle for positioning elements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// Get the center point of the rectangle
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Calculate distance from this rect's center to a point
    pub fn distance_to_point(&self, x: f64, y: f64) -> f64 {
        let (cx, cy) = self.center();
        ((cx - x).powi(2) + (cy - y).powi(2)).sqrt()
    }
}

/// WASD input keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WASDKey {
    W, // Up
    A, // Left
    S, // Down
    D, // Right
}

impl WASDKey {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "W" => Some(WASDKey::W),
            "A" => Some(WASDKey::A),
            "S" => Some(WASDKey::S),
            "D" => Some(WASDKey::D),
            _ => None,
        }
    }

    /// Get directional unit vector
    pub fn direction_vector(&self) -> (f64, f64) {
        match self {
            WASDKey::W => (0.0, -1.0), // Up
            WASDKey::A => (-1.0, 0.0), // Left
            WASDKey::S => (0.0, 1.0),  // Down
            WASDKey::D => (1.0, 0.0),  // Right
        }
    }
}

/// Layout mode for domain navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutMode {
    /// Grid layout with specified number of columns
    Grid { columns: usize },
    /// List layout (vertical or horizontal)
    List { direction: ListDirection },
    /// Free-form spatial layout using actual coordinates
    Spatial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListDirection {
    Vertical,
    Horizontal,
}

/// Direction of a gate (which edge of the domain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateDirection {
    Top,
    Bottom,
    Left,
    Right,
}

impl GateDirection {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "top" => Some(GateDirection::Top),
            "bottom" => Some(GateDirection::Bottom),
            "left" => Some(GateDirection::Left),
            "right" => Some(GateDirection::Right),
            _ => None,
        }
    }
}

/// Type of navigable element
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    Button,
    Gate,
}

/// A navigable button element within a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonElement {
    pub id: String,
    pub bounds: Option<Rect>,
    pub enabled: bool,
    pub order: usize, // Sequential order for list/grid layouts
}

/// A gate element that allows domain switching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateElement {
    pub id: String,
    pub bounds: Option<Rect>,
    pub target_domain: String,
    pub direction: GateDirection,
    pub entry_point: Option<usize>, // Index to enter in target domain
}

/// A domain containing navigable elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub parent_id: Option<String>,
    pub buttons: Vec<ButtonElement>,
    pub gates: Vec<GateElement>,
    pub current_index: usize,
    pub layout_mode: LayoutMode,
}

impl Domain {
    pub fn new(id: String, parent_id: Option<String>, layout_mode: LayoutMode) -> Self {
        Self {
            id,
            parent_id,
            buttons: Vec::new(),
            gates: Vec::new(),
            current_index: 0,
            layout_mode,
        }
    }

    /// Get total number of navigable elements (buttons + gates)
    pub fn element_count(&self) -> usize {
        self.buttons.len() + self.gates.len()
    }

    /// Get element by index (buttons first, then gates)
    pub fn get_element_at_index(&self, index: usize) -> Option<(ElementType, String)> {
        if index < self.buttons.len() {
            Some((ElementType::Button, self.buttons[index].id.clone()))
        } else {
            let gate_index = index - self.buttons.len();
            self.gates
                .get(gate_index)
                .map(|g| (ElementType::Gate, g.id.clone()))
        }
    }

    /// Find index of element by ID
    pub fn find_element_index(&self, element_id: &str) -> Option<usize> {
        // Search buttons first
        if let Some(pos) = self.buttons.iter().position(|b| b.id == element_id) {
            return Some(pos);
        }
        // Search gates
        if let Some(pos) = self.gates.iter().position(|g| g.id == element_id) {
            return Some(self.buttons.len() + pos);
        }
        None
    }
}

/// Current cursor position in the navigation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub domain_id: String,
    pub element_id: String,
    pub element_type: ElementType,
}

/// Target of a navigation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationTarget {
    /// Move to a button
    Button(String),
    /// Reached a gate (ready to switch)
    Gate(String),
    /// Hit edge of domain with no gate
    Boundary,
}

/// Result of a navigation action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NavigationResult {
    /// Cursor moved to new element
    CursorMoved {
        domain_id: String,
        element_id: String,
        element_type: ElementType,
    },
    /// Cursor is at a gate, ready to switch
    AtGate {
        gate_id: String,
        target_domain: String,
    },
    /// Hit boundary of domain
    BoundaryReached,
    /// No active domain to navigate
    NoActiveDomain,
    /// Domain switched successfully
    DomainSwitched {
        from_domain: String,
        to_domain: String,
        new_element_id: String,
    },
    /// Error occurred
    Error { message: String },
}
