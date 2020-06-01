declare global {
  interface Window {
    DEBUG: boolean;
  }
}

window.DEBUG = process.env.NODE_ENV === "development" ? true : false;

export const debug = (...args: any[]): void => {
  if (!window.DEBUG) {
    return;
  }
  console.log("[RadicalNative::background]", ...args);
};
