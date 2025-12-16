import { onMount, onCleanup, JSX, createContext, useContext, createSignal, Accessor, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Rect type matching Rust backend
interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Direction type for boundary lock
type BoundaryDirection = 'top' | 'bottom' | 'left' | 'right';

interface DomainProps {
  /** Unique identifier for this domain */
  id: string;
  /** Parent domain ID (for nested domains) */
  parentDomain?: string;
  /** Layout mode: 'list-vertical', 'list-horizontal', 'grid', or 'spatial' */
  layoutMode: 'list-vertical' | 'list-horizontal' | 'grid' | 'spatial';
  /** Number of columns (required for grid layout) */
  gridColumns?: number;
  /** Child elements */
  children?: JSX.Element;
  /** Optional class name */
  class?: string;
  /** Directions where cursor cannot exit this domain (for spatial navigation) */
  boundaryLock?: BoundaryDirection[];
}

// Context to provide domain info to child buttons
interface DomainContextValue {
  id: string;
  isReady: Accessor<boolean>;
}

const DomainContext = createContext<DomainContextValue>();

// Module-level tracking to prevent HMR/dev mode double-registration
const registeredDomains = new Set<string>();

/**
 * Get the current domain context (id and ready state)
 */
export function useDomain(): DomainContextValue | undefined {
  return useContext(DomainContext);
}

/**
 * Get the current domain ID from context (convenience function)
 */
export function useDomainId(): string | undefined {
  return useContext(DomainContext)?.id;
}

/**
 * Domain - A container that registers with the Rust navigation system
 * 
 * Provides context for child Button components to know their domain.
 * Automatically registers/unregisters with the DomainNavigator on mount/unmount.
 * Child buttons wait for domain registration before registering themselves.
 * Reports its screen bounds for spatial navigation between domains.
 */
export default function Domain(props: DomainProps) {
  const [isReady, setIsReady] = createSignal(false);
  let containerRef: HTMLDivElement | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let isMounted = true; // Track mount state to prevent stale updates

  // Cache last sent bounds to prevent spamming the backend
  let lastBounds: Rect | null = null;
  let updateAnimationFrame: number | null = null;

  /**
   * Calculate and report domain bounds to Rust backend
   * IMPORTANT: Only sends bounds if they have non-zero dimensions
   */
  const updateBounds = async () => {
    // Skip if unmounted or no ref
    if (!isMounted || !containerRef) {
      return;
    }

    // Cancel any pending frame
    if (updateAnimationFrame) {
      cancelAnimationFrame(updateAnimationFrame);
    }

    updateAnimationFrame = requestAnimationFrame(async () => {
      if (!isMounted || !containerRef) return;

      const rect = containerRef.getBoundingClientRect();

      // CRITICAL: Skip if bounds are zero (element is hidden/not rendered)
      if (rect.width === 0 || rect.height === 0) {
        // Only log once if we haven't already
        if (lastBounds !== null) {
          // Optional: could log here, but usually noisy
        }
        return;
      }

      const newBounds: Rect = {
        x: Math.round(rect.left),
        y: Math.round(rect.top),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
      };

      // Diffing: Check if bounds actually changed
      if (lastBounds &&
        Math.abs(newBounds.x - lastBounds.x) < 2 &&
        Math.abs(newBounds.y - lastBounds.y) < 2 &&
        Math.abs(newBounds.width - lastBounds.width) < 2 &&
        Math.abs(newBounds.height - lastBounds.height) < 2) {
        return;
      }

      lastBounds = newBounds;
      // console.log(`[Domain ${props.id}] Updating bounds:`, newBounds);

      try {
        await invoke('update_domain_bounds', {
          domainId: props.id,
          bounds: newBounds,
        });
      } catch (error) {
        if (isMounted) {
          console.error(`[Domain ${props.id}] Failed to update bounds:`, error);
        }
      }
    });
  };

  // Handle window resize (updates position even if element size unchanged)
  const handleWindowResize = () => {
    if (isMounted && isReady()) {
      updateBounds();
    }
  };

  // Register domain with Rust backend on mount
  onMount(async () => {
    console.log(`[Domain ${props.id}] Mounting, layoutMode=${props.layoutMode}`);

    try {
      await invoke('register_domain', {
        domainId: props.id,
        parentDomain: props.parentDomain || null,
        layoutMode: props.layoutMode,
        gridColumns: props.gridColumns || null
      });

      console.log(`[Domain ${props.id}] Registered successfully`);
      setIsReady(true);

      // After registration, report initial bounds
      // Use requestAnimationFrame to ensure layout is complete
      requestAnimationFrame(() => {
        updateBounds();

        // Set up resize observer AFTER the first bounds update
        if (containerRef) {
          resizeObserver = new ResizeObserver(() => {
            updateBounds();
          });
          resizeObserver.observe(containerRef);
        }
      });

      // Also listen for window resize (affects position when viewport changes)
      window.addEventListener('resize', handleWindowResize);
    } catch (error) {
      const msg = String(error);
      if (msg.includes("already exists")) {
        console.log(`[Domain ${props.id}] Already registered (hot reload?)`);
        setIsReady(true);
        requestAnimationFrame(() => {
          updateBounds();
        });
      } else {
        console.error(`[Domain ${props.id}] Failed to register:`, error);
      }
    }
  });

  // React to layoutMode prop changes
  createEffect(() => {
    const mode = props.layoutMode;
    if (isReady()) {
      console.log(`[Domain ${props.id}] Layout mode changed to: ${mode}`);
      // Update bounds when layout mode changes
      requestAnimationFrame(() => {
        updateBounds();
      });
    }
  });

  // Unregister domain on cleanup
  onCleanup(() => {
    console.log(`[Domain ${props.id}] Cleaning up`);
    isMounted = false; // Set immediately to prevent further async ops

    // Remove from module-level registry
    registeredDomains.delete(props.id);

    // Remove window resize listener
    window.removeEventListener('resize', handleWindowResize);

    // Disconnect resize observer
    if (resizeObserver) {
      resizeObserver.disconnect();
    }

    // Unregister domain (don't await to avoid stale access)
    invoke('unregister_domain', { domainId: props.id }).catch(() => {
      // Silently ignore
    });
  });

  const contextValue: DomainContextValue = {
    id: props.id,
    isReady
  };

  return (
    <DomainContext.Provider value={contextValue}>
      <div
        ref={containerRef}
        class={`domain ${props.class || ''}`}
        data-domain-id={props.id}
      >
        {props.children}
      </div>
    </DomainContext.Provider>
  );
}

