// Copyright 2019 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { debug } from "~/background/debug";

export class SeshatPort {
  private port?: browser.runtime.Port;
  private rpcId = 0;
  private rpcPromises: Map<number, any> = new Map();
  private ready = false;

  constructor() {
    this.init();
  }

  async handleExternalMessage(message: any): Promise<any> {
    switch (message.method) {
      case "supportsEventIndexing":
        return this.ready;

      default:
        return this.postMessage(message);
    }
  }

  private init(): void {
    this.port = browser.runtime.connectNative("im.riot.booster.pack");
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
      this.port?.postMessage(message);
    });
  }

  private handleMessage(message: any): void {
    if (message.ready) {
      debug("port ready");
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
  }
}
