/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/explicit-function-return-type */

let rpcId = 0;
const rpcPromises: Map<number, any> = new Map();

function rpcHandleMessage(message: any) {
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

      if (!message.error) {
        rpcPromise.resolve(message.reply);
      } else {
        rpcPromise.reject(message.error);
      }
      rpcPromises.delete(message.rpcId);
      break;
  }
}

function rpcPostMessage(type: string, message: any): Promise<any> {
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
  supportsEventIndexing() {
    return rpcPostMessage("seshat", { method: "supportsEventIndexing" });
  }

  initEventIndex() {
    return rpcPostMessage("seshat", { method: "initEventIndex" });
  }

  addEventToIndex(ev: any, profile: any) {
    return rpcPostMessage("seshat", {
      method: "addEventToIndex",
      content: { ev, profile },
    });
  }

  isEventIndexEmpty() {
    return rpcPostMessage("seshat", { method: "isEventIndexEmpty" });
  }

  isRoomIndexed(roomId: string): Promise<boolean> {
    return rpcPostMessage("seshat", {
      method: "isRoomIndexed",
      content: { roomId },
    });
  }

  commitLiveEvents() {
    return rpcPostMessage("seshat", { method: "commitLiveEvents" });
  }

  searchEventIndex(config: any) {
    const term = config.search_term;

    return rpcPostMessage("seshat", {
      method: "searchEventIndex",
      content: { term, config },
    });
  }

  addHistoricEvents(events: any, checkpoint: any, oldCheckpoint: any) {
    return rpcPostMessage("seshat", {
      method: "addHistoricEvents",
      content: {
        events,
        checkpoint,
        oldCheckpoint,
      },
    });
  }

  addCrawlerCheckpoint(checkpoint: any) {
    return rpcPostMessage("seshat", {
      method: "addCrawlerCheckpoint",
      content: { checkpoint },
    });
  }

  removeCrawlerCheckpoint(oldCheckpoint: any) {
    return rpcPostMessage("seshat", {
      method: "removeCrawlerCheckpoint",
      content: { oldCheckpoint },
    });
  }

  loadFileEvents(args: any) {
    return rpcPostMessage("seshat", {
      method: "loadFileEvents",
      content: { ...args },
    });
  }

  loadCheckpoints() {
    return rpcPostMessage("seshat", { method: "loadCheckpoints" });
  }

  closeEventIndex() {
    return rpcPostMessage("seshat", { method: "closeEventIndex" });
  }

  getStats() {
    return rpcPostMessage("seshat", { method: "getStats" });
  }

  getUserVersion() {
    return rpcPostMessage("seshat", { method: "getUserVersion" });
  }

  setUserVersion(version: number): Promise<void> {
    return rpcPostMessage("seshat", {
      method: "setUserVersion",
      content: { version },
    });
  }

  deleteEventIndex() {
    return rpcPostMessage("seshat", { method: "deleteEventIndex" });
  }

  deleteEvent(eventId: any) {
    return rpcPostMessage("seshat", {
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

    this.platform.getPickleKey = async (
      userId: string,
      deviceId: string
    ): Promise<string | null> => {
      try {
        return await rpcPostMessage("keytar", {
          method: "getPickleKey",
          content: { userId, deviceId },
        });
      } catch (e) {
        return null;
      }
    };

    this.platform.createPickleKey = async (
      userId: string,
      deviceId: string
    ): Promise<string | null> => {
      try {
        return await rpcPostMessage("keytar", {
          method: "createPickleKey",
          content: { userId, deviceId },
        });
      } catch (e) {
        return null;
      }
    };

    this.platform.destroyPickleKey = async (
      userId: string,
      deviceId: string
    ): Promise<void> => {
      try {
        await rpcPostMessage("keytar", {
          method: "destroyPickleKey",
          content: { userId, deviceId },
        });
      } catch (e) {}
    };
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
      rpcHandleMessage(event.data);
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
