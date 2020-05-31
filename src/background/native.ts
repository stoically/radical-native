import { debug } from "./debug";

export class NativePort {
  private port?: browser.runtime.Port;
  private rpcId = 0;
  private rpcPromises: Map<number, any> = new Map();
  private ready = false;

  constructor() {
    this.init();
  }

  async handleRuntimeMessage(message: any): Promise<any> {
    debug("message for seshat received", message);
    switch (message.method) {
      case "supportsEventIndexing":
        return this.ready;

      default:
        return this.postMessage(message);
    }
  }

  private init(): void {
    this.port = browser.runtime.connectNative("radical.native");
    this.port.onDisconnect.addListener(this.handleDisconnect.bind(this));
    this.port.onMessage.addListener(this.handleMessage.bind(this));
  }

  private close(): void {
    this.ready = false;
    this.port?.onDisconnect.removeListener(this.handleDisconnect.bind(this));
    this.port?.onMessage.removeListener(this.handleMessage.bind(this));
    delete this.port;
  }

  private postMessage(message: any): Promise<void> {
    return new Promise((resolve, reject) => {
      this.rpcId++;
      // eslint-disable-next-line @typescript-eslint/camelcase
      message.rpc_id = this.rpcId;
      this.rpcPromises.set(this.rpcId, {
        message,
        resolve,
        reject,
      });
      debug("posting to radical.native", message);
      this.port?.postMessage(message);
    });
  }

  private handleMessage(message: any): void {
    if (message.ready) {
      debug("port ready");
      browser.browserAction.enable();
      browser.browserAction.setTitle({ title: "Radical Native" });
      browser.browserAction.setBadgeText({ text: null });
      this.ready = true;
      return;
    }

    const rpcPromise = this.rpcPromises.get(message.rpc_id);
    if (!rpcPromise) {
      debug("port message received without matching rpcPromise", message);
      return;
    }

    if (!message.error) {
      debug("port message received", {
        message,
        origExternalMessage: rpcPromise.message,
      });
      rpcPromise.resolve(message.reply);
    } else {
      console.error("port error received", {
        error: message.error,
        origExternalMessage: rpcPromise.message,
      });
      rpcPromise.reject(new Error(message.error));
    }
    this.rpcPromises.delete(message.rpc_id);
  }

  private handleDisconnect(port: browser.runtime.Port): void {
    debug("port disconnected", port);
    this.close();

    if (port.error) {
      browser.browserAction.setBadgeText({ text: "err" });
      browser.browserAction.setTitle({
        title: "Cannot connect to the native application, trying again in 60s",
      });
    }

    debug("retrying port connection in 60s");
    setTimeout(() => {
      this.init();
    }, 60 * 1000);
  }
}