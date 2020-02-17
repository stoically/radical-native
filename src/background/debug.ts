declare global {
  interface Window {
    DEBUG: boolean;
  }
}

window.DEBUG = false;

export const debug = (...args: any[]): void => {
  if (!window.DEBUG) {
    return;
  }
  console.log("[RadicalNative::background]", ...args);
};
