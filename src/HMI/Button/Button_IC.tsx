import { createSignal, onMount, onCleanup, createEffect, JSX } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useDomain } from "../A_Domain/Domain";
import "./Button_IS.css";

// ============================================================================
// BUTTON_IC - The "Marker" Component
// ============================================================================
// A lightweight, composable button that:
// 1. Registers itself with the Rust navigation system
// 2. Reacts to DOM events dispatched by Controller.tsx
// 3. Has NO direct Tauri event listeners (only DOM events)
// ============================================================================

interface ButtonProps {
  /** Unique identifier for this button - MUST be unique in the DOM */
  id: string;
  /** Domain this button belongs to (auto-detected from context if not provided) */
  domainId?: string;
  /** Sequential order for list/grid navigation */
  order: number;
  /** Click handler - called on mouse click OR keyboard activation */
  onClick?: () => void;
  /** Button content */
  children?: JSX.Element;
  /** Optional class name(s) - merged with internal classes */
  class?: string;
}

/**
 * Button_IC - A navigable button marker component
 * 
 * This component is a "marker" that:
 * - Registers with Rust backend for WASD navigation
 * - Reacts to `sys-cursor-move` DOM events for focus styling
 * - Receives clicks from both mouse AND keyboard (via Controller.tsx)
 * 
 * Architecture:
 * - Controller.tsx handles all Tauri events
 * - Controller dispatches `sys-cursor-move` for focus changes
 * - Controller calls `.click()` on this button for keyboard activation
 */
export default function Button_IC(props: ButtonProps) {
  const [isFocused, setIsFocused] = createSignal(false);
  const [isActive, setIsActive] = createSignal(false);
  const [isRegistered, setIsRegistered] = createSignal(false);
  let buttonRef: HTMLButtonElement | undefined;

  // Get domain context (if within a Domain component)
  const domainContext = useDomain();
  
  // Use provided domainId or get from context
  const getDomainId = () => props.domainId ?? domainContext?.id ?? '';

  // =========================================================================
  // RUST REGISTRATION (Button -> Rust Backend)
  // =========================================================================
  
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
      try {
        const cursor = await invoke('get_cursor_position');
        if (cursor && typeof cursor === 'object' && 'element_id' in cursor) {
          setIsFocused(
            (cursor as any).element_id === props.id &&
            (cursor as any).domain_id === domainId
          );
        }
      } catch {
        // Cursor position not available yet - will be set by first sys-cursor-move
      }
    } catch (error) {
      const msg = String(error);
      if (msg.includes("already exists")) {
        // Already registered (e.g., from hot reload)
        setIsRegistered(true);
      } else {
        console.error(`[Button_IC] Failed to register ${props.id}:`, error);
      }
    }
  };

  // Wait for domain ready, then register
  createEffect(() => {
    const isDomainReady = domainContext?.isReady() ?? true;
    
    if (!isDomainReady || isRegistered()) {
      return;
    }

    const domainId = getDomainId();
    if (!domainId) {
      console.warn(`[Button_IC] ${props.id} has no domain ID`);
      return;
    }

    registerButton(domainId);
  });

  // Unregister on cleanup
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
      // Silently ignore - may have been cleaned up by domain
    }
  });

  // =========================================================================
  // FOCUS REACTIVITY (Controller -> Button via DOM event)
  // =========================================================================
  
  onMount(() => {
    const handleFocus = (e: CustomEvent) => {
      // Am I the target?
      const isMe = e.detail.element_id === props.id;
      setIsFocused(isMe);
    };

    window.addEventListener('sys-cursor-move', handleFocus as EventListener);
    
    onCleanup(() => {
      window.removeEventListener('sys-cursor-move', handleFocus as EventListener);
    });
  });

  // =========================================================================
  // RESIZE HANDLER (Update bounds for spatial navigation)
  // =========================================================================
  
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
        // Use update_button_bounds instead of unregister/re-register
        // This preserves cursor state and is much simpler
        await invoke('update_button_bounds', {
          domainId: domainId,
          buttonId: props.id,
          bounds
        });
      } catch {
        // Silently ignore - button may have been unregistered
      }
    };

    window.addEventListener('resize', handleResize);
    
    onCleanup(() => {
      window.removeEventListener('resize', handleResize);
    });
  });

  // =========================================================================
  // CLICK HANDLER (Mouse click OR Controller.tsx keyboard activation)
  // =========================================================================
  
  const handleClick = () => {
    // Visual feedback: flash active state
    setIsActive(true);
    setTimeout(() => setIsActive(false), 150);
    
    // Call user's onClick handler
    if (props.onClick) {
      props.onClick();
    }
  };

  // =========================================================================
  // RENDER
  // =========================================================================
  
  // Build class string: internal classes + user's custom classes
  const getClassName = () => {
    const classes = ['nav-button'];
    if (isFocused()) classes.push('nav-button-focused');
    if (isActive()) classes.push('nav-button-active');
    if (props.class) classes.push(props.class);
    return classes.join(' ');
  };

  return (
    <button
      ref={buttonRef}
      id={props.id}
      tabIndex={-1}
      onMouseDown={(e) => e.preventDefault()}
      class={getClassName()}
      onClick={handleClick}
      data-button-id={props.id}
      data-domain-id={getDomainId()}
    >
      {props.children}
    </button>
  );
}
