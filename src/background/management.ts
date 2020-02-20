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

  private handleRiotWebExt(
    extensionInfo: browser.management.ExtensionInfo
  ): void {
    if (extensionInfo.id !== RIOT_WEBEXT_ID) {
      return;
    }
    this.sendReadyMessage();
  }

  private sendReadyMessage(): void {
    browser.runtime.sendMessage(RIOT_WEBEXT_ID, {
      method: "ready",
    });
  }
}
