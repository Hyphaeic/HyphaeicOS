import { onMount, onCleanup, JSX } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { addWindow, removeWindow, updateWindow, WindowInstance } from "../store";
import { findNavigationTarget } from "./navigation_helpers";

// ============================================================================
// CONTROLLER - The Centralized Input Hub (HMR-Safe Singleton)
// ============================================================================
// Uses a window-level singleton to ensure only ONE set of Tauri listeners exists,
// even through aggressive HMR reloads. DOM listeners are replaced on each mount.
// ============================================================================

// Event payload types from Rust
interface CursorMovedPayload {
  domain_id: string;
  element_id: string;
  element_type: string;
}

interface ElementActivatedPayload {
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

// Window action types for external triggering
export type WindowAction = "minimize" | "maximize" | "close";

// Exposed functions for triggering window actions externally (keybindings, etc.)
export const windowActions = {
  minimize: (windowId: string) => {
    invoke('set_window_state', { id: windowId, windowState: 'Minimized' }).catch(console.error);
  },
  maximize: (windowId: string) => {
    invoke('set_window_state', { id: windowId, windowState: 'Maximized' }).catch(console.error);
  },
  close: (windowId: string) => {
    invoke('close_window', { id: windowId }).catch(console.error);
  },
};

interface ControllerProps {
  children?: JSX.Element;
}

// Singleton state that survives HMR
interface ControllerState {
  tauriListenersActive: boolean;
  tauriUnlisteners: UnlistenFn[];
  domCleanup: (() => void) | null;
}

const getControllerState = (): ControllerState => {
  const key = '__HYPHA_CONTROLLER_STATE__';
  if (!(window as any)[key]) {
    (window as any)[key] = {
      tauriListenersActive: false,
      tauriUnlisteners: [],
      domCleanup: null,
    };
  }
  return (window as any)[key];
};

/**
 * Controller - The Centralized Input Hub
 * 
 * HMR-safe: Tauri listeners are set up ONCE and persist until page refresh.
 * DOM listeners are replaced on each mount to pick up code changes.
 */
export default function Controller(props: ControllerProps) {
  
  onMount(() => {
    const state = getControllerState();

    // -----------------------------------------------------------------------
    // DOM LISTENERS (replaced on each mount for HMR)
    // -----------------------------------------------------------------------
    
    // Clean up any existing DOM listeners from previous instance
    if (state.domCleanup) {
      state.domCleanup();
      state.domCleanup = null;
    }

    // KeyDown Handler
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.key === 'F11') {
        e.preventDefault();
        invoke('toggle_fullscreen').catch(console.error);
        return;
      }
      if (e.key === 'F12') return;
      
      if (import.meta.env.DEV && e.key.toLowerCase() === 'x' && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        try {
          const domains = await invoke('get_all_domains') as string[];
          const cursor = await invoke('get_cursor_position');
          console.log('=== [Controller] Navigation State ===');
          console.log('Domains:', domains);
          console.log('Cursor:', cursor);
        } catch (e) { console.error(e); }
      }
    };

    // Shortcut Management
    let isToggling = false;
    const enableShortcuts = async () => {
      if (isToggling) return;
      isToggling = true;
      try { await invoke('set_global_shortcuts_enabled', { enabled: true }); } 
      catch (e) { console.error(e); }
      finally { setTimeout(() => { isToggling = false; }, 100); }
    };

    const disableShortcuts = async () => {
      if (isToggling) return;
      isToggling = true;
      try { await invoke('set_global_shortcuts_enabled', { enabled: false }); }
      catch (e) { console.error(e); }
      finally { setTimeout(() => { isToggling = false; }, 100); }
    };

    // Attach DOM listeners
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('focus', enableShortcuts);
    window.addEventListener('blur', disableShortcuts);

    // Store cleanup for DOM listeners
    state.domCleanup = () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('focus', enableShortcuts);
      window.removeEventListener('blur', disableShortcuts);
    };

    // Initial shortcut enable
    if (document.hasFocus()) {
      enableShortcuts();
    }

    // -----------------------------------------------------------------------
    // TAURI LISTENERS (set up ONCE, persist until page refresh)
    // -----------------------------------------------------------------------
    
    if (!state.tauriListenersActive) {
      state.tauriListenersActive = true;
      
      (async () => {
        try {
          // Activation (Rust -> DOM Click)
          const u1 = await listen<ElementActivatedPayload>('button-activate', (event) => {
            const { element_id } = event.payload;
            
            // Map OSbar buttons to window spawning with source info
            if (element_id === 'osbar-btn-1') {
              invoke('spawn_window', { 
                contentKey: 'TESTING_DUMMY',
                sourceElementId: element_id,
                sourceDomainId: 'osbar-nav'
              }).catch(console.error);
            } else if (element_id === 'osbar-btn-2') {
              invoke('spawn_window', { 
                contentKey: 'EMPTY_WINDOW_2',
                sourceElementId: element_id,
                sourceDomainId: 'osbar-nav'
              }).catch(console.error);
            } else if (element_id === 'osbar-btn-3') {
              invoke('spawn_window', { 
                contentKey: 'EMPTY_WINDOW_3',
                sourceElementId: element_id,
                sourceDomainId: 'osbar-nav'
              }).catch(console.error);
            }

            const target = document.getElementById(element_id);
            if (target) {
              target.click();
            } else {
              window.dispatchEvent(new CustomEvent('button-activate', { detail: event.payload }));
            }
          });
          state.tauriUnlisteners.push(u1);

          // Window Created
          const u6 = await listen<WindowInstance>('window-created', (event) => {
            addWindow(event.payload);
          });
          state.tauriUnlisteners.push(u6);

          // Window Closed
          const u7 = await listen<string>('window-closed', (event) => {
            removeWindow(event.payload);
          });
          state.tauriUnlisteners.push(u7);

          // Window State Changed
          const u8 = await listen<WindowInstance>('window-state-changed', (event) => {
            updateWindow(event.payload);
          });
          state.tauriUnlisteners.push(u8);

          // Focus (Rust -> DOM Event)
          const u2 = await listen<CursorMovedPayload>('cursor-moved', (event) => {
            window.dispatchEvent(new CustomEvent('sys-cursor-move', { detail: event.payload }));
          });
          state.tauriUnlisteners.push(u2);

          // Return Focus (when window closed)
          const u9 = await listen<CursorMovedPayload>('return-focus', async (event) => {
            try {
              // Wait for DOM updates to process window removal
              await new Promise(r => setTimeout(r, 50));
              
              // Set active domain first
              await invoke('set_active_domain', { domainId: event.payload.domain_id });
              
              // Then emit cursor position to trigger visual update
              await invoke('emit_cursor_position');
            } catch (e) {
              console.error('[Controller] Failed to return focus:', e);
            }
          });
          state.tauriUnlisteners.push(u9);

          // Gate
          const u3 = await listen<AtGatePayload>('at-gate', (event) => {
            window.dispatchEvent(new CustomEvent('at-gate', { detail: event.payload }));
          });
          state.tauriUnlisteners.push(u3);

          // Domain Switched
          const u4 = await listen<DomainSwitchedPayload>('domain-switched', (event) => {
            window.dispatchEvent(new CustomEvent('domain-switched', { detail: event.payload }));
          });
          state.tauriUnlisteners.push(u4);

          // Boundary
          const u5 = await listen<BoundaryReachedPayload>('boundary-reached', async (event) => {
            window.dispatchEvent(new CustomEvent('boundary-reached', { detail: event.payload }));
            try {
              const activeDomain = await invoke('get_active_domain') as string | null;
              
              if (activeDomain) {
                const domains = await invoke('get_all_domains') as string[];
                const targetDomain = await findNavigationTarget(event.payload.direction, activeDomain, domains);
                
                if (targetDomain) {
                  await invoke('set_active_domain', { domainId: targetDomain });
                  await invoke('emit_cursor_position');
                }
              }
            } catch (error) {
              console.error('[Controller] Boundary handler error:', error);
            }
          });
          state.tauriUnlisteners.push(u5);

          console.log("[Controller] Tauri listeners registered");
        } catch (err) {
          console.error("[Controller] Failed to setup Tauri listeners", err);
          state.tauriListenersActive = false;
        }
      })();

      // Initial cursor setup (only on first mount)
      setTimeout(async () => {
        try {
          const emitted = await invoke('emit_cursor_position') as boolean;
          if (!emitted) {
            try {
              await invoke('set_active_domain', { domainId: 'osbar-nav' });
              await invoke('emit_cursor_position');
            } catch {
              const domains = await invoke('get_all_domains') as string[];
              if (domains.length > 0) {
                await invoke('set_active_domain', { domainId: domains[0] });
                await invoke('emit_cursor_position');
              }
            }
          }
        } catch {}
      }, 350);
    }
  });

  onCleanup(() => {
    const state = getControllerState();
    // Only clean up DOM listeners - Tauri listeners persist for HMR
    if (state.domCleanup) {
      state.domCleanup();
      state.domCleanup = null;
    }
    // NOTE: We do NOT clean up Tauri listeners or reset tauriListenersActive
    // They persist until page refresh to survive HMR
  });

  return <>{props.children}</>;
}
