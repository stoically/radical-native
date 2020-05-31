import { v4 as uuidv4 } from "uuid";
import { debug } from "./debug";
import { NativePort } from "../../port/native";

export class Background {
  public manifest = browser.runtime.getManifest();
  public version = this.manifest.version;
  public browserType = this.manifest.applications?.gecko ? "firefox" : "chrome";
  public port = new NativePort();

  private uuid!: string;
  private initialized = false;
  private initializedPromise: Promise<void>;
  private bundleResourceURL = browser.runtime.getURL("resources/bundle.js");
  private riots: any[] = [];
  private riotTabs: Set<number> = new Set();

  constructor() {
    browser.browserAction.disable();
    browser.browserAction.setBadgeText({ text: "init" });
    browser.browserAction.setTitle({ title: "Initializing.." });

    this.initializedPromise = new Promise(this.initialize.bind(this));
    this.setupListeners();
  }

  private async initialize(resolve: any, reject: any): Promise<void> {
    try {
      const storage = await browser.storage.local.get();
      if (storage.uuid) {
        this.uuid = storage.uuid;
      } else {
        this.uuid = uuidv4();
        await browser.storage.local.set({ uuid: this.uuid });
      }

      if (!storage.version || storage.version !== this.version) {
        await browser.storage.local.set({ version: this.version });
      }

      if (storage.riots) {
        this.riots = storage.riots;
      }
    } catch (error) {
      reject(`unrecoverable storage error: ${error.toString()}`);
      throw error;
    }
    this.initialized = true;
    resolve();
  }

  private setupListeners(): void {
    browser.webRequest.onBeforeRequest.addListener(
      async (details: any): Promise<browser.webRequest.BlockingResponse> => {
        if (!this.initialized) {
          debug("incoming request, waiting for initialization", details);
          await this.initializedPromise;
        }

        if (details.url.endsWith("/bundle.js")) {
          debug("incoming bundle request", details);
          return this.riotBundleListener(details);
        }

        return {};
      },
      {
        urls: ["<all_urls>"],
        types: ["script", "xmlhttprequest"],
      },
      ["blocking"]
    );

    browser.runtime.onInstalled.addListener(
      ({ temporary }: { temporary: boolean }) => {
        if (temporary) {
          window.DEBUG = true;
        }
      }
    );

    browser.browserAction.onClicked.addListener(
      this.onBrowserActionClick.bind(this)
    );

    browser.browserAction.setBadgeBackgroundColor({
      color: "gray",
    });

    browser.runtime.onMessage.addListener(
      (message: any, sender: browser.runtime.MessageSender) => {
        debug("internal message received", message, sender);
        switch (message.type) {
          case "seshat":
            const url = new URL(sender.url!);
            const cookieStore =
              this.browserType === "firefox"
                ? // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                  sender.tab!.cookieStoreId!
                : "default";
            message.content.eventStore = `web-${this.uuid}-${encodeURIComponent(
              `${url.origin}${url.pathname}`
            )}-${cookieStore}`;

            return this.port.handleRuntimeMessage(message.content);
        }
      }
    );
  }

  private async riotBundleListener(details: {
    url: string;
    tabId: number;
  }): Promise<browser.webRequest.BlockingResponse> {
    await browser.tabs
      .executeScript(details.tabId, {
        file: "contentscript.js",
        runAt: "document_start",
      })
      .catch(() => {
        // expected because of how parcel packages the contentscript
      })
      .finally(async () => {
        return browser.tabs.sendMessage(details.tabId, {
          method: "ready",
          bundle: `${details.url}?load`,
        });
      });

    browser.browserAction.setTitle({
      title: "Radical Native active for this Riot. Click to disable.",
      tabId: details.tabId,
    });
    browser.browserAction.setBadgeText({
      tabId: details.tabId,
      text: "on",
    });
    this.riotTabs.add(details.tabId);

    // TODO: just let the original bundle load, since we injected the necessary
    // stuff already anyway
    return {
      redirectUrl: this.bundleResourceURL,
    };
  }

  private async onBrowserActionClick(tab: browser.tabs.Tab): Promise<void> {
    debug("browser action clicked", tab);
    if (!tab.url?.startsWith("http")) {
      browser.browserAction.disable(tab.id);
      browser.browserAction.setTitle({
        title: `Can't activate Radical Native for ${tab.url}`,
        tabId: tab.id,
      });
      browser.browserAction.setBadgeText({ text: "err", tabId: tab.id });
      return;
    }
    const url = new URL(tab.url!);
    const riot = {
      protocol: url.protocol,
      hostname: url.hostname,
      pathname: url.pathname,
      cookieStoreId: tab.cookieStoreId || false,
    };
    const pattern = `${riot.protocol}//${riot.hostname}${riot.pathname}*`;
    const origins = [pattern];

    if (!this.riotTabs.has(tab.id!)) {
      const granted = await browser.permissions.request({
        origins,
      });
      if (!granted) {
        return;
      }

      this.riots.push(riot);
    } else {
      this.riots = this.riots.filter((enabledRiot: any) => {
        return !(
          riot.protocol === enabledRiot.protocol &&
          riot.hostname === enabledRiot.hostname &&
          riot.pathname === enabledRiot.pathname
        );
      });
      browser.permissions.remove({
        origins,
      });
      browser.browserAction.setBadgeText({
        tabId: tab.id!,
        text: null,
      });

      this.riotTabs.delete(tab.id!);
    }

    await browser.storage.local.set({ riots: this.riots });
    browser.tabs.reload(tab.id);
  }
}
