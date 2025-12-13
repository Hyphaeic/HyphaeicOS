use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum WindowState {
    /// Half-size window in its assigned slot
    Minimized,
    /// Full-size window spanning entire compositor
    Maximized,
    /// Window is hidden (not rendered)
    Hidden,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum CompositorSlot {
    Left,
    Right,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WindowInstance {
    pub id: String,              // Unique UUID
    pub content_key: String,     // What to render (e.g., "SYS_TERMINAL")
    pub title: String,           // Window Title
    pub state: WindowState,      // Minimized, Maximized, Hidden
    pub slot: CompositorSlot,    // Which slot this window occupies
    pub z_order: u32,            // Stacking order (for future overlapping)
    pub source_element_id: Option<String>, // ID of element that spawned this window
    pub source_domain_id: Option<String>,  // ID of domain that spawned this window
}
