import { createSignal, onMount, onCleanup, createEffect, JSX } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useDomain } from "../B_Domain/Domain";
import "./Button_IS.css";

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
      console.log(`Button '${props.id}' registered in domain '${domainId}' with order ${props.order}`);

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
        console.log(`Button '${props.id}' already registered in '${domainId}'`);
        // If already registered, still mark as registered so we can receive updates
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
      console.log(`Button '${props.id}' waiting for domain...`);
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
      console.log(`Button '${props.id}' unregistered from domain '${domainId}'`);
    } catch (error) {
      console.error(`Failed to unregister button ${props.id}:`, error);
    }
  });

  // Listen for cursor-moved events from global handler
  onMount(() => {
    const handleCursorMoved = (event: CustomEvent) => {
      const detail = event.detail;
      const domainId = getDomainId();
      setIsFocused(
        detail.element_id === props.id &&
        detail.domain_id === domainId
      );
    };

    window.addEventListener('cursor-moved', handleCursorMoved as EventListener);
    
    onCleanup(() => {
      window.removeEventListener('cursor-moved', handleCursorMoved as EventListener);
    });
  });

  // Update bounds on resize (only if registered)
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
      } catch (error) {
        console.error(`Failed to update bounds for button ${props.id}:`, error);
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

  // Handle Enter/Space key when focused
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (isFocused() && (e.key === 'Enter' || e.key === ' ')) {
        e.preventDefault();
        handleClick();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    
    onCleanup(() => {
      window.removeEventListener('keydown', handleKeyDown);
    });
  });

  return (
    <button
      ref={buttonRef}
      class={`nav-button ${isFocused() ? 'nav-button-focused' : ''} ${props.class || ''}`}
      onClick={handleClick}
      data-button-id={props.id}
      data-domain-id={getDomainId()}
    >
      {props.children}
    </button>
  );
}
