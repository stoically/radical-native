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

import { SeshatPort } from "./ports/seshat";
import { debug } from "./debug";

export class Background {
  private seshatPort: SeshatPort;
  constructor() {
    this.seshatPort = new SeshatPort();

    browser.runtime.onMessageExternal.addListener(
      this.handleExternalMessage.bind(this)
    );

    browser.runtime.onInstalled.addListener(
      ({ temporary }: { temporary: boolean }) => {
        if (temporary) {
          window.DEBUG = true;
        }
      }
    );
  }

  handleExternalMessage(
    message: any,
    sender: browser.runtime.MessageSender
  ): any {
    debug("external message received", message, sender);
    if (sender.id !== "@riot-webext") {
      throw new Error("Access denied");
    }

    switch (message.type) {
      case "seshat":
        return this.seshatPort.handleExternalMessage(message);
    }
  }
}
