import { debug } from "./debug";
import { Background } from "./lib";

export class NativePort {
  private name =
    process.env.NODE_ENV === "development"
      ? "radical.native.dev"
      : "radical.native";
  private port?: browser.runtime.Port;
  private rpcPromises: Map<number, any> = new Map();
  private ready = false;
  private bg: Background;

  constructor(bg: Background) {
    this.bg = bg;
    this.init();
  }

  async handleRuntimeMessage(
    runtimeMessage: any,
    sender: browser.runtime.MessageSender
  ): Promise<any> {
    debug("runtime message received", runtimeMessage);
    if (!this.ready) {
      debug("port not ready, waiting 5s");
      // port not ready yet, give it 5s to change its mind
      await new Promise((resolve) => setTimeout(resolve, 5 * 1000));

      if (!this.ready) {
        debug("port not reachable, probably not installed");
        return null;
      }
    }

    // construct native message from runtime message
    const message = runtimeMessage.content;
    message.type = runtimeMessage.type;
    message.rpcId = runtimeMessage.rpcId;

    switch (message.type) {
      case "seshat":
        if (message.method === "supportsEventIndexing") {
          return true;
        }

        const url = new URL(sender.url!);
        message.eventStore = `web-${this.bg.uuid}-${encodeURIComponent(
          `${url.origin}${url.pathname}`
        )}-${
          this.bg.browserType === "firefox"
            ? // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
              sender.tab!.cookieStoreId!
            : "default"
        }`;
        break;
    }

    return this.postMessage(message);
  }

  private init(): void {
    this.port = browser.runtime.connectNative(this.name);
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
      // eslint-disable-next-line @typescript-eslint/camelcase
      this.rpcPromises.set(message.rpcId, {
        message,
        resolve,
        reject,
      });
      debug(`posting to ${this.name}`, message);
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
      if (!this.ready) {
        this.init();
      }
    }, 60 * 1000);
  }
}
