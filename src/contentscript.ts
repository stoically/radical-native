/* eslint-disable @typescript-eslint/no-explicit-any */

let bundleURL: string;

const handleFromBundleMessage = (message: any): void => {
  switch (message.method) {
    case "ready":
      window.postMessage(
        {
          type: "bundle",
          target: "page",
          method: "init",
          bundle: bundleURL,
        },
        "*"
      );
      break;
  }
};

const handleSeshatMessage = async (message: any): Promise<void> => {
  const reply = await browser.runtime.sendMessage(message);
  const rpcReply = {
    type: "seshat",
    target: "page",
    method: "rpc",
    rpcId: message.rpcId,
    reply,
  };
  window.postMessage(rpcReply, "*");
};

window.addEventListener("message", function(event) {
  if (event.source !== window || event?.data?.target !== "contentscript") {
    console.log("[RadicalNative::contentscript] ignoring message", event);
    return;
  }
  console.log("[RadicalNative::contentscript] window message", event);

  switch (event.data.type) {
    case "bundle":
      handleFromBundleMessage(event.data);
      break;

    case "seshat":
      handleSeshatMessage(event.data);
      break;
  }
});

browser.runtime.onMessage.addListener(
  async (message: any): Promise<any> => {
    console.log("[RadicalNative::contentscript] runtime.onMessage", message);

    if (message.method === "ready") {
      bundleURL = message.bundle;
      return "ready";
    }
  }
);
