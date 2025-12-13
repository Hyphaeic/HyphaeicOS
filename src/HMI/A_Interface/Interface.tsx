import { onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import OSbar_IC from "../OSbar/OSbar_IC";
import Compositor_IC from "../WindowManager/Compositor/Compositor_IC";
import Window_IC from "../WindowManager/Window/Window_IC";
import TESTING_DUMMY from "../TESTING_DUMMY/TESTING_DUMMY";

// Module-level state for hot reload protection
// These persist across component re-mounts during hot reload
let tauriListenersSetup = false;
let tauriUnlisteners: UnlistenFn[] = [];
let f11HandlerRegistered = false;
let focusHandlersRegistered = false;

// Event payload types from Rust
interface CursorMovedPayload {
  domain_id: string;
  element_id: string;
  element_type: string;
}

interface AtGatePayload {
  gate_id: string;
  target_domain: string;
}

interface DomainSwitchedPayload {
  from_domain: string;
  to_domain: string;
  new_element_id: string;
}

interface BoundaryReachedPayload {
  direction: string;
}

interface ButtonActivatePayload {
  domain_id: string;
  element_id: string;
  element_type: string;
}

/**
 * Interface component - Core composer for OSbar and other visual elements
 * 
 * Input handling architecture (Rust-first):
 * 1. WASD/Enter/Space captured at OS level by Rust via global shortcuts
 * 2. Rust processes navigation and emits Tauri events
 * 3. This component listens for Tauri events and re-broadcasts as DOM events
 * 4. F11 (fullscreen) is the ONLY key captured here (webview-specific function)
 */
export default function Interface() {
  // Setup Tauri event listeners (events emitted from Rust input handler)
  // Uses module-level tracking to prevent duplicate listeners on hot reload
  onMount(async () => {
    // Skip if already setup (hot reload protection)
    if (tauriListenersSetup) {
      return;
    }
    tauriListenersSetup = true;
    
    // Listen for cursor-moved events from Rust
    const unlistenCursor = await listen<CursorMovedPayload>('cursor-moved', (event) => {
      window.dispatchEvent(new CustomEvent('cursor-moved', {
        detail: event.payload
      }));
    });
    tauriUnlisteners.push(unlistenCursor);
    
    // Listen for at-gate events from Rust
    const unlistenGate = await listen<AtGatePayload>('at-gate', (event) => {
      window.dispatchEvent(new CustomEvent('at-gate', {
        detail: event.payload
      }));
    });
    tauriUnlisteners.push(unlistenGate);
    
    // Listen for domain-switched events from Rust
    const unlistenDomain = await listen<DomainSwitchedPayload>('domain-switched', (event) => {
      window.dispatchEvent(new CustomEvent('domain-switched', {
        detail: event.payload
      }));
    });
    tauriUnlisteners.push(unlistenDomain);
    
    // Listen for boundary-reached events from Rust
    const unlistenBoundary = await listen<BoundaryReachedPayload>('boundary-reached', async (event) => {
      window.dispatchEvent(new CustomEvent('boundary-reached', {
        detail: event.payload
      }));

      // --- Custom Boundary Handling Logic ---
      // "Implicit Gates": Switch domains based on boundary hit direction
      try {
        const activeDomain = await invoke('get_active_domain') as string | null;
        
        if (activeDomain === 'osbar-nav' && event.payload.direction === 'right') {
          // Switch OSbar -> Window Header
          await invoke('set_active_domain', { domainId: 'window-header-nav' });
          await invoke('emit_cursor_position');
        } 
        else if (activeDomain === 'window-header-nav' && event.payload.direction === 'left') {
          // Switch Window Header -> OSbar
          await invoke('set_active_domain', { domainId: 'osbar-nav' });
          await invoke('emit_cursor_position');
        }
      } catch (error) {
        console.error('Boundary handler error:', error);
      }
    });
    tauriUnlisteners.push(unlistenBoundary);
    
    // Listen for button-activate events from Rust (Enter/Space on a button)
    const unlistenActivate = await listen<ButtonActivatePayload>('button-activate', (event) => {
      window.dispatchEvent(new CustomEvent('button-activate', { detail: event.payload }));
    });
    tauriUnlisteners.push(unlistenActivate);
  });
  
  // Cleanup Tauri listeners only on full unmount (not hot reload)
  onCleanup(() => {
    // Note: We keep listeners alive during hot reload
    // They will be cleaned up when the app fully unloads
  });

  // F11 fullscreen toggle - ONLY webview-specific key handling
  // All other input (WASD, Enter, Space) is captured by Rust at OS level
  onMount(() => {
    if (!f11HandlerRegistered) {
      f11HandlerRegistered = true;
      
      const handleF11 = async (e: KeyboardEvent) => {
        if (e.key === 'F11') {
          e.preventDefault();
          try {
            await invoke('toggle_fullscreen');
          } catch (error) {
            console.error('Failed to toggle fullscreen:', error);
          }
        }
        
        // Debug: 'X' key dumps navigation state (development only)
        if (e.key.toLowerCase() === 'x' && !e.ctrlKey && !e.altKey && !e.metaKey) {
          e.preventDefault();
          try {
            const domains = await invoke('get_all_domains') as string[];
            const cursor = await invoke('get_cursor_position') as any;
            console.log('=== Navigation State ===');
            console.log('Domains:', domains);
            console.log('Cursor:', cursor);
            
            for (const domainId of domains) {
              const domainInfo = await invoke('debug_domain', { domainId }) as any;
              console.log(`Domain '${domainId}':`, {
                buttons: domainInfo.buttons.map((b: any) => ({
                  id: b.id,
                  order: b.order,
                  enabled: b.enabled
                })),
                currentIndex: domainInfo.current_index,
                layoutMode: domainInfo.layout_mode
              });
            }
          } catch (error) {
            console.error('Failed to get nav state:', error);
          }
        }
      };

      window.addEventListener('keydown', handleF11);
      
      onCleanup(() => {
        window.removeEventListener('keydown', handleF11);
        f11HandlerRegistered = false;
      });
    }

    // Manage global shortcuts registration based on window focus
    if (!focusHandlersRegistered) {
      focusHandlersRegistered = true;

      const enableShortcuts = async () => {
        try {
          await invoke('set_global_shortcuts_enabled', { enabled: true });
        } catch (error) {
          console.error('Failed to enable global shortcuts:', error);
        }
      };

      const disableShortcuts = async () => {
        try {
          await invoke('set_global_shortcuts_enabled', { enabled: false });
        } catch (error) {
          console.error('Failed to disable global shortcuts:', error);
        }
      };

      // Enable on focus, disable on blur
      window.addEventListener('focus', enableShortcuts);
      window.addEventListener('blur', disableShortcuts);

      // Enable shortcuts on initial mount (since window is focused)
      enableShortcuts();

      // Note: We don't reset focusHandlersRegistered on cleanup
      // to prevent duplicate handlers during hot reload
    }
  });

  // Request initial cursor position from Rust after registrations complete
  onMount(async () => {
    // Wait for components to register their domains/buttons/gates
    await new Promise(resolve => setTimeout(resolve, 350));
    
    try {
      const emitted = await invoke('emit_cursor_position') as boolean;
      if (!emitted) {
        // No cursor position - set OSbar as active domain
        try {
          await invoke('set_active_domain', { domainId: 'osbar-nav' });
          await invoke('emit_cursor_position');
        } catch {
          // Fallback to any registered domain
          const domains = await invoke('get_all_domains') as string[];
          if (domains.length > 0) {
            await invoke('set_active_domain', { domainId: domains[0] });
            await invoke('emit_cursor_position');
          }
        }
      }
    } catch {
      // Silently ignore - cursor will be set on first navigation
    }
  });

  return (
    <>
      <OSbar_IC />
      <Compositor_IC 
        leftContent={
          <Window_IC>
            <TESTING_DUMMY />
          </Window_IC>
        }
      />
    </>
  );
}
