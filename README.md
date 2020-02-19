# Radical Native

Extending Riot Web with native messaging capabilities

- Matrix Room: [#radical-webext:matrix.org](https://matrix.to/#/#radical-webext:matrix.org)

#### Features

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
- [x] Riot Web over HTTP

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

## Usage

### Riot Web over HTTP

- Open any Riot website in your browser
- Click the Radical Native icon in the toolbar (RAM icon)
- Riot website should reload and icon should have an "on" badge
- Check Riot's "Settings > Security & Privacy > Message search > Manage", it should show [ongoing work](https://github.com/vector-im/riot-web/issues/12334)

### Radical Add-on

- See https://github.com/stoically/radical#search

## Troubleshooting

- Check the console output from install step 2 and try to execute the radical native binary directly (the path mentioned after "Installed to:") - it should respond with "ready: true"
- Check the Radical Native console for error logs: `about:debugging#/runtime/this-firefox` > Radical Native Inspect
- If indexing gets stuck you can safely disable and enable it in the "Manage" dialog

## Development

- Ubuntu/Debian: `apt install libsqlcipher0 libsqlcipher-dev`
- MacOS: `brew install libsqlcipher`

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

## Attribution

Icon made by [Freepik](https://www.flaticon.com/authors/freepik) from [www.flaticon.com](https://www.flaticon.com/)
