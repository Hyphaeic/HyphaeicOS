# Domain Navigation System - Usage Examples

## Quick Start Example

Here's a complete example showing how to use the domain navigation system.

## Scenario: File Browser with App Window

Let's build a UI with two domains: a file browser and an app window. Users can navigate within each domain and switch between them.

### Step 1: Create Domain Components

```typescript
// FileExplorer.tsx
import { onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export function FileExplorer() {
  onMount(async () => {
    // Register this as a domain
    await invoke('register_domain', {
      domainId: 'file-explorer',
      parentDomain: null,
      layoutMode: 'list-vertical',
      gridColumns: null
    });
  });

  onCleanup(async () => {
    await invoke('unregister_domain', {
      domainId: 'file-explorer'
    });
  });

  return (
    <div class="file-explorer">
      <h2>File Explorer</h2>
      {/* Buttons will be added here */}
    </div>
  );
}
```

### Step 2: Add Navigable Buttons

```typescript
// NavigableButton.tsx
import { onMount, onCleanup, createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface ButtonProps {
  domainId: string;
  buttonId: string;
  order: number;
  onClick?: () => void;
  children?: any;
}

export function NavigableButton(props: ButtonProps) {
  const [isFocused, setIsFocused] = createSignal(false);
  let buttonRef: HTMLButtonElement | undefined;

  onMount(async () => {
    // Get button bounds
    const rect = buttonRef?.getBoundingClientRect();

    // Register with Rust
    await invoke('register_button', {
      domainId: props.domainId,
      buttonId: props.buttonId,
      bounds: rect ? {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height
      } : null,
      order: props.order
    });

    // Listen for cursor movements
    const unlisten = await listen('cursor-moved', (event: any) => {
      setIsFocused(
        event.payload.element_id === props.buttonId &&
        event.payload.domain_id === props.domainId
      );
    });

    onCleanup(async () => {
      unlisten();
      await invoke('unregister_button', {
        domainId: props.domainId,
        buttonId: props.buttonId
      });
    });
  });

  return (
    <button
      ref={buttonRef}
      class={`nav-button ${isFocused() ? 'focused' : ''}`}
      onClick={props.onClick}
    >
      {props.children}
    </button>
  );
}
```

### Step 3: Use Buttons in Domain

```typescript
// FileExplorer.tsx (updated)
export function FileExplorer() {
  onMount(async () => {
    await invoke('register_domain', {
      domainId: 'file-explorer',
      parentDomain: null,
      layoutMode: 'list-vertical',
      gridColumns: null
    });
  });

  return (
    <div class="file-explorer">
      <h2>File Explorer</h2>
      
      <NavigableButton
        domainId="file-explorer"
        buttonId="btn-file-1"
        order={0}
        onClick={() => console.log("Open file 1")}
      >
        📄 Document.txt
      </NavigableButton>

      <NavigableButton
        domainId="file-explorer"
        buttonId="btn-file-2"
        order={1}
        onClick={() => console.log("Open file 2")}
      >
        📄 Image.png
      </NavigableButton>

      <NavigableButton
        domainId="file-explorer"
        buttonId="btn-file-3"
        order={2}
        onClick={() => console.log("Open file 3")}
      >
        📁 Folder
      </NavigableButton>
    </div>
  );
}
```

### Step 4: Add Global WASD Handler

```typescript
// App.tsx
import { onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function App() {
  onMount(async () => {
    // Listen for keyboard events
    window.addEventListener('keydown', async (e) => {
      if (['w', 'a', 's', 'd'].includes(e.key.toLowerCase())) {
        e.preventDefault();
        
        const result = await invoke('handle_wasd_input', {
          key: e.key.toUpperCase()
        });

        console.log('Navigation result:', result);

        // Emit event for frontend components
        if (result.type === 'CursorMoved') {
          window.dispatchEvent(new CustomEvent('cursor-moved', {
            detail: result
          }));
        }
        else if (result.type === 'AtGate') {
          console.log('At gate! Press Enter to switch domains');
          // Show visual indicator
        }
      }
      else if (e.key === 'Enter' || e.key === ' ') {
        // Try to switch domain at gate
        const result = await invoke('switch_domain');
        if (result.type === 'DomainSwitched') {
          console.log(`Switched from ${result.from_domain} to ${result.to_domain}`);
          window.dispatchEvent(new CustomEvent('cursor-moved', {
            detail: { element_id: result.new_element_id }
          }));
        }
      }
    });
  });

  return (
    <div>
      <FileExplorer />
      {/* Other domains */}
    </div>
  );
}
```

### Step 5: Add CSS for Visual Cursor

```css
/* styles.css */
.nav-button {
  padding: 12px 20px;
  margin: 8px;
  border: 2px solid transparent;
  background: #2a2a2a;
  color: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.nav-button.focused {
  border-color: #00ff88;
  box-shadow: 
    0 0 15px rgba(0, 255, 136, 0.5),
    inset 0 0 20px rgba(0, 255, 136, 0.1);
  transform: scale(1.05);
  background: #1a3a2a;
}

.nav-button:hover {
  background: #3a3a3a;
}
```

## Testing the System

1. **Start the app**: The first domain registered becomes active
2. **Press S**: Cursor moves down to next button
3. **Press W**: Cursor moves up to previous button
4. **Press A/D**: No effect in vertical list (as designed)
5. **Reach gate**: Visual indicator shows domain switch is available
6. **Press Enter**: Switch to target domain

## Grid Layout Example

```typescript
// Menu with 3 columns
await invoke('register_domain', {
  domainId: 'main-menu',
  parentDomain: null,
  layoutMode: 'grid',
  gridColumns: 3
});

// Add 9 buttons (3x3 grid)
for (let i = 0; i < 9; i++) {
  await invoke('register_button', {
    domainId: 'main-menu',
    buttonId: `btn-${i}`,
    bounds: null, // Optional for grid
    order: i
  });
}

// Now WASD navigates in 2D:
// W/S = up/down between rows
// A/D = left/right between columns
```

## Spatial Layout Example

```typescript
// Free-form layout with actual positions
await invoke('register_domain', {
  domainId: 'canvas',
  parentDomain: null,
  layoutMode: 'spatial',
  gridColumns: null
});

// Buttons can be anywhere on screen
await invoke('register_button', {
  domainId: 'canvas',
  buttonId: 'btn-top-left',
  bounds: { x: 50, y: 50, width: 100, height: 40 },
  order: 0
});

await invoke('register_button', {
  domainId: 'canvas',
  buttonId: 'btn-center',
  bounds: { x: 300, y: 200, width: 100, height: 40 },
  order: 1
});

// Navigation finds nearest element in WASD direction
```

## Adding Gates for Domain Switching

```typescript
// In FileExplorer domain
await invoke('register_gate', {
  gateId: 'gate-explorer-to-editor',
  sourceDomain: 'file-explorer',
  targetDomain: 'text-editor',
  direction: 'right',
  entryPoint: 0 // Start at first button in editor
});

// When user navigates to this gate:
// 1. Navigation returns AtGate result
// 2. Frontend shows indicator
// 3. User presses Enter/Space
// 4. invoke('switch_domain') moves to text-editor domain
```

## Debugging

```typescript
// Get all registered domains
const domains = await invoke('get_all_domains');
console.log('Registered domains:', domains);

// Get current cursor position
const cursor = await invoke('get_cursor_position');
console.log('Cursor:', cursor);

// Get active domain
const active = await invoke('get_active_domain');
console.log('Active domain:', active);
```

## Best Practices

1. **Always unregister**: Use `onCleanup` to unregister domains/buttons when components unmount
2. **Update bounds**: If layout changes (resize), unregister and re-register with new bounds
3. **Order matters**: For grid/list layouts, order determines navigation sequence
4. **Test boundaries**: Try navigating at edges to ensure boundary detection works
5. **Visual feedback**: Always provide clear visual indication of cursor position
6. **Gate indicators**: Show when user is at a gate and can switch domains

## Common Patterns

### Modal Dialog (Temporary Domain)
```typescript
function Modal() {
  onMount(async () => {
    await invoke('register_domain', {
      domainId: 'modal',
      parentDomain: 'main',
      layoutMode: 'list-vertical'
    });
    
    // Make modal active (overrides parent)
    await invoke('set_active_domain', { domainId: 'modal' });
  });

  onCleanup(async () => {
    // Return to parent domain
    await invoke('set_active_domain', { domainId: 'main' });
    await invoke('unregister_domain', { domainId: 'modal' });
  });
}
```

### Dynamic Button List
```typescript
// As list changes, update buttons
createEffect(() => {
  files().forEach((file, index) => {
    invoke('register_button', {
      domainId: 'file-list',
      buttonId: `file-${file.id}`,
      order: index
    });
  });
});
```

## Next Steps

Now that the Rust core is complete, you can:
1. Create reusable SolidJS components (Domain, Button, Gate)
2. Add event listeners for cursor updates
3. Implement visual cursor styling
4. Add gamepad support alongside WASD
5. Create domain transition animations
