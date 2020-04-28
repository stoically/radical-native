# Radical Native

[![Radical Native Matrix room #radical-webext:matrix.org](https://img.shields.io/badge/matrix-%23radical--webext%3Amatrix.org-blue)](https://matrix.to/#/#radical-webext:matrix.org)

Extending [Riot Web](https://github.com/vector-im/riot-web) with native capabilities

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

- [x] Riot Web over HTTP
- [x] [Radical](https://github.com/stoically/radical): Riot Web bundled as Firefox Add-on

## Install

### 1. Radical Native Binary

- Ubuntu/Debian: [Install the binary from the latest released `.deb` file](https://github.com/stoically/radical-native/releases)

Hint: The event store is saved into the `radical-native` directory inside your [user data directory](https://github.com/soc/dirs-rs#features).

### 2. Radical Native Add-on

- [Install the Firefox Add-on from the latest released `.xpi` file](https://github.com/stoically/radical-native/releases)

The Radical Native Firefox Add-on facilitates the communication between Riot Web and the Radical Native Binary.

## Usage

### Riot Web over HTTP

- Open any Riot website in your browser
- Click the Radical Native icon in the toolbar (RAM icon)
- Riot website should reload and icon should have an "on" badge
- Check Riot's "Settings > Security & Privacy > Message search > Manage", it should show ongoing work

### Radical Add-on

- See https://github.com/stoically/radical#search

## Troubleshooting

- Try to execute the `radical-native` binary directly - it should respond with "ready: true"
- Check the Radical Native Add-on console for error logs: `about:debugging#/runtime/this-firefox` > Radical Native Inspect
- If indexing gets stuck you can safely disable and enable it in the "Manage" dialog

## Development

- `cargo install cargo-watch`
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
