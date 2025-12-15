# Controller Architecture: Centralized Input Hub

## Overview

The `Controller.tsx` component is the **single point of input handling** for the entire HyphaeicOS frontend. It serves as the bridge between the Rust backend (via Tauri events) and the DOM-based UI components, centralizing all event translation and keyboard input management.

**Core Principle**: Only `Controller.tsx` is allowed to import `@tauri-apps/api/event`. All other components receive events through standard DOM `CustomEvent`s dispatched by the Controller.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Rust Backend                             │
│  (Global Shortcuts: WASD, Enter, Space via OS-level capture)   │
└───────────────────────┬─────────────────────────────────────────┘
                        │
                        │ Tauri Events (IPC)
                        │ - cursor-moved
                        │ - button-activate
                        │ - at-gate
                        │ - domain-switched
                        │ - boundary-reached
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Controller.tsx (SINGLETON)                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Tauri Event Listeners (ONE set, persists through HMR)   │  │
│  │  - button-activate → document.getElementById().click()   │  │
│  │  - cursor-moved → sys-cursor-move CustomEvent           │  │
│  │  - Other events → DOM CustomEvent dispatches            │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ DOM Event Handlers (Replaced on each HMR cycle)          │  │
│  │  - window focus/blur → set_global_shortcuts_enabled      │  │
│  │  - keydown (F11, F12, debug keys)                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└───────────────────────┬─────────────────────────────────────────┘
                        │
                        │ DOM CustomEvents
                        │ - sys-cursor-move
                        │ - button-activate (fallback only)
                        │ - at-gate
                        │ - domain-switched
                        │ - boundary-reached
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                    UI Components (Reactive)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │  Button_IC   │  │  Domain.tsx  │  │   Others     │        │
│  │  - Listens:  │  │  - Listens:  │  │  - Listen to │        │
│  │    sys-cursor│  │    (domain-  │  │    relevant  │        │
│  │    -move     │  │    switched) │  │    events    │        │
│  │  - Receives: │  │              │  │              │        │
│  │    .click()  │  │              │  │              │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Responsibilities

### Controller.tsx

**Purpose**: Centralized input hub and event translation layer.

**Responsibilities**:

1. **Tauri Event Translation** (Rust → DOM)
   - Listens to all Tauri events from the Rust backend
   - Translates them to DOM `CustomEvent`s for UI consumption
   - Handles `button-activate` by calling `document.getElementById().click()` directly

2. **Keyboard Input Handling** (Browser → Logic)
   - Captures F11 (fullscreen toggle via Tauri invoke)
   - Captures F12 (DevTools - passes through)
   - Debug keys (X for state dump) in development mode
   - **Note**: WASD/Enter/Space are handled by Rust at OS level, NOT here

3. **Global Shortcuts Management**
   - Enables/disables Rust global shortcuts on window focus/blur
   - Ensures shortcuts don't interfere with other apps when window is unfocused

4. **Initial Cursor Position Setup**
   - Waits for domains/buttons to register
   - Sets initial active domain and emits cursor position

**HMR-Safe Design**:
- Uses window-level singleton state (`__HYPHA_CONTROLLER_STATE__`)
- Tauri listeners are registered **ONCE** and persist through HMR cycles
- DOM listeners are replaced on each mount to pick up code changes

---

### Button_IC.tsx (Marker Component)

**Purpose**: Lightweight button that reacts to Controller events.

**Responsibilities**:

1. **Registration with Rust**
   - Registers itself with the Rust `DomainNavigator` via `invoke('register_button')`
   - Provides bounds and order for spatial/list/grid navigation

2. **Focus Reactivity**
   - Listens for `sys-cursor-move` DOM event from Controller
   - Updates `isFocused` signal when `element_id` matches its `id`

3. **Click Handling**
   - Receives clicks from:
     - Mouse clicks (native DOM `onClick`)
     - Keyboard activation (Controller calls `.click()` on the element)
   - Provides visual feedback (active state flash)
   - Calls user-provided `onClick` handler

4. **Bounds Updates**
   - Re-registers with Rust on window resize for spatial navigation

**Key Properties**:
- Must have unique `id` prop (used for `document.getElementById`)
- Uses `tabIndex={-1}` to prevent native keyboard focus
- Uses `onMouseDown={(e) => e.preventDefault()}` to prevent focus on click
- No direct Tauri imports - purely reactive to DOM events

---

### Interface.tsx (Layout Component)

**Purpose**: Pure visual composition layer.

**Responsibilities**:
- Composes visual components (`OSbar_IC`, `Compositor_IC`, etc.)
- Wraps everything in `<Controller>` to ensure input handling is active
- **NO logic, NO listeners, NO hooks** (except pure UI state)

---

## Event Flow

### Example 1: Keyboard Activation (Enter/Space on Button)

```
User presses Enter key
    ↓
OS captures key → Rust global shortcut handler
    ↓
Rust: process_activate() in lib.rs
    ↓
Rust checks cursor.element_type
    ↓ (if Button)
Rust: app.emit("button-activate", { element_id, domain_id, ... })
    ↓ (Tauri IPC)
Controller.tsx: listen('button-activate') callback
    ↓
Controller: const target = document.getElementById(element_id)
    ↓
Controller: target.click()  // Native DOM click event
    ↓
Button_IC.tsx: onClick handler fires
    ↓
Button_IC: handleClick() runs
    ↓
Button_IC: setIsActive(true) → visual feedback
    ↓
Button_IC: props.onClick() → user callback (console.log, etc.)
```

### Example 2: WASD Navigation (Focus Change)

```
User presses 'S' key
    ↓
OS captures key → Rust global shortcut handler
    ↓
Rust: process_wasd_navigation() in lib.rs
    ↓
Rust: DomainNavigator.handle_wasd_input(WASDKey::S)
    ↓
Rust updates cursor_position state
    ↓
Rust: app.emit("cursor-moved", { element_id, domain_id, ... })
    ↓ (Tauri IPC)
Controller.tsx: listen('cursor-moved') callback
    ↓
Controller: window.dispatchEvent(new CustomEvent('sys-cursor-move', { detail: ... }))
    ↓
Button_IC.tsx: window.addEventListener('sys-cursor-move') fires
    ↓
Button_IC: Checks if e.detail.element_id === props.id
    ↓ (if match)
Button_IC: setIsFocused(true) → CSS class updates → visual highlight
```

### Example 3: Mouse Click

```
User clicks button with mouse
    ↓
Browser: Native click event fires
    ↓
Button_IC.tsx: onClick handler fires (native DOM event)
    ↓
Button_IC: handleClick() runs
    ↓
Button_IC: setIsActive(true) → visual feedback
    ↓
Button_IC: props.onClick() → user callback
```

**Note**: Mouse clicks bypass Rust entirely. Only keyboard navigation goes through Rust → Controller → Button.

---

## HMR-Safe Singleton Pattern

### Problem

Hot Module Reload (HMR) in development can cause components to remount rapidly. If Tauri listeners are registered on every mount without cleanup, they accumulate, causing:
- Multiple event handlers firing for a single action
- Memory leaks
- Performance degradation

### Solution

The Controller uses a **window-level singleton state** that survives HMR cycles:

```typescript
interface ControllerState {
  tauriListenersActive: boolean;  // Flag: listeners already registered?
  tauriUnlisteners: UnlistenFn[]; // Array of cleanup functions
  domCleanup: (() => void) | null; // DOM listener cleanup
}

const getControllerState = (): ControllerState => {
  const key = '__HYPHA_CONTROLLER_STATE__';
  if (!(window as any)[key]) {
    (window as any)[key] = { ... };
  }
  return (window as any)[key];
};
```

### Behavior

**First Mount (or after page refresh)**:
1. `tauriListenersActive === false`
2. Registers all Tauri listeners
3. Sets `tauriListenersActive = true`
4. Stores cleanup functions in `tauriUnlisteners`

**Subsequent Mounts (HMR cycles)**:
1. `tauriListenersActive === true`
2. **Skips Tauri listener registration**
3. Only replaces DOM listeners (for code changes)

**On Component Unmount**:
1. Cleans up DOM listeners (removed from window)
2. **Does NOT clean up Tauri listeners** (they persist until page refresh)

**On Page Refresh**:
- Window object is cleared
- Singleton state resets
- Fresh listeners are registered

### Trade-offs

- ✅ Prevents listener accumulation during HMR
- ✅ Allows code changes to DOM listeners to take effect
- ⚠️ Tauri listeners persist until full page refresh (acceptable for development)
- ⚠️ Requires full refresh to clear Tauri listeners after stopping dev server

---

## Integration Points

### With Rust Backend

**Tauri Events Received** (via `listen()`):
- `cursor-moved`: Cursor position changed
- `button-activate`: Button should be activated (Enter/Space pressed)
- `at-gate`: Cursor reached a domain gate
- `domain-switched`: Successfully switched domains
- `boundary-reached`: Cursor hit domain boundary

**Tauri Commands Invoked** (via `invoke()`):
- `set_global_shortcuts_enabled({ enabled: boolean })`: Enable/disable WASD shortcuts
- `toggle_fullscreen()`: Toggle window fullscreen
- `get_active_domain()`: Get current active domain ID
- `set_active_domain({ domainId })`: Set active domain
- `emit_cursor_position()`: Force emit cursor position event
- `get_all_domains()`: Get all registered domains (debug)
- `get_cursor_position()`: Get current cursor position (debug)
- `debug_domain({ domainId })`: Get domain debug info

### With UI Components

**DOM Events Dispatched** (to `window`):
- `sys-cursor-move`: Cursor moved (new standard event name)
- `button-activate`: Button activation (fallback only, if element not found)
- `at-gate`: Gate reached
- `domain-switched`: Domain switched
- `boundary-reached`: Boundary hit

**Direct DOM Manipulation**:
- `document.getElementById(element_id).click()`: Triggers button click programmatically

### With Button_IC

**Button_IC Requirements**:
- Must have `id={props.id}` on the root `<button>` element
- Must listen for `sys-cursor-move` event
- Must handle native `onClick` for mouse clicks
- Controller will call `.click()` for keyboard activation

**Button_IC Interaction**:
1. Button registers with Rust (provides bounds, order)
2. Controller receives `button-activate` from Rust
3. Controller finds button via `document.getElementById(id)`
4. Controller calls `target.click()`
5. Button's `onClick` fires → user callback executes

---

## State Management

### Window-Level State (Singleton)

```typescript
window.__HYPHA_CONTROLLER_STATE__ = {
  tauriListenersActive: boolean,  // Prevents duplicate Tauri registration
  tauriUnlisteners: UnlistenFn[], // Cleanup functions for Tauri listeners
  domCleanup: (() => void) | null // Cleanup function for DOM listeners
}
```

**Lifetime**: Survives HMR, cleared on page refresh.

### Component-Level State (Button_IC)

```typescript
const [isFocused, setIsFocused] = createSignal(false);  // Reactive focus state
const [isActive, setIsActive] = createSignal(false);    // Active flash state
const [isRegistered, setIsRegistered] = createSignal(false); // Registration status
```

**Lifetime**: Component lifecycle (cleared on unmount).

---

## Error Handling

### Missing Element ID

If `document.getElementById(element_id)` returns `null`:
- Controller logs warning: `[Controller] Activation received for missing ID: ${element_id}`
- Controller dispatches fallback `button-activate` DOM event (for non-DOM components)

### Tauri Listener Registration Failure

If `listen()` throws:
- Error logged: `[Controller] Failed to setup Tauri listeners`
- `tauriListenersActive` flag reset to `false` (allows retry on next mount)
- DOM listeners still function (partial functionality)

### Global Shortcuts Failure

If `set_global_shortcuts_enabled` fails:
- Error logged to console
- No user-facing error (graceful degradation)
- Navigation may not work until window regains focus

---

## Development vs Production

### Development Mode

- Debug key 'X' dumps navigation state to console
- More verbose error logging
- HMR singleton pattern active

### Production Mode

- Debug keys disabled
- Minimal logging
- Same singleton pattern (no HMR, but still safe)

---

## Testing Considerations

### Unit Testing Controller

- Mock `@tauri-apps/api/event.listen()` to return fake unlisten functions
- Mock `invoke()` to return test data
- Mock `document.getElementById()` to return test elements
- Verify DOM events are dispatched correctly
- Test singleton state behavior

### Integration Testing

- Test with real Rust backend
- Verify event flow: Rust → Controller → DOM → Component
- Test HMR behavior (mount/unmount cycles)
- Test focus/blur shortcut enabling/disabling

### Button_IC Testing

- Test focus state updates on `sys-cursor-move` event
- Test click handling (mouse and programmatic)
- Test Rust registration/unregistration
- Test bounds updates on resize

---

## Migration Notes

### From Old Architecture

**Before**:
- `Interface.tsx` handled Tauri events
- Each `Button_IC` listened to multiple events directly
- Global shortcuts managed in `Interface.tsx`

**After**:
- `Controller.tsx` is the single Tauri event handler
- `Button_IC` only listens to `sys-cursor-move` (focus state)
- Activation happens via `.click()` call, not event listener
- `Interface.tsx` is pure layout

### Breaking Changes

- `Button_IC` no longer listens to `button-activate` DOM event (Controller handles it)
- `Button_IC` must have `id={props.id}` on root element (for `getElementById`)
- Components should listen to `sys-cursor-move` instead of `cursor-moved` (though legacy event still dispatched)

---

## Future Enhancements

### Potential Improvements

1. **Event Debouncing**: Debounce rapid `cursor-moved` events to reduce re-renders
2. **Event Priority Queue**: Queue events during navigation to prevent race conditions
3. **Listener Health Check**: Periodically verify Tauri listeners are still active
4. **Analytics**: Track event frequency/patterns for optimization
5. **Custom Key Bindings**: Allow user-configurable key bindings (still via Rust)

### Architectural Considerations

- Current design assumes one Controller instance (enforced by singleton)
- If multiple windows are needed, singleton pattern may need adjustment
- Tauri listeners are per-process, so multiple windows would share listeners (may need routing)

---

## File Structure

```
src/HMI/A_Controller/
├── Controller.tsx         # Main component (input hub)
└── ARCHITECTURE.md        # This document
```

**Dependencies**:
- `@tauri-apps/api/event` (listen)
- `@tauri-apps/api/core` (invoke)
- `solid-js` (onMount, onCleanup, JSX)

**Used By**:
- `Interface.tsx` (wraps children in `<Controller>`)

**Integrates With**:
- `Button_IC.tsx` (receives events via DOM)
- Rust backend (receives Tauri events)
- All domain/component systems (via DOM events)

---

## Summary

The Controller architecture provides a **clean separation of concerns**:
- **Rust**: Handles OS-level input capture and navigation logic
- **Controller**: Translates Tauri events to DOM events, manages keyboard fallbacks
- **UI Components**: Reactively respond to DOM events, no direct Tauri coupling

This design ensures:
- ✅ Single source of truth for input handling
- ✅ HMR-safe development experience
- ✅ Decoupled, testable components
- ✅ Clear event flow and responsibilities

