import { debug } from "./debug";
import { Management } from "./management";
import { SeshatPort } from "./ports/seshat";

export class Background {
  public manifest = browser.runtime.getManifest();
  public version = this.manifest.version;
  public browserType = this.manifest.applications?.gecko ? "firefox" : "chrome";
  public management = new Management();
  public seshat = new SeshatPort();

  constructor() {
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
        return this.seshat.handleExternalMessage(message);
    }
  }
}
