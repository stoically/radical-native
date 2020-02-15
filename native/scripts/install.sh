#!/bin/bash

local_bin=0
release_bin=0

while getopts "lr" opt; do
    case "$opt" in
    l)  local_bin=1
        ;;
    r)  release_bin=1
        ;;
    esac
done

case "$OSTYPE" in
  linux*)   echo "OS: Linux"
            if [ -z "$XDG_DATA_HOME" ]; then
              DATA_HOME="$HOME/.local/share"
            else
              DATA_HOME="$XDG_DATA_HOME"
            fi
            RELEASE_BIN_NAME="radical-native_linux-x86_64"
            ;;
  darwin*)  echo "OS: OSX"
            DATA_HOME="$HOME/Library/Application Support"
            RELEASE_BIN_NAME="radical-native_mac"
            ;; 
  *)        echo "Unsupported OS: $OSTYPE"
            exit 1
            ;;
esac

HOST_BIN_HOME="$DATA_HOME/radical-native"
mkdir -p "$HOST_BIN_HOME"
NATIVE_MANIFEST_NAME="radical.native"
NATIVE_MANIFEST_FILENAME="$NATIVE_MANIFEST_NAME.json"

echo "Installing radical-native"
if [ $local_bin = 1 ]; then
  if [ $release_bin = 0 ]; then
    NATIVE_HOST_APP_BIN="$PWD/target/debug/radical-native"
  else
    NATIVE_HOST_APP_BIN="$PWD/target/release/radical-native"
  fi
  echo "Using local $NATIVE_HOST_APP_BIN"
else
  ORG="stoically"
  REPO="radical-native"
  NATIVE_HOST_APP_BIN="$HOST_BIN_HOME/$RELEASE_BIN_NAME"
  LATEST_RELEASE_VERSION=$(curl -s https://api.github.com/repos/$ORG/$REPO/releases | grep -oP -m1  '"tag_name": "\K(.*)(?=")')
  curl -L -o "$NATIVE_HOST_APP_BIN" "https://github.com/$ORG/$REPO/releases/download/$LATEST_RELEASE_VERSION/$RELEASE_BIN_NAME"
  chmod +x "$NATIVE_HOST_APP_BIN"
  echo "Installed radical-native to: $NATIVE_HOST_APP_BIN"
fi

install() {
  if [ "$1" = "firefox" ]; then
    ALLOWED='"allowed_extensions": [ "@radical-native" ]'
    case "$OSTYPE" in
      linux*)   NATIVE_HOSTS_PATH="$HOME/.mozilla/native-messaging-hosts" ;;
      darwin*)  NATIVE_HOSTS_PATH="$HOME/Library/Application Support/Mozilla/NativeMessagingHosts" ;; 
    esac
  fi
  if [ "$1" = "chromium" ]; then
    ALLOWED='"allowed_origins": [ "chrome-extension://hdikcfhaiboiiihkfmgaldafdbplnjok/" ]'
    case "$OSTYPE" in
      linux*)   NATIVE_HOSTS_PATH="$HOME/.config/chromium/NativeMessagingHosts" ;;
      darwin*)  NATIVE_HOSTS_PATH="$HOME/Library/Application Support/Chromium/NativeMessagingHosts" ;; 
    esac
  fi
  if [ "$1" = "chrome" ]; then
    ALLOWED='"allowed_origins": [ "chrome-extension://hdikcfhaiboiiihkfmgaldafdbplnjok/" ]'
    case "$OSTYPE" in
      linux*)   NATIVE_HOSTS_PATH="$HOME/.config/google-chrome/NativeMessagingHosts" ;;
      darwin*)  NATIVE_HOSTS_PATH="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts" ;; 
    esac
  fi

  NATIVE_MANIFEST=$(cat <<-END
  {
    "name": "$NATIVE_MANIFEST_NAME",
    "description": "Radical Native",
    "path": "$NATIVE_HOST_APP_BIN",
    "type": "stdio",
    $ALLOWED
  }
END
  )

  NATIVE_MANIFEST_PATH="$NATIVE_HOSTS_PATH/$NATIVE_MANIFEST_FILENAME"
  mkdir -p "$NATIVE_HOSTS_PATH"
  echo "$NATIVE_MANIFEST" > "$NATIVE_MANIFEST_PATH"
  echo "$1: Installed native messaging manifest to: $NATIVE_MANIFEST_PATH"
}

install "firefox"
#install "chrome"
#install "chromium"