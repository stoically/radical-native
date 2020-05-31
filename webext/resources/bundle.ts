/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/camelcase */
/* eslint-disable @typescript-eslint/explicit-function-return-type */

let rpcId = 0;
const rpcPromises: Map<number, any> = new Map();

function handleMessage(message: any) {
  switch (message.method) {
    case "rpc":
      const rpcPromise = rpcPromises.get(message.rpcId);
      if (!rpcPromise) {
        console.log(
          "[RadicalNative::page] message received without matching rpcPromise",
          message
        );
        return;
      }
      rpcPromise.resolve(message.reply);
      rpcPromises.delete(message.rpcId);
      break;
  }
}

function postMessage(type: string, message: any): Promise<any> {
  return new Promise((resolve, reject) => {
    rpcId++;
    const rpcMessage = {
      type,
      target: "contentscript",
      rpcId,
      content: message,
    };
    rpcPromises.set(rpcId, {
      message: rpcMessage,
      resolve,
      reject,
    });
    window.postMessage(rpcMessage, "*");
  });
}

class SeshatIndexManager {
  async supportsEventIndexing() {
    return postMessage("seshat", { method: "supportsEventIndexing" });
  }

  async initEventIndex() {
    return postMessage("seshat", { method: "initEventIndex" });
  }

  async addEventToIndex(ev: any, profile: any) {
    return postMessage("seshat", {
      method: "addEventToIndex",
      content: { ev, profile },
    });
  }

  async isEventIndexEmpty() {
    return postMessage("seshat", { method: "isEventIndexEmpty" });
  }

  async commitLiveEvents() {
    return postMessage("seshat", { method: "commitLiveEvents" });
  }

  async searchEventIndex(config: any) {
    const term = config.search_term;
    delete config.search_term;

    return postMessage("seshat", {
      method: "searchEventIndex",
      content: { term, config },
    });
  }

  async addHistoricEvents(events: any, checkpoint: any, oldCheckpoint: any) {
    return postMessage("seshat", {
      method: "addHistoricEvents",
      content: {
        events,
        checkpoint,
        oldCheckpoint,
      },
    });
  }

  async addCrawlerCheckpoint(checkpoint: any) {
    return postMessage("seshat", {
      method: "addCrawlerCheckpoint",
      content: { checkpoint },
    });
  }

  async removeCrawlerCheckpoint(oldCheckpoint: any) {
    return postMessage("seshat", {
      method: "removeCrawlerCheckpoint",
      content: { oldCheckpoint },
    });
  }

  async loadFileEvents(args: any) {
    return postMessage("seshat", {
      method: "loadFileEvents",
      content: { ...args },
    });
  }

  async loadCheckpoints() {
    return postMessage("seshat", { method: "loadCheckpoints" });
  }

  async closeEventIndex() {
    return postMessage("seshat", { method: "closeEventIndex" });
  }

  async getStats() {
    return postMessage("seshat", { method: "getStats" });
  }

  async deleteEventIndex() {
    return postMessage("seshat", { method: "deleteEventIndex" });
  }

  async deleteEvent(eventId: any) {
    return postMessage("seshat", {
      method: "deleteEvent",
      content: { eventId },
    });
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

window.addEventListener("message", function (event) {
  if (event.source !== window || event?.data?.target !== "page") {
    return;
  }

  switch (event.data.type) {
    case "bundle":
      handleToBundleMessage(event.data);
      break;

    default:
      handleMessage(event.data);
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
