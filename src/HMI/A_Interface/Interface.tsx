import { onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import OSbar_IC from "../OSbar/OSbar_IC";

// Module-level flag to prevent concurrent navigation processing
// Persists across hot reloads
let isProcessingNavigation = false;

// Track if handler is already registered (for hot reload protection)
let handlerRegistered = false;

/**
 * Interface component - Core composer for OSbar and other visual elements
 * Also handles global WASD keyboard navigation
 */
export default function Interface() {
  
  // Setup global WASD keyboard handler
  onMount(() => {
    // Prevent duplicate handler registration during hot reload
    if (handlerRegistered) {
      console.warn('Keyboard handler already registered, skipping...');
      return;
    }
    handlerRegistered = true;
    
    // Debug: log all registered domains and buttons
    const debugNavState = async () => {
      try {
        const domains = await invoke('get_all_domains') as string[];
        const cursor = await invoke('get_cursor_position') as any;
        console.log('=== Navigation State ===');
        console.log('Domains:', domains);
        console.log('Cursor:', cursor);
        
        // Get detailed info for each domain
        for (const domainId of domains) {
          try {
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
          } catch (e) {
            console.error(`Failed to get info for domain ${domainId}:`, e);
          }
        }
      } catch (error) {
        console.error('Failed to get nav state:', error);
      }
    };

    // Wait a bit for components to register, then show state
    setTimeout(debugNavState, 1000);

    const handleKeyDown = async (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      
      // Debug: press 'x' key alone to show navigation state
      if (key === 'x' && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        await debugNavState();
        return;
      }
      
      // Handle WASD navigation
      if (['w', 'a', 's', 'd'].includes(key)) {
        // Ignore key repeat events (when key is held down)
        // This prevents double-firing while keeping first press snappy
        if (e.repeat) {
          e.preventDefault();
          return;
        }
        
        // Prevent concurrent navigation processing
        if (isProcessingNavigation) {
          e.preventDefault();
          return;
        }
        
        e.preventDefault();
        isProcessingNavigation = true;
        
        try {
          const result = await invoke('handle_wasd_input', {
            key: key.toUpperCase()
          }) as any;

          console.log(`[${key.toUpperCase()}] Navigation result:`, result);

          // Emit cursor-moved event for frontend components
          if (result.type === 'CursorMoved') {
            window.dispatchEvent(new CustomEvent('cursor-moved', {
              detail: {
                domain_id: result.domain_id,
                element_id: result.element_id,
                element_type: result.element_type
              }
            }));
          } else if (result.type === 'AtGate') {
            console.log('At gate! Press Enter to switch domains');
            // Show visual indicator for gate
            window.dispatchEvent(new CustomEvent('at-gate', {
              detail: {
                gate_id: result.gate_id,
                target_domain: result.target_domain
              }
            }));
          } else if (result.type === 'BoundaryReached') {
            console.log('Boundary reached');
          }
        } catch (error) {
          console.error('WASD navigation error:', error);
        } finally {
          // Reset flag after processing completes
          isProcessingNavigation = false;
        }
      }
      
      // Handle Enter/Space for domain switching at gates or button activation
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        
        try {
          const cursor = await invoke('get_cursor_position') as any;
          if (cursor && cursor.element_type === 'Gate') {
            const result = await invoke('switch_domain') as any;
            
            if (result.type === 'DomainSwitched') {
              console.log(`Switched from ${result.from_domain} to ${result.to_domain}`);
              window.dispatchEvent(new CustomEvent('cursor-moved', {
                detail: {
                  domain_id: result.to_domain,
                  element_id: result.new_element_id,
                  element_type: 'Button'
                }
              }));
            }
          }
          // Button activation is handled in Button_IC
        } catch (error) {
          // Not at a gate or other error - this is expected
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    
    onCleanup(() => {
      window.removeEventListener('keydown', handleKeyDown);
      handlerRegistered = false;
      isProcessingNavigation = false;
    });
  });

  // Dispatch initial cursor position after component mounts and registrations complete
  onMount(async () => {
    // Wait for registrations (adjust timing if needed)
    await new Promise(resolve => setTimeout(resolve, 200));
    
    try {
      const cursor = await invoke('get_cursor_position') as any;
      if (cursor && cursor.element_id) {
        console.log('Dispatching initial cursor position:', cursor);
        window.dispatchEvent(new CustomEvent('cursor-moved', {
          detail: {
            domain_id: cursor.domain_id,
            element_id: cursor.element_id,
            element_type: cursor.element_type
          }
        }));
      } else {
        console.warn('No initial cursor position - trying to set default');
        // Fallback: try to set active domain and first element
        try {
          const domains = await invoke('get_all_domains') as string[];
          if (domains.length > 0) {
            await invoke('set_active_domain', { domainId: domains[0] });
            const newCursor = await invoke('get_cursor_position') as any;
            if (newCursor && newCursor.element_id) {
              window.dispatchEvent(new CustomEvent('cursor-moved', {
                detail: {
                  domain_id: newCursor.domain_id,
                  element_id: newCursor.element_id,
                  element_type: newCursor.element_type
                }
              }));
            }
          }
        } catch (fallbackError) {
          console.error('Fallback cursor setup failed:', fallbackError);
        }
      }
    } catch (error) {
      console.error('Failed to get initial cursor position:', error);
    }
  });

  return (
    <>
      <OSbar_IC />
      {/* Add other visual elements here as needed */}
    </>
  );
}
