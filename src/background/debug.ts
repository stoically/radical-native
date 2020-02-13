declare global {
  interface Window {
    DEBUG: boolean;
  }
}

window.DEBUG = true;

export const debug = (...args: any[]): void => {
  if (!window.DEBUG) {
    return;
  }
  console.log(...args);
};
