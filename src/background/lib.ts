import { debug } from "./debug";
import { Management } from "./management";
import { SeshatPort } from "./ports/seshat";

export class Background {
  public manifest = browser.runtime.getManifest();
  public version = this.manifest.version;
  public browserType = this.manifest.applications?.gecko ? "firefox" : "chrome";
  public management = new Management();
  public seshat = new SeshatPort();

  private initialized = false;
  private bundleResourceURL = browser.runtime.getURL("resources/bundle.js");
  private riots: any[] = [];
  private riotTabs: Set<number> = new Set();

  constructor() {
    browser.storage.local.set({ version: this.version });
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
            message.content.eventStore = `web-${encodeURIComponent(
              `${url.origin}${url.pathname}`
            )}-${cookieStore}`;

            return this.seshat.handleRuntimeMessage(message.content);
        }
      }
    );
  }

  async initialize(): Promise<void> {
    const initializedPromise = new Promise(async (resolve: any) => {
      try {
        this.riots = await this.getStoredRiots();
      } catch (error) {
        debug("initializing failed", error);
      }
      this.initialized = true;
      resolve();
    });

    browser.webRequest.onBeforeRequest.addListener(
      async (details: any): Promise<browser.webRequest.BlockingResponse> => {
        if (!this.initialized) {
          debug("incoming request, waiting for initialization", details);
          await initializedPromise;
        }

        if (
          this.browserType === "firefox" &&
          details.url.includes("/config.json?cachebuster=")
        ) {
          debug("incoming config request", details);
          return this.riotConfigListener(details);
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
  }

  async getStoredRiots(): Promise<any> {
    const { riots } = await browser.storage.local.get("riots");
    return riots || [];
  }

  async riotConfigListener(details: {
    requestId: string;
  }): Promise<browser.webRequest.BlockingResponse> {
    const filter = browser.webRequest.filterResponseData(
      details.requestId
    ) as any;
    const decoder = new TextDecoder("utf-8");
    const encoder = new TextEncoder();

    const data: any[] = [];
    filter.ondata = (event: any): void => {
      data.push(event.data);
    };

    filter.onstop = (): void => {
      let configStr = "";
      if (data.length == 1) {
        configStr = decoder.decode(data[0]);
      } else {
        for (let i = 0; i < data.length; i++) {
          const stream = i == data.length - 1 ? false : true;
          configStr += decoder.decode(data[i], { stream });
        }
      }

      try {
        const config = JSON.parse(configStr);
        if (!config.features) {
          config.features = {};
        }
        if (!config.features.feature_event_indexing) {
          // eslint-disable-next-line @typescript-eslint/camelcase
          config.features.feature_event_indexing = "enable";
          configStr = JSON.stringify(config, null, 2);
          debug("added indexing feature to config.json");
        }
      } catch (error) {
        // no-op
      }

      filter.write(encoder.encode(configStr));
      filter.close();
    };

    return {};
  }

  async riotBundleListener(details: {
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

    browser.browserAction.setBadgeText({
      tabId: details.tabId,
      text: "on",
    });
    this.riotTabs.add(details.tabId);

    return {
      redirectUrl: this.bundleResourceURL,
    };
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
        return this.seshat.handleRuntimeMessage(message);
    }
  }

  async onBrowserActionClick(tab: browser.tabs.Tab): Promise<void> {
    debug("browser action clicked", tab);
    const url = new URL(tab.url!);
    const riot = {
      protocol: url.protocol,
      hostname: url.hostname,
      pathname: url.pathname,
      bundleURL: `${url.protocol}//${url.hostname}${url.pathname}bundles/*/bundle.js`,
      cookieStoreId: tab.cookieStoreId || false,
    };
    const pattern = `${riot.protocol}//${riot.hostname}${riot.pathname}*`;

    if (!this.riotTabs.has(tab.id!)) {
      const granted = await browser.permissions.request({
        origins: [pattern],
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
        origins: [pattern],
      });
      browser.browserAction.setBadgeText({
        tabId: tab.id!,
        text: null,
      });
    }

    await browser.storage.local.set({ riots: this.riots });
    browser.tabs.reload(tab.id);
  }
}
