# Riot Booster Pack

Pushing Riot Web beyond the limits of the web platform

- [x] Search functionality in encrypted rooms using [seshat](https://github.com/matrix-org/seshat)
- [ ] OS global keyboard shortcuts (e.g. push to talk)
- [ ] Secure OS key storage (similar to e.g. [keytar](https://www.npmjs.com/package/keytar))
- [ ] Tray icon

Supported platforms

- [x] Linux
- [x] MacOS
- [ ] Windows

## Install

### 1. SQLCipher

- Ubuntu/Debian: `apt install libsqlcipher0`
- MacOS: `brew install sqlcipher`

SQLCipher is needed so that the search index can be encrypted on disk.

### 2. Native Booster Pack Binary

```
curl -LsSf https://git.io/JvWXo | bash
```

This one-liner is a [simple shell script](https://github.com/stoically/riot-web-booster-pack/blob/master/native/scripts/install.sh) that downloads the [native booster pack binary from the releases](https://github.com/stoically/riot-web-booster-pack/releases), stores it, and generates a [native manifest](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests#Manifest_location) pointing to the binary for Firefox, Chrome and Chromium.

### 3. WebExtension

- [Install the Firefox Add-on Beta from Releases](https://github.com/stoically/riot-web-booster-pack/releases)

The WebExtension facilitates the communication between Riot Web and the Native Booster Pack Binary.

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
