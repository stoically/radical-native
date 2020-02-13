# Riot Booster Pack

Pushing Riot Web beyond the limits of the web platform

- [x] Search functionality in encrypted rooms using [seshat](https://github.com/matrix-org/seshat)
- [ ] OS global keyboard shortcuts (e.g. push to talk)
- [ ] Secure OS key storage (similar to e.g. [keytar](https://www.npmjs.com/package/keytar))
- [ ] Tray icon

## Install

### 1. SQLCipher

- Ubuntu/Debian: `apt install libsqlcipher0`
- MacOS: `brew install sqlcipher`

### 2. Native Helper

```
curl -sSf https://git.io/JvWXo | bash
```

### 3. WebExtension

- [Get and install the Firefox Add-on Beta from Releases](https://github.com/stoically/riot-web-booster-pack/releases)

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
