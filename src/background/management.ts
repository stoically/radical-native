// Copyright 2020 stoically@protonmail.com
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

const RIOT_WEBEXT_ID = "@riot-webext";

export class Management {
  constructor() {
    browser.management.onInstalled.addListener(
      this.handleRiotWebExt.bind(this)
    );
    browser.management.onEnabled.addListener(this.handleRiotWebExt.bind(this));

    browser.management
      .get(RIOT_WEBEXT_ID)
      .then((extensionInfo: browser.management.ExtensionInfo) => {
        if (extensionInfo.enabled) {
          this.sendReadyMessage();
        }
      })
      .catch(() => {
        // noop
      });
  }

  handleRiotWebExt(extensionInfo: browser.management.ExtensionInfo): void {
    if (extensionInfo.id !== RIOT_WEBEXT_ID) {
      return;
    }
    this.sendReadyMessage();
  }

  sendReadyMessage(): void {
    browser.runtime.sendMessage(RIOT_WEBEXT_ID, {
      method: "ready",
    });
  }
}
