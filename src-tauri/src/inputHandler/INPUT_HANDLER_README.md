# Domain Navigation Input Handler

## Overview

This is a domain-based keyboard navigation system for WASD-controlled cursor movement. It provides a visual cursor that moves between interactive elements within different UI "domains" (containers), with the ability to switch between domains via "gates".

## Architecture

### Core Concepts

1. **Domain**: A container component with its own collection of navigable elements (buttons, gates). Each domain has a layout mode that determines navigation behavior.

2. **Button**: An interactive element within a domain that the cursor can focus on.

3. **Gate**: A special boundary element that allows switching from one domain to another. When the cursor reaches a gate, the user can activate it to switch domains.

4. **Cursor**: A visual highlight that indicates which element is currently focused. The cursor is managed entirely by Rust and emitted to the frontend for rendering.

5. **Layout Modes**:
   - **Grid**: Elements arranged in rows/columns (specify number of columns)
   - **List**: Elements arranged linearly (vertical or horizontal)
   - **Spatial**: Free-form positioning using actual screen coordinates

## File Structure

```
src-tauri/src/inputHandler/
├── mod.rs                    # Module exports
├── types.rs                  # Data structures and types
├── domain_navigator.rs       # Main navigation logic
└── spatial.rs               # Spatial navigation algorithms
```

## Key Types

### `DomainNavigator`
The main state manager that tracks all domains, buttons, gates, and cursor position.

### `WASDKey`
Enum representing WASD keys for navigation:
- `W` - Up
- `A` - Left  
- `S` - Down
- `D` - Right

### `NavigationResult`
Result enum returned by navigation operations:
- `CursorMoved { domain_id, element_id, element_type }` - Cursor moved to new element
- `AtGate { gate_id, target_domain }` - Cursor at gate, ready to switch
- `BoundaryReached` - Hit edge of domain
- `NoActiveDomain` - No active domain set
- `DomainSwitched { from_domain, to_domain, new_element_id }` - Domain switch successful
- `Error { message }` - Error occurred

## Tauri Commands

### Domain Management

#### `register_domain(domain_id, parent_domain, layout_mode, grid_columns)`
Register a new domain container.
- `layout_mode`: "grid", "list-vertical", "list-horizontal", or "spatial"
- `grid_columns`: Number of columns (required for grid mode)

#### `unregister_domain(domain_id)`
Remove a domain and all its elements.

#### `set_active_domain(domain_id)`
Set which domain is currently receiving input.

#### `get_active_domain()`
Get the ID of the currently active domain.

### Element Management

#### `register_button(domain_id, button_id, bounds, order)`
Add a button to a domain.
- `bounds`: Optional `{x, y, width, height}` for spatial navigation
- `order`: Sequential position for list/grid layouts

#### `unregister_button(domain_id, button_id)`
Remove a button from a domain.

#### `register_gate(gate_id, source_domain, target_domain, direction, entry_point)`
Add a gate for domain switching.
- `direction`: "top", "bottom", "left", or "right"
- `entry_point`: Optional index of element to focus on when entering target domain

#### `unregister_gate(domain_id, gate_id)`
Remove a gate from a domain.

### Navigation

#### `handle_wasd_input(key)`
Process WASD keyboard input.
- `key`: "W", "A", "S", or "D"
- Returns: `NavigationResult`

#### `switch_domain()`
Switch to the target domain when cursor is at a gate.
- Returns: `NavigationResult::DomainSwitched` on success

#### `get_cursor_position()`
Get current cursor position.
- Returns: `{ domain_id, element_id, element_type }` or null

#### `get_all_domains()`
Get list of all registered domain IDs (for debugging).

## Usage Flow

### 1. Setup Domain

```rust
// Frontend calls:
invoke('register_domain', {
  domainId: 'main-menu',
  parentDomain: null,
  layoutMode: 'list-vertical',
  gridColumns: null
});
```

### 2. Add Elements

```rust
// Add buttons
invoke('register_button', {
  domainId: 'main-menu',
  buttonId: 'btn-start',
  bounds: { x: 100, y: 100, width: 200, height: 50 },
  order: 0
});

invoke('register_button', {
  domainId: 'main-menu',
  buttonId: 'btn-options',
  bounds: { x: 100, y: 160, width: 200, height: 50 },
  order: 1
});

// Add gate to another domain
invoke('register_gate', {
  gateId: 'gate-to-game',
  sourceDomain: 'main-menu',
  targetDomain: 'game-screen',
  direction: 'bottom',
  entryPoint: 0
});
```

### 3. Handle Navigation

```rust
// User presses S (down)
let result = invoke('handle_wasd_input', { key: 'S' });

// Result: { type: 'CursorMoved', domain_id: 'main-menu', element_id: 'btn-options', ... }

// User presses S again (now at gate)
let result = invoke('handle_wasd_input', { key: 'S' });

// Result: { type: 'AtGate', gate_id: 'gate-to-game', target_domain: 'game-screen' }

// User activates gate (e.g., presses Enter/Space)
let result = invoke('switch_domain');

// Result: { type: 'DomainSwitched', from_domain: 'main-menu', to_domain: 'game-screen', ... }
```

## Navigation Algorithms

### Grid Layout
Navigation calculates row/column positions and moves accordingly. Boundaries prevent moving outside the grid.

### List Layout
Simple sequential navigation. Vertical lists use W/S, horizontal lists use A/D.

### Spatial Layout
Uses actual screen coordinates to find the nearest element in the direction of travel. Algorithm:
1. Filter elements that are in the correct direction
2. Calculate distance with directional weighting
3. Prioritize elements directly aligned with the movement axis
4. Select nearest valid element

## Testing

Run the built-in tests:
```bash
cd src-tauri
cargo test
```

Tests include:
- Domain registration
- Button registration and auto-focus
- List navigation (vertical)
- Grid navigation (all directions)
- Boundary detection
- Directional filtering for spatial navigation

## Future Enhancements

- [ ] Domain stacking (modals on top of other domains)
- [ ] Automatic bounds calculation from frontend
- [ ] Focus history/breadcrumbs
- [ ] Gamepad support
- [ ] Mouse hover integration
- [ ] Custom key bindings
- [ ] Animated cursor transitions
- [ ] Wrapping navigation (connect edges)

## Integration with Frontend

The frontend should:
1. Register domains when components mount
2. Register buttons/gates with their screen positions
3. Listen for navigation events (cursor-moved, at-gate, domain-activated)
4. Render visual cursor highlight based on Rust state
5. Unregister elements when components unmount

Example frontend integration coming in separate documentation.
