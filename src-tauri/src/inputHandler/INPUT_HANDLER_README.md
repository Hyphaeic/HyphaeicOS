# Domain Navigation Input Handler

## Overview

This is a domain-based keyboard navigation system for WASD-controlled cursor movement. It provides a visual cursor that moves between interactive elements within different UI "domains" (containers), with the ability to switch between domains via "gates".

**Architecture**: **Rust-first** input capture. WASD/Enter/Space keys are captured at the OS level by Rust via `tauri-plugin-global-shortcut`. Navigation events are emitted directly from Rust to the frontend. The frontend **does not capture navigation keys** - it only listens for events. When the window blurs, global shortcuts are released so other apps can use these keys; they are re-registered on focus.

## Architecture

### Rust-First Input Capture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      OS-LEVEL KEY CAPTURE                           │
│                  (tauri-plugin-global-shortcut)                     │
│                                                                     │
│  Keys captured by Rust at OS level:                                │
│    W, A, S, D      → Navigation                                    │
│    Enter, Space    → Activation (gate switch / button press)       │
│    (Released on window blur, re-registered on focus)               │
│                                                                     │
│  Keys captured by Frontend (webview-specific):                     │
│    F11             → Toggle fullscreen (requires webview API)      │
└─────────────────────────────────────────────────────────────────────┘
```

This system uses **Rust-first input handling** where:
1. **Rust** captures WASD/Enter/Space at the OS level via global shortcuts
2. **Rust** processes navigation and emits Tauri events
3. **Frontend** listens for Tauri events and re-broadcasts as DOM events
4. **Child components** react to DOM events (unchanged API)
5. **F11 only** is captured in the frontend (webview-specific fullscreen toggle)

This creates a unidirectional data flow: `OS Key Event → Rust Processing → Event Emission → UI Update`

### Why Rust-First?

- **Guaranteed capture**: Keys work even if webview loses focus
- **Lower latency**: No IPC round-trip for key → navigation
- **Single source of truth**: All input logic in one place (Rust)
- **Consistency**: Same behavior across all platforms

### Core Concepts

1. **Domain**: A container component with its own collection of navigable elements (buttons, gates). Each domain has a layout mode that determines navigation behavior.

2. **Button**: An interactive element within a domain that the cursor can focus on.

3. **Gate**: A special boundary element that allows switching from one domain to another. When the cursor reaches a gate, the user can activate it to switch domains.

4. **Cursor**: A visual highlight that indicates which element is currently focused. The cursor is managed entirely by Rust and emitted to the frontend for rendering.

5. **Layout Modes**:
   - **Grid**: Elements arranged in rows/columns (specify number of columns)
   - **List**: Elements arranged linearly (vertical or horizontal)
   - **Spatial**: Free-form positioning using actual screen coordinates

### Event System

#### Tauri Events (Rust → Frontend)
Events emitted by Rust's `lib.rs` via `app.emit()`:

- **`cursor-moved`**: Cursor moved to a new element
  ```typescript
  { domain_id: string, element_id: string, element_type: "Button" | "Gate" }
  ```

- **`at-gate`**: Cursor reached a gate (ready to switch domains)
  ```typescript
  { gate_id: string, target_domain: string }
  ```

- **`domain-switched`**: Successfully switched from one domain to another
  ```typescript
  { from_domain: string, to_domain: string, new_element_id: string }
  ```

- **`boundary-reached`**: Cursor hit edge of domain (no movement)
  ```typescript
  { direction: "up" | "down" | "left" | "right" }
  ```

- **`button-activate`**: Enter/Space pressed on a button (not a gate)
  ```typescript
  { domain_id: string, element_id: string, element_type: "Button" }
  ```

#### DOM Events (Interface.tsx → Child Components)
The frontend relay (`Interface.tsx`) translates Tauri events to DOM `CustomEvent`s 
on `window` object, maintaining the same API for child components:

- `window.dispatchEvent(new CustomEvent('cursor-moved', { detail: {...} }))`
- `window.dispatchEvent(new CustomEvent('at-gate', { detail: {...} }))`
- `window.dispatchEvent(new CustomEvent('domain-switched', { detail: {...} }))`
- `window.dispatchEvent(new CustomEvent('boundary-reached', { detail: {...} }))`
- `window.dispatchEvent(new CustomEvent('button-activate', { detail: {...} }))`

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

## Global Shortcut Registration

The following keys are registered as global shortcuts at app startup in `lib.rs`:

```rust
// Registered in setup() via tauri-plugin-global-shortcut
Shortcut::new(Some(Modifiers::empty()), Code::KeyW)    // W - Navigate up
Shortcut::new(Some(Modifiers::empty()), Code::KeyA)    // A - Navigate left
Shortcut::new(Some(Modifiers::empty()), Code::KeyS)    // S - Navigate down
Shortcut::new(Some(Modifiers::empty()), Code::KeyD)    // D - Navigate right
Shortcut::new(Some(Modifiers::empty()), Code::Enter)   // Enter - Activate
Shortcut::new(Some(Modifiers::empty()), Code::Space)   // Space - Activate
```

When a shortcut is pressed:
1. `ShortcutState::Pressed` triggers handler
2. Handler calls `process_wasd_navigation()` or `process_activate()`
3. Navigation is processed via `DomainNavigator`
4. Events are emitted directly to frontend via `app.emit()`

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

### Utility Commands

#### `emit_cursor_position()`
Emit current cursor position as Tauri event.

**Returns**: `boolean` - `true` if cursor position exists and event was emitted

**Emits**: `cursor-moved` event with current cursor state

**Use Cases**:
- Initial UI setup after component registration
- Re-synchronizing after hot reload

#### `toggle_fullscreen()`
Toggle window fullscreen mode. Called from frontend F11 handler.

### Query Commands (No Events)

#### `get_cursor_position()`
Get current cursor position without emitting events.
- Returns: `{ domain_id, element_id, element_type }` or null

#### `get_all_domains()`
Get list of all registered domain IDs (for debugging).

#### `debug_domain(domain_id)`
Get detailed domain information (buttons, gates, layout mode, current index).

## Complete Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                      USER PRESSES WASD KEY                          │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 │ OS-level key event
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│            RUST: tauri-plugin-global-shortcut                       │
│                                                                     │
│  .with_handler(move |app, shortcut, event| {                       │
│    if event.state == ShortcutState::Pressed {                      │
│      if shortcut == &shortcut_w {                                  │
│        process_wasd_navigation(app, &navigator, WASDKey::W);       │
│      }                                                              │
│      // ... etc for A, S, D, Enter, Space                          │
│    }                                                                │
│  })                                                                 │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 │ Direct call (no IPC!)
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│           RUST: process_wasd_navigation() in lib.rs                 │
│                                                                     │
│  fn process_wasd_navigation(app, navigator, key) {                 │
│    let mut nav = navigator.lock()?;                                │
│    let result = nav.handle_wasd_input(key);  ────────────────┐     │
│                                                               │     │
│    match result {                                             │     │
│      CursorMoved { ... } => {                                 │     │
│        app.emit("cursor-moved", payload);  ◄─── EMIT EVENT   │     │
│      }                                                        │     │
│      ...                                                      │     │
│    }                                                          │     │
│  }                                                            │     │
└───────────────────────────────────────────────────────────────┼─────┘
                                                                │
                    ┌───────────────────────────────────────────┘
                    │ DomainNavigator processes navigation
                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│              RUST: inputHandler/domain_navigator.rs                 │
│                                                                     │
│  pub fn handle_wasd_input(&mut self, key: WASDKey) {               │
│    - Get active domain                                             │
│    - Calculate next index based on layout mode                     │
│    - Update cursor position                                        │
│    - Return NavigationResult                                       │
│  }                                                                  │
│                                                                     │
│  State Updated: cursor_position = { domain, element, type }        │
└─────────────────────────────────────────────────────────────────────┘
                    │
                    │ Events emitted via app.emit()
                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    TAURI EVENT SYSTEM                               │
│                 (IPC: Rust → TypeScript)                            │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 │ Tauri event: 'cursor-moved'
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│              FRONTEND: Interface.tsx Event Listeners                │
│                                                                     │
│  onMount(async () => {                                             │
│    await listen('cursor-moved', (event) => {                       │
│      window.dispatchEvent(                                         │
│        new CustomEvent('cursor-moved', {                           │
│          detail: event.payload                                      │
│        })                                                           │
│      );                                                             │
│    });                                                              │
│  });                                                                │
│                                                                     │
│  // NO keyboard capture here (except F11)!                         │
│  // Rust handles all WASD/Enter/Space                              │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 │ DOM CustomEvent
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  CHILD COMPONENTS (Button_IC, etc.)                 │
│                                                                     │
│  onMount(() => {                                                   │
│    window.addEventListener('cursor-moved', (e) => {                │
│      if (e.detail.element_id === buttonId) {                       │
│        setIsFocused(true);  // Update UI                           │
│      }                                                              │
│    });                                                              │
│  });                                                                │
└─────────────────────────────────────────────────────────────────────┘
```

## Implementation Guide

### Step 1: Setup Domain (Component Mount)

```typescript
// In your domain component (e.g., OSbar_IC.tsx)
import { invoke } from "@tauri-apps/api/core";

onMount(async () => {
  // Register the domain with Rust
  await invoke('register_domain', {
    domainId: 'main-menu',
    parentDomain: null,
    layoutMode: 'list-vertical',  // or 'grid', 'list-horizontal', 'spatial'
    gridColumns: null  // only needed for grid layout
  });
});

onCleanup(async () => {
  // Clean up when component unmounts
  await invoke('unregister_domain', { domainId: 'main-menu' });
});
```

### Step 2: Register Elements

```typescript
// Register buttons in your component
onMount(async () => {
  // Add buttons
  await invoke('register_button', {
    domainId: 'main-menu',
    buttonId: 'btn-start',
    bounds: { x: 100, y: 100, width: 200, height: 50 },  // Optional, for spatial
    order: 0  // Required, for list/grid positioning
  });

  await invoke('register_button', {
    domainId: 'main-menu',
    buttonId: 'btn-options',
    bounds: { x: 100, y: 160, width: 200, height: 50 },
    order: 1
  });

  // Add gate to connect to another domain
  await invoke('register_gate', {
    gateId: 'gate-to-game',
    sourceDomain: 'main-menu',
    targetDomain: 'game-screen',
    direction: 'bottom',  // 'top', 'bottom', 'left', 'right'
    entryPoint: 0  // Optional: which element to focus in target domain
  });
});

onCleanup(async () => {
  await invoke('unregister_button', { 
    domainId: 'main-menu', 
    buttonId: 'btn-start' 
  });
  // ... unregister other elements
});
```

### Step 3: Listen for Navigation Events (Child Components)

```typescript
// In your button component (e.g., Button_IC.tsx)
import { onMount, onCleanup, createSignal } from "solid-js";

export default function Button_IC(props: { id: string, label: string }) {
  const [isFocused, setIsFocused] = createSignal(false);
  
  onMount(() => {
    // Listen for cursor movement (from Rust via Interface.tsx relay)
    const handleCursorMoved = (e: CustomEvent) => {
      if (e.detail.element_id === props.id) {
        setIsFocused(true);
      } else {
        setIsFocused(false);
      }
    };
    
    // Listen for button activation (Enter/Space pressed on this button)
    const handleActivate = (e: CustomEvent) => {
      if (e.detail.element_id === props.id) {
        console.log(`Button ${props.id} activated!`);
        // Handle button action here
      }
    };
    
    window.addEventListener('cursor-moved', handleCursorMoved);
    window.addEventListener('button-activate', handleActivate);
    
    onCleanup(() => {
      window.removeEventListener('cursor-moved', handleCursorMoved);
      window.removeEventListener('button-activate', handleActivate);
    });
  });
  
  return (
    <button class={isFocused() ? 'focused' : ''}>
      {props.label}
    </button>
  );
}
```

### Step 4: Setup Top-Level Event Relay (Interface.tsx)

**This is already implemented** - you don't need to modify Interface.tsx. It:
1. Listens for Tauri events from Rust
2. Re-broadcasts as DOM CustomEvents
3. Handles F11 for fullscreen (only webview key capture)

```typescript
// Interface.tsx - Already implemented
import { listen } from "@tauri-apps/api/event";

// Rust events → DOM events relay
onMount(async () => {
  await listen('cursor-moved', (event) => {
    window.dispatchEvent(new CustomEvent('cursor-moved', {
      detail: event.payload
    }));
  });
  // ... similar for other events
});

// F11 is the ONLY key captured in frontend
const handleF11 = async (e: KeyboardEvent) => {
  if (e.key === 'F11') {
    e.preventDefault();
    await invoke('toggle_fullscreen');
  }
};
window.addEventListener('keydown', handleF11);
```

### Step 5: Initialize Cursor Position

```typescript
// In Interface.tsx or root component
onMount(async () => {
  // Wait for components to register their domains/buttons
  await new Promise(resolve => setTimeout(resolve, 200));
  
  // Ask Rust to emit current cursor position
  const emitted = await invoke('emit_cursor_position');
  
  if (!emitted) {
    // No cursor position - setup default
    const domains = await invoke('get_all_domains');
    if (domains.length > 0) {
      await invoke('set_active_domain', { domainId: domains[0] });
      await invoke('emit_cursor_position');
    }
  }
});
```

## Event Flow Examples

### Example 1: Simple WASD Navigation

```
User presses 'S' key
       ↓
OS delivers key event to Rust global shortcut handler
       ↓
Rust: with_handler() callback triggered with shortcut_s
       ↓
Rust: process_wasd_navigation(app, navigator, WASDKey::S)
       ↓
DomainNavigator.handle_wasd_input(WASDKey::S)
  - Finds current index in active domain
  - Calculates next index (current + 1 for vertical list)
  - Updates cursor_position
       ↓
Returns NavigationResult::CursorMoved { 
  domain_id: "menu", 
  element_id: "btn-2",
  element_type: Button 
}
       ↓
Rust: app.emit("cursor-moved", payload)
       ↓
Interface.tsx: listen('cursor-moved') callback
       ↓
window.dispatchEvent(new CustomEvent('cursor-moved', {...}))
       ↓
Button_IC components receive event, update focus state
       ↓
UI re-renders with new button highlighted
```

### Example 2: Button Activation

```
User presses Enter/Space on a button
       ↓
Rust: global shortcut handler detects Enter/Space
       ↓
Rust: process_activate(app, navigator)
       ↓
Checks cursor.element_type
  - If Gate: calls switch_domain(), emits domain-switched
  - If Button: emits button-activate
       ↓
Rust: app.emit("button-activate", { element_id: "btn-start", ... })
       ↓
Interface.tsx: listen('button-activate') → window.dispatchEvent(...)
       ↓
Button_IC: handleActivate() triggered
       ↓
Button performs its action
```

### Example 3: Domain Switching via Gate

```
User presses 'S' → navigates to gate → presses Enter
       ↓
Rust: cursor now on gate element
       ↓
Rust: app.emit("at-gate", { target_domain: "submenu" })
       ↓
UI shows gate indicator
       ↓
User presses Enter
       ↓
Rust: process_activate() detects cursor.element_type == Gate
       ↓
Rust: DomainNavigator.switch_domain()
  - Updates active_domain_id to target
  - Sets cursor to first element in new domain
       ↓
Rust emits:
  1. app.emit("domain-switched", { from: "menu", to: "submenu", ... })
  2. app.emit("cursor-moved", { domain_id: "submenu", element_id: "btn-1", ... })
       ↓
UI transitions to new domain with first button focused
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

## Debugging

### Check Navigation State
Press **'X'** key (debug only) to dump navigation state to console:
```javascript
=== Navigation State ===
Domains: ["main-menu", "game-screen"]
Cursor: { domain_id: "main-menu", element_id: "btn-1", element_type: "Button" }
Domain 'main-menu': {
  buttons: [
    { id: "btn-start", order: 0, enabled: true },
    { id: "btn-options", order: 1, enabled: true }
  ],
  currentIndex: 0,
  layoutMode: { List: { direction: "Vertical" } }
}
```

### Manual Cursor Emission
```typescript
await invoke('emit_cursor_position');  // Forces cursor-moved event
```

### Query Commands
```typescript
const cursor = await invoke('get_cursor_position');
const domains = await invoke('get_all_domains');
const domainInfo = await invoke('debug_domain', { domainId: 'main-menu' });
```

## Integration Checklist

### For New Domain Components:
- [ ] Call `register_domain()` in `onMount()`
- [ ] Call `unregister_domain()` in `onCleanup()`
- [ ] Choose appropriate layout mode (grid/list/spatial)
- [ ] Register all buttons with correct `order` values
- [ ] Register gates if connecting to other domains
- [ ] Unregister all elements in cleanup

### For Button/Interactive Components:
- [ ] Listen to `window` events: `'cursor-moved'`, `'button-activate'`
- [ ] Update visual state based on `element_id` match
- [ ] Handle activation via `button-activate` event
- [ ] Clean up event listeners in `onCleanup()`

### For Top-Level (Interface.tsx) - Already Done:
- [x] Tauri event listeners relay to DOM events
- [x] F11 keyboard capture for fullscreen
- [x] `emit_cursor_position()` on mount

## Common Patterns

### Pattern 1: Modal Domain
```typescript
// Modal opens
await invoke('register_domain', { 
  domainId: 'modal',
  parentDomain: 'main-menu',
  layoutMode: 'list-vertical'
});
await invoke('set_active_domain', { domainId: 'modal' });
// Register modal buttons...
await invoke('emit_cursor_position');

// Modal closes
await invoke('set_active_domain', { domainId: 'main-menu' });
await invoke('unregister_domain', { domainId: 'modal' });
await invoke('emit_cursor_position');
```

### Pattern 2: Dynamic Button List
```typescript
// Buttons change (e.g., filtered list)
for (const btn of oldButtons) {
  await invoke('unregister_button', { domainId: 'list', buttonId: btn.id });
}
for (const [idx, btn] of newButtons.entries()) {
  await invoke('register_button', { 
    domainId: 'list', 
    buttonId: btn.id, 
    order: idx 
  });
}
await invoke('emit_cursor_position');  // Refresh cursor
```

## Future Enhancements

- [ ] Domain stacking (modals on top of other domains)
- [ ] Automatic bounds calculation from frontend
- [ ] Focus history/breadcrumbs
- [ ] Gamepad support (also via Rust)
- [ ] Mouse hover integration
- [ ] Custom key bindings
- [ ] Animated cursor transitions
- [ ] Wrapping navigation (connect edges)
