# HyphaeicOS

A desktop operating system shell built with **Tauri 2** (Rust backend) and **SolidJS** (TypeScript frontend). The system features a domain-based WASD keyboard navigation system, windowed application management, and a modular HMI (Human-Machine Interface) architecture.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)
![SolidJS](https://img.shields.io/badge/SolidJS-1.9-blue?logo=solid)
![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue?logo=typescript)

---

## 🏗️ Architecture Overview

HyphaeicOS follows a **Rust-first architecture** where the backend owns all application state and input handling, while the frontend serves as a reactive UI layer.

```
┌─────────────────────────────────────────────────────────────────────┐
│                            Rust Backend                             │
│  ┌───────────────┐  ┌─────────────────┐  ┌───────────────────────┐  │
│  │ Asset Loader  │  │ Domain Navigator│  │   State Manager       │  │
│  │ (remote cache)│  │ (WASD navigation)│  │   (window tracking)  │  │
│  └───────────────┘  └─────────────────┘  └───────────────────────┘  │
│                              ↕ Tauri Events (IPC)                   │
├─────────────────────────────────────────────────────────────────────┤
│                         TypeScript Frontend                         │
│  ┌───────────────┐  ┌─────────────────┐  ┌───────────────────────┐  │
│  │   Controller  │  │     Domain      │  │      Interface        │  │
│  │ (event relay) │  │ (navigation UI) │  │   (visual layout)     │  │
│  └───────────────┘  └─────────────────┘  └───────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🔧 Backend (Rust/Tauri)

The Rust backend (`src-tauri/`) handles all core functionality including input capture, navigation logic, window management, and asset caching.

### Technology Stack

- **Tauri 2.0**: Cross-platform desktop framework
- **tauri-plugin-global-shortcut**: OS-level keyboard capture
- **tauri-plugin-opener**: System opener integration
- **reqwest**: HTTP client for asset downloading
- **tokio**: Async runtime for file I/O
- **serde**: JSON serialization/deserialization
- **uuid**: Unique window identifiers
- **portable-pty**: PTY support for terminal emulation

### Backend Modules

```
src-tauri/src/
├── lib.rs              # Main entry point, Tauri commands, event emission
├── main.rs             # Application bootstrap
├── assetLoader/        # Remote asset downloading and caching
│   ├── mod.rs
│   └── asset_loader.rs # URL-based asset management
├── inputHandler/       # WASD navigation system
│   ├── mod.rs
│   ├── types.rs        # Data structures (Domain, Button, Gate, etc.)
│   ├── domain_navigator.rs  # Navigation logic and cursor management
│   └── spatial.rs      # Spatial navigation algorithms
└── state/              # Application state management
    ├── mod.rs          # StateManager (window tracking, slots)
    └── window.rs       # Window types (WindowInstance, WindowState)
```

### Core Backend Systems

#### 1. Domain Navigation (`inputHandler/`)

The navigation system provides WASD-based cursor movement across UI "domains" (containers of interactive elements).

**Key Concepts:**
- **Domain**: A UI container with navigable elements (buttons, gates)
- **Button**: An interactive element that can receive focus
- **Gate**: A boundary element for switching between domains
- **Cursor**: The currently focused element (managed by Rust)

**Layout Modes:**
- `list-vertical`: Elements in a vertical list (W/S navigation)
- `list-horizontal`: Elements in a horizontal list (A/D navigation)
- `grid`: 2D grid layout with configurable columns
- `spatial`: Free-form positioning using screen coordinates

**How it works:**
1. Global shortcuts (WASD, Enter, Space) are captured at OS level
2. Rust processes navigation based on current domain's layout mode
3. Cursor position is updated in Rust state
4. Events are emitted to frontend via Tauri IPC

```rust
// Navigation result types
enum NavigationResult {
    CursorMoved { domain_id, element_id, element_type },
    AtGate { gate_id, target_domain },
    DomainSwitched { from_domain, to_domain, new_element_id },
    BoundaryReached,
    NoActiveDomain,
    Error { message },
}
```

#### 2. Window Management (`state/`)

The window manager handles a dual-slot compositor system where up to two windows can be displayed simultaneously.

**Window States:**
- `Minimized`: Half-size, occupies one slot
- `Maximized`: Full-size, spans entire compositor
- `Hidden`: Not rendered
- `Closing`: Playing close animation

**Compositor Slots:**
```rust
enum CompositorSlot {
    Left,   // First available slot
    Right,  // Second slot
}
```

**Window Lifecycle:**
1. `spawn_window()`: Creates window in first available slot
2. `set_window_state()`: Transitions between states
3. `close_window()`: Triggers closing animation
4. `remove_window()`: Removes from state after animation

#### 3. Asset Loader (`assetLoader/`)

Downloads and caches remote assets (images, videos, audio, documents) to the local app data directory.

**Features:**
- URL-to-filename hashing for cache keys
- Type-based subdirectories (`images/`, `videos/`, etc.)
- Cache status checking
- Cache clearing (per-type or all)

**Tauri Commands:**
```rust
load_asset(url, asset_type) -> AssetInfo
is_asset_cached(url, asset_type) -> bool
get_asset_cache_path(url, asset_type) -> String
clear_asset_cache(asset_type?) -> String
```

### Tauri Commands (API)

#### Navigation Commands
| Command | Parameters | Description |
|---------|------------|-------------|
| `register_domain` | `domainId`, `parentDomain?`, `layoutMode`, `gridColumns?` | Register a navigation domain |
| `unregister_domain` | `domainId` | Remove a domain |
| `register_button` | `domainId`, `buttonId`, `bounds?`, `order` | Add a button to a domain |
| `unregister_button` | `domainId`, `buttonId` | Remove a button |
| `register_gate` | `gateId`, `sourceDomain`, `targetDomain`, `direction`, `entryPoint?` | Add a domain gate |
| `set_active_domain` | `domainId` | Set which domain receives input |
| `handle_wasd_input` | `key` | Process navigation (usually via global shortcuts) |
| `switch_domain` | - | Activate current gate |
| `emit_cursor_position` | - | Force emit cursor-moved event |

#### Window Commands
| Command | Parameters | Description |
|---------|------------|-------------|
| `spawn_window` | `contentKey`, `sourceElementId?`, `sourceDomainId?` | Create a new window |
| `close_window` | `id` | Begin window close animation |
| `remove_window` | `id` | Remove window from state |
| `set_window_state` | `id`, `windowState` | Change window state |

#### Utility Commands
| Command | Parameters | Description |
|---------|------------|-------------|
| `toggle_fullscreen` | - | Toggle window fullscreen mode |
| `set_global_shortcuts_enabled` | `enabled` | Enable/disable WASD shortcuts |
| `greet` | `name` | Test command |

### Tauri Events (Backend → Frontend)

Events emitted by Rust to notify the frontend of state changes:

| Event | Payload | Description |
|-------|---------|-------------|
| `cursor-moved` | `{ domain_id, element_id, element_type }` | Cursor position changed |
| `button-activate` | `{ domain_id, element_id, element_type }` | Button activation (Enter/Space) |
| `at-gate` | `{ gate_id, target_domain }` | Cursor reached a gate |
| `domain-switched` | `{ from_domain, to_domain, new_element_id }` | Domain switch completed |
| `boundary-reached` | `{ direction }` | Cursor hit domain edge |
| `window-created` | `WindowInstance` | New window spawned |
| `window-closed` | `string (id)` | Window removed |
| `window-state-changed` | `WindowInstance` | Window state updated |
| `return-focus` | `{ domain_id, element_id }` | Return focus after window close |

---

## 🎨 Frontend (SolidJS/TypeScript)

The frontend (`src/`) is a reactive UI layer that listens to Tauri events and renders the interface.

### Technology Stack

- **SolidJS 1.9**: Reactive UI framework
- **Vite 6**: Build tool and dev server
- **TypeScript 5.6**: Type safety
- **@tauri-apps/api**: Tauri event/invoke bindings

### Frontend Structure

```
src/
├── App.tsx             # Root component
├── App.css             # Global styles
├── index.tsx           # Entry point
├── vite-env.d.ts       # Vite type definitions
└── HMI/                # Human-Machine Interface components
    ├── store.ts        # Window store (SolidJS reactive state)
    ├── A_Controller/   # Input hub (event translation)
    ├── A_Domain/       # Domain wrapper component
    ├── A_Interface/    # Visual layout composition
    ├── Button/         # Interactive button component
    ├── Background/     # Background layer
    ├── OSbar/          # Navigation/status bar
    ├── WindowManager/  # Window and compositor components
    │   ├── Compositor/ # Dual-slot compositor
    │   └── Window/     # Window chrome and content
    └── TESTING_DUMMY/  # Test content component
```

### Core Frontend Systems

#### 1. Controller (`A_Controller/Controller.tsx`)

The Controller is the **single point of Tauri event handling**. It translates Rust events into DOM CustomEvents for UI components.

**Responsibilities:**
- Listen to all Tauri events from Rust
- Dispatch DOM CustomEvents to components
- Handle button activation via `document.getElementById().click()`
- Manage global shortcut enable/disable on focus/blur
- Handle F11 fullscreen toggle

**HMR-Safe Design:**
The Controller uses a window-level singleton pattern to survive Hot Module Reload:
```typescript
window.__HYPHA_CONTROLLER_STATE__ = {
  tauriListenersActive: boolean,
  tauriUnlisteners: UnlistenFn[],
  domCleanup: (() => void) | null
}
```

#### 2. Domain (`A_Domain/Domain.tsx`)

A wrapper component that registers a navigation domain with Rust.

**Usage:**
```tsx
<Domain id="main-menu" layoutMode="list-vertical">
  <Button id="btn-start" order={0} onClick={handleStart} />
  <Button id="btn-options" order={1} onClick={handleOptions} />
</Domain>
```

**Provides Context:**
- `useDomain()`: Get domain context (id, isReady)
- `useDomainId()`: Get just the domain ID

#### 3. Store (`store.ts`)

SolidJS reactive store for window state:
```typescript
export const [windowStore, setWindowStore] = createStore<WindowStoreState>({
  windows: [],
});

// Helpers
addWindow(window: WindowInstance)
removeWindow(id: string)
updateWindow(updatedWindow: WindowInstance)
getWindowInSlot(slot: CompositorSlot)
```

### Event Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                      USER PRESSES WASD KEY                           │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ OS-level key event
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  Rust: Global Shortcut Handler                       │
│                  (tauri-plugin-global-shortcut)                      │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ Direct Rust call
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  Rust: DomainNavigator.handle_wasd_input()           │
│                  → Updates cursor_position                           │
│                  → Returns NavigationResult                          │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ app.emit("cursor-moved", payload)
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  TypeScript: Controller.tsx                          │
│                  listen('cursor-moved') callback                     │
│                  → window.dispatchEvent(CustomEvent)                 │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ DOM CustomEvent
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  Component: Button_IC.tsx                            │
│                  → Checks if element_id matches                      │
│                  → setIsFocused(true/false)                          │
│                  → CSS class updates                                 │
└─────────────────────────────────────────────────────────────────────┘
```

### Integration with Backend

**Frontend → Backend (invoke):**
```typescript
import { invoke } from "@tauri-apps/api/core";

// Register domain on mount
await invoke('register_domain', {
  domainId: 'main-menu',
  layoutMode: 'list-vertical'
});

// Spawn window on button click
await invoke('spawn_window', {
  contentKey: 'TESTING_DUMMY',
  sourceElementId: 'btn-open',
  sourceDomainId: 'osbar-nav'
});
```

**Backend → Frontend (events):**
```typescript
import { listen } from "@tauri-apps/api/event";

// Listen for cursor movement
const unlisten = await listen('cursor-moved', (event) => {
  window.dispatchEvent(new CustomEvent('sys-cursor-move', {
    detail: event.payload
  }));
});
```

---

## 🚀 Getting Started

### Prerequisites

- **Node.js** 18+ and **pnpm**
- **Rust** 1.75+ with `cargo`
- **Tauri CLI**: `cargo install tauri-cli`

### Development

```bash
# Install dependencies
pnpm install

# Start development server (runs both Vite and Tauri)
pnpm tauri dev
```

### Production Build

```bash
# Build the application
pnpm tauri build
```

### Project Structure

```
HyphaeicOS/
├── src/                  # Frontend source (SolidJS/TypeScript)
├── src-tauri/            # Backend source (Rust/Tauri)
│   ├── src/              # Rust source files
│   ├── Cargo.toml        # Rust dependencies
│   └── tauri.conf.json   # Tauri configuration
├── public/               # Static assets
├── dist/                 # Build output
├── package.json          # Node.js dependencies
├── vite.config.ts        # Vite configuration
└── tsconfig.json         # TypeScript configuration
```

---

## 🧪 Testing

### Rust Tests

```bash
cd src-tauri
cargo test
```

Tests include:
- Domain registration and unregistration
- Button registration with auto-focus
- Navigation (list, grid, spatial)
- Boundary detection
- Gate switching

### Debug Tools

**In Development Mode:**
- Press **X** key to dump navigation state to console
- Press **F11** to toggle fullscreen
- Press **F12** to open DevTools

**Query Commands:**
```typescript
// Get all registered domains
const domains = await invoke('get_all_domains');

// Get current cursor position
const cursor = await invoke('get_cursor_position');

// Get detailed domain info
const info = await invoke('debug_domain', { domainId: 'main-menu' });
```

---

## 📁 Configuration

### Tauri Configuration (`tauri.conf.json`)

```json
{
  "productName": "HyphaeicOS",
  "version": "0.1.0",
  "identifier": "com.eonk.hyphaeicos",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{
      "title": "HyphaeicOS",
      "width": 800,
      "height": 600
    }],
    "security": {
      "assetProtocol": {
        "enable": true,
        "scope": { "allow": ["$APPDATA/**"] }
      }
    }
  }
}
```

### Vite Configuration (`vite.config.ts`)

- SolidJS plugin enabled
- Dev server on port 1420
- HMR configured for Tauri

---

## 📖 Documentation

Detailed documentation is available in the following files:

| Document | Location | Description |
|----------|----------|-------------|
| Input Handler README | `src-tauri/src/inputHandler/INPUT_HANDLER_README.md` | Complete navigation system docs |
| Controller Architecture | `src/HMI/A_Controller/ARCHITECTURE.md` | Frontend input handling design |
| Domain Navigation Example | `src-tauri/src/inputHandler/DOMAIN_NAVIGATION_EXAMPLE.md` | Usage examples |
| Implementation Summary | `src-tauri/src/inputHandler/IMPLEMENTATION_SUMMARY.md` | Technical details |

---

## 🏛️ Design Philosophy

1. **Rust-First**: Backend owns all state; frontend is reactive
2. **Event-Driven**: Unidirectional data flow via Tauri events
3. **Domain-Based Navigation**: Structured UI regions for keyboard control
4. **Dual-Slot Compositor**: Two-window side-by-side layout
5. **HMR-Safe**: Development experience survives hot reload

---
