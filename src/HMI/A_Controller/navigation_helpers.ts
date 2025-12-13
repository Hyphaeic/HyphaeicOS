import { windowStore } from "../store";

// Helper to get window domains
export async function getWindowDomains(allDomains: string[]) {
  return allDomains.filter(d => d.endsWith('-header-nav'));
}

// Helper to find navigation target
export async function findNavigationTarget(
  direction: string, 
  currentDomain: string, 
  allDomains: string[]
): Promise<string | null> {
  const windowDomains = await getWindowDomains(allDomains);
  
  // OSBar -> Window Logic
  if (currentDomain === 'osbar-nav' && direction === 'right') {
    // Prefer Left slot, then Right slot
    const leftWin = windowStore.windows.find(w => w.slot === 'Left' && w.state !== 'Hidden');
    if (leftWin) {
      const target = `${leftWin.id}-header-nav`;
      if (allDomains.includes(target)) return target;
    }
    
    const rightWin = windowStore.windows.find(w => w.slot === 'Right' && w.state !== 'Hidden');
    if (rightWin) {
      const target = `${rightWin.id}-header-nav`;
      if (allDomains.includes(target)) return target;
    }
  }
  
  // Window -> OSBar Logic
  if (currentDomain.endsWith('-header-nav') && direction === 'left') {
    // Check if we are in the Left slot
    const winId = currentDomain.replace('-header-nav', '');
    const win = windowStore.windows.find(w => w.id === winId);
    
    if (win && win.slot === 'Left') {
      return 'osbar-nav';
    }
    
    // If we are in Right slot, check if there is a Left slot window
    if (win && win.slot === 'Right') {
      const leftWin = windowStore.windows.find(w => w.slot === 'Left' && w.state !== 'Hidden');
      if (leftWin) {
        // Navigate to Left window
        const target = `${leftWin.id}-header-nav`;
        if (allDomains.includes(target)) return target;
      } else {
        // No left window, go to OSBar
        return 'osbar-nav';
      }
    }
  }
  
  // Window -> Window Logic (Right)
  if (currentDomain.endsWith('-header-nav') && direction === 'right') {
    const winId = currentDomain.replace('-header-nav', '');
    const win = windowStore.windows.find(w => w.id === winId);
    
    // If we are in Left slot, look for Right slot window
    if (win && win.slot === 'Left') {
      const rightWin = windowStore.windows.find(w => w.slot === 'Right' && w.state !== 'Hidden');
      if (rightWin) {
        const target = `${rightWin.id}-header-nav`;
        if (allDomains.includes(target)) return target;
      }
    }
  }
  
  return null;
}
