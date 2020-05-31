import { Background } from "./background/lib";

declare global {
  interface Window {
    bg: Background;
  }
}

window.bg = new Background();
