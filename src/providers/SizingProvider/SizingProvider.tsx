// src/providers/SizingProvider.tsx
import { createContext, useContext, ParentComponent, createEffect } from "solid-js";
import { createStore } from "solid-js/store";

interface SizingConfig {
  // Base unit (could be viewport-relative or absolute)
  baseUnit: number;
  
  // Ratios for different component types
  ratios: {
    windowTitleBar: number;      // e.g., 2.5 (2.5x base unit)
    windowMinWidth: number;
    windowMinHeight: number;
    buttonHeight: number;
    buttonPadding: number;
    osBarHeight: number;
    borderWidth: number;
    borderRadius: number;
    fontSize: {
      small: number;
      medium: number;
      large: number;
    };
    spacing: {
      tight: number;
      normal: number;
      loose: number;
    };
  };
  
  // Viewport dimensions (reactive)
  viewport: {
    width: number;
    height: number;
  };
}

// Computed values based on ratios
interface ComputedSizes {
  windowTitleBar: string;
  windowMinWidth: string;
  windowMinHeight: string;
  buttonHeight: string;
  buttonPadding: string;
  osBarHeight: string;
  borderWidth: string;
  borderRadius: string;
  fontSize: {
    small: string;
    medium: string;
    large: string;
  };
  spacing: {
    tight: string;
    normal: string;
    loose: string;
  };
}

type SizingContextValue = [
  SizingConfig,
  {
    computed: ComputedSizes;
    updateBaseUnit: (unit: number) => void;
    setRatio: (path: string, value: number) => void;
  }
];

const SizingContext = createContext<SizingContextValue>();

export const SizingProvider: ParentComponent = (props) => {
  const [sizing, setSizing] = createStore<SizingConfig>({
    baseUnit: 16, // 16px base (could be vw/vh based)
    ratios: {
      windowTitleBar: 2.5,     // 40px
      windowMinWidth: 20,      // 320px
      windowMinHeight: 15,     // 240px
      buttonHeight: 2.5,       // 40px
      buttonPadding: 1,        // 16px
      osBarHeight: 3,          // 48px
      borderWidth: 0.0625,     // 1px
      borderRadius: 0.5,       // 8px
      fontSize: {
        small: 0.875,          // 14px
        medium: 1,             // 16px
        large: 1.5,            // 24px
      },
      spacing: {
        tight: 0.5,            // 8px
        normal: 1,             // 16px
        loose: 2,              // 32px
      },
    },
    viewport: {
      width: window.innerWidth,
      height: window.innerHeight,
    },
  });

  // Update viewport dimensions on resize
  createEffect(() => {
    const handleResize = () => {
      setSizing("viewport", {
        width: window.innerWidth,
        height: window.innerHeight,
      });
      
      // Optional: Recalculate base unit based on viewport
      // setSizing("baseUnit", Math.min(window.innerWidth, window.innerHeight) * 0.01);
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  });

  // Computed values (automatically reactive)
  const computed = (): ComputedSizes => ({
    windowTitleBar: `${sizing.baseUnit * sizing.ratios.windowTitleBar}px`,
    windowMinWidth: `${sizing.baseUnit * sizing.ratios.windowMinWidth}px`,
    windowMinHeight: `${sizing.baseUnit * sizing.ratios.windowMinHeight}px`,
    buttonHeight: `${sizing.baseUnit * sizing.ratios.buttonHeight}px`,
    buttonPadding: `${sizing.baseUnit * sizing.ratios.buttonPadding}px`,
    osBarHeight: `${sizing.baseUnit * sizing.ratios.osBarHeight}px`,
    borderWidth: `${sizing.baseUnit * sizing.ratios.borderWidth}px`,
    borderRadius: `${sizing.baseUnit * sizing.ratios.borderRadius}px`,
    fontSize: {
      small: `${sizing.baseUnit * sizing.ratios.fontSize.small}px`,
      medium: `${sizing.baseUnit * sizing.ratios.fontSize.medium}px`,
      large: `${sizing.baseUnit * sizing.ratios.fontSize.large}px`,
    },
    spacing: {
      tight: `${sizing.baseUnit * sizing.ratios.spacing.tight}px`,
      normal: `${sizing.baseUnit * sizing.ratios.spacing.normal}px`,
      loose: `${sizing.baseUnit * sizing.ratios.spacing.loose}px`,
    },
  });

  const updateBaseUnit = (unit: number) => {
    setSizing("baseUnit", unit);
  };

  const setRatio = (path: string, value: number) => {
    setSizing("ratios", path as any, value);
  };

  const value: SizingContextValue = [
    sizing,
    {
      computed: computed(),
      updateBaseUnit,
      setRatio,
    },
  ];

  return (
    <SizingContext.Provider value={value}>
      {props.children}
    </SizingContext.Provider>
  );
};

export function useSizing() {
  const context = useContext(SizingContext);
  if (!context) {
    throw new Error("useSizing must be used within a SizingProvider");
  }
  return context;
}