import { createSignal, onMount, onCleanup, createEffect, JSX } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useDomain } from "../A_Domain/Domain";
import "./Button_IS.css";

// Module-level tracking to prevent duplicate event listeners during hot reload
// Key: button ID, Value: cleanup function
const activeCursorListeners = new Map<string, () => void>();
const activeActivateListeners = new Map<string, () => void>();

interface ButtonProps {
  /** Unique identifier for this button */
  id: string;
  /** Domain this button belongs to (auto-detected from context if not provided) */
  domainId?: string;
  /** Sequential order for list/grid navigation */
  order: number;
  /** Click handler */
  onClick?: () => void;
  /** Button content */
  children?: JSX.Element;
  /** Optional class name */
  class?: string;
}

/**
 * NavigableButton - A button that integrates with the Rust WASD navigation system
 * 
 * Automatically registers/unregisters with the DomainNavigator on mount/unmount.
 * Waits for domain to be ready before registering.
 * Listens for cursor-moved events to show focus state.
 */
export default function Button_IC(props: ButtonProps) {
  const [isFocused, setIsFocused] = createSignal(false);
  const [isRegistered, setIsRegistered] = createSignal(false);
  const [isActive, setIsActive] = createSignal(false);
  let buttonRef: HTMLButtonElement | undefined;

  // Get domain context (if within a Domain component)
  const domainContext = useDomain();
  
  // Use provided domainId or get from context
  const getDomainId = () => props.domainId ?? domainContext?.id ?? '';

  // Async registration function
  const registerButton = async (domainId: string) => {
    try {
      // Get button bounds for spatial navigation
      const rect = buttonRef?.getBoundingClientRect();
      const bounds = rect ? {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height
      } : null;

      await invoke('register_button', {
        domainId: domainId,
        buttonId: props.id,
        bounds,
        order: props.order
      });

      setIsRegistered(true);

      // Check if this button is initially focused
      const cursor = await invoke('get_cursor_position');
      if (cursor && typeof cursor === 'object' && 'element_id' in cursor) {
        setIsFocused(
          (cursor as any).element_id === props.id &&
          (cursor as any).domain_id === domainId
        );
      }
    } catch (error) {
      const msg = String(error);
      if (msg.includes("already exists")) {
        // Already registered (e.g., from hot reload) - just mark as registered
        setIsRegistered(true);
        
        // Update focus state just in case
        const cursor = await invoke('get_cursor_position');
        if (cursor && typeof cursor === 'object' && 'element_id' in cursor) {
          setIsFocused(
            (cursor as any).element_id === props.id &&
            (cursor as any).domain_id === domainId
          );
        }
      } else {
        console.error(`Failed to register button ${props.id}:`, error);
      }
    }
  };

  // Wait for domain ready, then register
  // Using createEffect to track the isReady signal
  createEffect(() => {
    // Must read the isReady signal to create reactive dependency
    const isDomainReady = domainContext?.isReady() ?? true;
    
    // Early exit if domain not ready - effect will re-run when it becomes ready
    if (!isDomainReady) {
      return;
    }

    // Early exit if already registered
    if (isRegistered()) {
      return;
    }

    const domainId = getDomainId();
    if (!domainId) {
      console.warn(`Button ${props.id} has no domain ID`);
      return;
    }

    // Register the button
    registerButton(domainId);
  });

  // Unregister button on cleanup
  onCleanup(async () => {
    if (!isRegistered()) return;
    
    const domainId = getDomainId();
    if (!domainId) return;

    try {
      await invoke('unregister_button', {
        domainId: domainId,
        buttonId: props.id
      });
    } catch {
      // Silently ignore - button may have been unregistered by domain cleanup
    }
  });

  // Listen for cursor-moved events from global handler
  // Uses module-level tracking to prevent duplicate listeners during hot reload
  onMount(() => {
    // Clean up any existing listener for this button ID first
    const existingCleanup = activeCursorListeners.get(props.id);
    if (existingCleanup) {
      existingCleanup();
    }
    
    const handleCursorMoved = (event: CustomEvent) => {
      const detail = event.detail;
      const domainId = getDomainId();
      setIsFocused(
        detail.element_id === props.id &&
        detail.domain_id === domainId
      );
    };

    window.addEventListener('cursor-moved', handleCursorMoved as EventListener);
    
    // Store cleanup function in module-level map
    const cleanup = () => {
      window.removeEventListener('cursor-moved', handleCursorMoved as EventListener);
      activeCursorListeners.delete(props.id);
    };
    activeCursorListeners.set(props.id, cleanup);
    
    onCleanup(cleanup);
  });

  // Update bounds on resize (only if registered)
  // Note: Uses module-level tracking to prevent duplicate handlers from hot reload
  onMount(() => {
    const handleResize = async () => {
      if (!buttonRef || !isRegistered()) return;
      
      const domainId = getDomainId();
      if (!domainId) return;

      const rect = buttonRef.getBoundingClientRect();
      const bounds = {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height
      };

      try {
        // Unregister and re-register with new bounds
        await invoke('unregister_button', {
          domainId: domainId,
          buttonId: props.id
        });
        await invoke('register_button', {
          domainId: domainId,
          buttonId: props.id,
          bounds,
          order: props.order
        });
      } catch {
        // Silently ignore - button may have been cleaned up
      }
    };

    window.addEventListener('resize', handleResize);
    
    onCleanup(() => {
      window.removeEventListener('resize', handleResize);
    });
  });

  // Handle click - trigger action when button is clicked
  const handleClick = () => {
    if (props.onClick) {
      props.onClick();
    }
  };

  // Listen for button-activate event from Rust (Enter/Space pressed on this button)
  // Uses module-level tracking to prevent duplicate listeners during hot reload
  onMount(() => {
    // Clean up any existing listener for this button ID first
    const existingCleanup = activeActivateListeners.get(props.id);
    if (existingCleanup) {
      existingCleanup();
    }
    
    const handleActivate = (event: CustomEvent) => {
      const detail = event.detail;
      const domainId = getDomainId();
      
      // Check if this button was activated
      if (detail.element_id === props.id && detail.domain_id === domainId) {
        // Visual feedback: flash active state
        setIsActive(true);
        setTimeout(() => setIsActive(false), 150);
        
        // Trigger the click handler
        handleClick();
      }
    };

    window.addEventListener('button-activate', handleActivate as EventListener);
    
    // Store cleanup function in module-level map
    const cleanup = () => {
      window.removeEventListener('button-activate', handleActivate as EventListener);
      activeActivateListeners.delete(props.id);
    };
    activeActivateListeners.set(props.id, cleanup);
    
    onCleanup(cleanup);
  });

  return (
    <button
      ref={buttonRef}
      class={`nav-button ${isFocused() ? 'nav-button-focused' : ''} ${isActive() ? 'nav-button-active' : ''} ${props.class || ''}`}
      onClick={handleClick}
      data-button-id={props.id}
      data-domain-id={getDomainId()}
    >
      {props.children}
    </button>
  );
}
