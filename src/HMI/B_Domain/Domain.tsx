import { onMount, onCleanup, JSX, createContext, useContext, createSignal, Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

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
}

// Context to provide domain info to child buttons
interface DomainContextValue {
  id: string;
  isReady: Accessor<boolean>;
}

const DomainContext = createContext<DomainContextValue>();

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
 */
export default function Domain(props: DomainProps) {
  const [isReady, setIsReady] = createSignal(false);

  // Register domain with Rust backend on mount
  onMount(async () => {
    try {
      await invoke('register_domain', {
        domainId: props.id,
        parentDomain: props.parentDomain || null,
        layoutMode: props.layoutMode,
        gridColumns: props.gridColumns || null
      });
      
      console.log(`Domain '${props.id}' registered with layout: ${props.layoutMode}`);
      
      // Set domain as active (first domain to register becomes active by default in Rust,
      // but we explicitly set it to be sure)
      await invoke('set_active_domain', { domainId: props.id });
      console.log(`Domain '${props.id}' set as active`);
      
      setIsReady(true);
    } catch (error) {
      console.error(`Failed to register domain ${props.id}:`, error);
    }
  });

  // Unregister domain on cleanup
  onCleanup(async () => {
    try {
      await invoke('unregister_domain', {
        domainId: props.id
      });
      
      console.log(`Domain '${props.id}' unregistered`);
    } catch (error) {
      console.error(`Failed to unregister domain ${props.id}:`, error);
    }
  });

  const contextValue: DomainContextValue = {
    id: props.id,
    isReady
  };

  return (
    <DomainContext.Provider value={contextValue}>
      <div class={`domain ${props.class || ''}`} data-domain-id={props.id}>
        {props.children}
      </div>
    </DomainContext.Provider>
  );
}
