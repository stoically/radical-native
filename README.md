# Radical Native

Extending Riot Web with native messaging capabilities

- [x] Search functionality in encrypted rooms using [seshat](https://github.com/matrix-org/seshat)
- [ ] OS global keyboard shortcuts (e.g. push to talk)
- [ ] Secure OS key storage (similar to e.g. [keytar](https://www.npmjs.com/package/keytar))
- [ ] Tray icon

#### Supported Platforms

- [x] Linux
- [x] MacOS
- [ ] Windows

#### Supported Browsers

- [x] Firefox
- [ ] Chrome

#### Supported Riots

- [x] [Radical](https://github.com/stoically/radical): Riot Web bundled as Firefox Add-on
- [ ] Riot Web over HTTP

## Install

### 1. SQLCipher

- Ubuntu/Debian: `apt install libsqlcipher0`
- MacOS: `brew install sqlcipher`

SQLCipher is needed for [seshat](https://github.com/matrix-org/seshat).

### 2. Radical Native Binary

```
curl -LsSf https://git.io/JvlNt | bash
```

This one-liner is a [simple shell script](https://github.com/stoically/radical-native/blob/master/native/scripts/install.sh) that downloads the [radical native binary from the releases](https://github.com/stoically/radical-native/releases), stores it, and generates a [native manifest](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests#Manifest_location) pointing to the binary for Firefox.

Hint: The binary and event store are saved into the `radical-native` directory inside your [user data directory](https://github.com/soc/dirs-rs#features).

### 3. Radical Native Add-on

- [Install the latest Radical Native Firefox Add-on](https://github.com/stoically/radical-native/releases)

The Radical Native Firefox Add-on facilitates the communication between Riot Web and the Radical Native Binary.

### 4. Radical Add-on

1. [Install the latest Radical Firefox Add-on](https://github.com/stoically/radical/releases)
2. Configure the Riot Web config in the Radical Add-on preferences to include

  ```
    "features": {
      "feature_event_indexing": "labs"
    }
  ```

3. Open/Reload an Riot tab, Login, go to "Settings > Labs" and toggle the "Enable local event indexing and E2EE search" feature
4. Reload Riot tab, go to "Settings > Security & Privacy", there you should see a "Manage" button under "Message search", clicking it should show ongoing work (when it's done indexing it says ["Indexed rooms: 0 out of N"](https://github.com/vector-im/riot-web/issues/12334))

The Radical Firefox Add-on bundles Riot Web and provides the needed patches to make search in encrypted rooms work.

### Troubleshooting

- Check the console output from step 3 and try to execute the radical native binary directly (the path mentioned after "Installed to:") - it should respond with "ready: true"
- Check the Radical Native console for error logs: `about:debugging` > This Firefox > Radical Native Inspect
- Try restarting Firefox with both Add-ons installed and enabled
- If indexing gets stuck you can safely disable and enable it in the "Manage" dialog


## Development

```
npm install
npm run dev
```

### Firefox

- Load the build located in `build/firefox` as Temporary Add-on via
  `about:debugging#/runtime/this-firefox`

### Chrome

- Load the build located in `build/chrome` as Unpacked extension via `chrome://extensions/`


## Tests

```shell
# watcher
npm run test:watch

# once
npm run test

# once & coverage
npm run test:coverage
```
