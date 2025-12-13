import { createStore } from "solid-js/store";

export type CompositorSlot = "Left" | "Right";
export type WindowState = "Minimized" | "Maximized" | "Hidden" | "Closing";

export interface WindowInstance {
  id: string;
  content_key: string;
  title: string;
  state: WindowState;
  slot: CompositorSlot;
  z_order: number;
  source_element_id?: string;
  source_domain_id?: string;
}

interface WindowStoreState {
  windows: WindowInstance[];
}

export const [windowStore, setWindowStore] = createStore<WindowStoreState>({
  windows: [],
});

export const addWindow = (window: WindowInstance) => {
  setWindowStore("windows", (windows) => [...windows, window]);
};

export const removeWindow = (id: string) => {
  setWindowStore("windows", (windows) => windows.filter((w) => w.id !== id));
};

export const updateWindow = (updatedWindow: WindowInstance) => {
  setWindowStore("windows", (w) => w.id === updatedWindow.id, updatedWindow);
};

// Slot-based helpers
export const getWindowInSlot = (slot: CompositorSlot) => 
  windowStore.windows.find((w) => w.slot === slot && w.state !== "Hidden");

export const getLeftWindow = () => getWindowInSlot("Left");
export const getRightWindow = () => getWindowInSlot("Right");
