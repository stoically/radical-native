/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/camelcase */
/* eslint-disable @typescript-eslint/explicit-function-return-type */

class SeshatIndexManager {
  private rpcId = 0;
  private rpcPromises: Map<number, any> = new Map();

  handleMessage(message: any) {
    switch (message.method) {
      case "rpc":
        const rpcPromise = this.rpcPromises.get(message.rpcId);
        if (!rpcPromise) {
          console.log(
            "[RadicalNative::page] message received without matching rpcPromise",
            message
          );
          return;
        }
        console.log("[RadicalNative::page] rpc reply received", message, {
          originalMessage: rpcPromise.message,
        });
        rpcPromise.resolve(message.reply);
        this.rpcPromises.delete(message.rpcId);
        break;
    }
  }

  postMessage(message: any) {
    return new Promise((resolve, reject) => {
      this.rpcId++;
      const rpcMessage = {
        type: "seshat",
        target: "contentscript",
        rpcId: this.rpcId,
        content: message,
      };
      this.rpcPromises.set(this.rpcId, {
        message: rpcMessage,
        resolve,
        reject,
      });
      console.log("[RadicalNative::page] posting rpc message", rpcMessage);
      window.postMessage(rpcMessage, "*");
    });
  }

  async supportsEventIndexing() {
    return this.postMessage({ method: "supportsEventIndexing" });
  }

  async initEventIndex() {
    return this.postMessage({ method: "initEventIndex" });
  }

  async addEventToIndex(ev: any, profile: any) {
    return this.postMessage({
      method: "addEventToIndex",
      content: { ev, profile },
    });
  }

  async isEventIndexEmpty() {
    return this.postMessage({ method: "isEventIndexEmpty" });
  }

  async commitLiveEvents() {
    return this.postMessage({ method: "commitLiveEvents" });
  }

  async searchEventIndex(searchConfig: any) {
    return this.postMessage({
      method: "searchEventIndex",
      content: { searchConfig },
    });
  }

  async addHistoricEvents(events: any, checkpoint: any, oldCheckpoint: any) {
    return this.postMessage({
      method: "addHistoricEvents",
      content: {
        events,
        checkpoint,
        oldCheckpoint,
      },
    });
  }

  async addCrawlerCheckpoint(checkpoint: any) {
    return this.postMessage({
      method: "addCrawlerCheckpoint",
      content: { checkpoint },
    });
  }

  async removeCrawlerCheckpoint(checkpoint: any) {
    return this.postMessage({
      method: "removeCrawlerCheckpoint",
      content: { checkpoint },
    });
  }

  async loadFileEvents(args: any) {
    return this.postMessage({ method: "loadFileEvents", content: { args } });
  }

  async loadCheckpoints() {
    return this.postMessage({ method: "loadCheckpoints" });
  }

  async closeEventIndex() {
    return this.postMessage({ method: "closeEventIndex" });
  }

  async getStats() {
    return this.postMessage({ method: "getStats" });
  }

  async deleteEventIndex() {
    return this.postMessage({ method: "deleteEventIndex" });
  }

  async deleteEvent(eventId: any) {
    return this.postMessage({ method: "deleteEvent", content: { eventId } });
  }
}

const indexManager = new SeshatIndexManager();

class PlatformPeg {
  private platform: any = null;

  get() {
    return this.platform;
  }

  set(plaf: any) {
    this.platform = plaf;
    this.platform.getEventIndexingManager = () => indexManager;
  }
}

interface Window {
  mxPlatformPeg: PlatformPeg;
}

window.mxPlatformPeg = new PlatformPeg();

const handleToBundleMessage = (message: any) => {
  switch (message.method) {
    case "init":
      const bundle = document.createElement("script");
      bundle.src = message.bundle;
      bundle.async = true;
      document.body.append(bundle);
      break;
  }
};

window.addEventListener("message", function(event) {
  if (event.source !== window || event?.data?.target !== "page") {
    console.log("[RadicalNative::page] ignoring message", event);
    return;
  }
  console.log("[RadicalNative::page] message received", event);

  switch (event.data.type) {
    case "bundle":
      handleToBundleMessage(event.data);
      break;

    case "seshat":
      indexManager.handleMessage(event.data);
      break;
  }
});

window.postMessage(
  {
    type: "bundle",
    method: "ready",
    target: "contentscript",
  },
  "*"
);
