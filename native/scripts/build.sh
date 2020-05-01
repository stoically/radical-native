#!/bin/bash
# based on https://github.com/vector-im/riot-desktop/tree/master/hak/matrix-seshat

set -e

NASM_VERSION="2.14.02"
OPENSSL_VERSION="1.1.1f"
SQLCIPHER_VERSION="4.3.0"

mkdir -p build/native/opt
OPT_DIR=$(realpath build/native/opt)
# https://stackoverflow.com/a/13701495
OPT_WIN_DIR=$(echo "${OPT_DIR}" | sed -e 's/^\///' -e 's/\//\\/g' -e 's/^./\0:/')
cd build/native || exit 1

fetch_openssl() {
    [ -d "openssl-${OPENSSL_VERSION}" ] && return

    curl -L -o openssl.tgz "https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz"
    tar xzf openssl.tgz
}

fetch_sqlcipher() {
    [ -d "sqlcipher-${SQLCIPHER_VERSION}" ] && return

    curl -L -o sqlcipher.tgz "https://github.com/sqlcipher/sqlcipher/archive/v${SQLCIPHER_VERSION}.tar.gz"
    tar xzf sqlcipher.tgz
}

linux() {
    mkdir -p build/native
    cd build/native/linux || exit 1
    fetch_sqlcipher
}

win_install_nasm() {
    if [ ! -d "nasm-${NASM_VERSION}" ]; then
        curl -L -o nasm.zip "https://www.nasm.us/pub/nasm/releasebuilds/${NASM_VERSION}/win64/nasm-${NASM_VERSION}-win64.zip"
        unzip nasm.zip
    fi

    PATH="$PATH:$(realpath nasm-${NASM_VERSION})"
    export PATH
}

win_vcvarsall() {
    vcvarsallbat=$(find /c/Program\ Files\ \(x86\)/Microsoft\ Visual\ Studio/ -name vcvarsall.bat)
    while read -r line; do
        export "${line?}" || continue
    done < <(cmd.exe //q //c "$vcvarsallbat" x86_amd64 \> nul \&\& env | dos2unix)
}

win_build_openssl() {
    cd openssl-${OPENSSL_VERSION} || exit 1

    /c/Strawberry/perl/bin/perl.exe Configure --prefix="${OPT_DIR}" \
        no-afalgeng no-capieng no-cms no-ct no-deprecated no-dgram \
        no-dso no-ec no-ec2m no-gost no-nextprotoneg no-ocsp no-sock \
        no-srp no-srtp no-tests no-ssl no-tls no-dtls no-shared no-aria \
        no-camellia no-cast no-chacha no-cmac no-des no-dh no-dsa no-ecdh \
        no-ecdsa no-idea no-md4 no-mdc2 no-ocb no-poly1305 no-rc2 no-rc4 \
        no-rmd160 no-scrypt no-seed no-siphash no-sm2 no-sm3 no-sm4 no-whirlpool \
        VC-WIN64A

    nmake build_libs
    nmake install_dev

    cd ..
}

win_build_sqlcipher() {
    cd sqlcipher-${SQLCIPHER_VERSION} || exit 1

    CCOPTS="-DSQLITE_HAS_CODEC -I ${OPT_WIN_DIR}\include" \
    LTLIBPATHS="/LIBPATH:${OPT_WIN_DIR}\lib" \
    LTLIBS="libcrypto.lib" \
    nmake //f Makefile.msc libsqlite3.lib

    cp libsqlite3.lib "${OPT_DIR}/lib/sqlcipher.lib"
    cp sqlite3.h "${OPT_DIR}/include/sqlcipher.h"

    cd ..
}

# tested with
# - Git for Windows bash (MINGW64)
# - Visual Studio Build Tools 2019
#   - C++ build tools
#     + MSVC VS 2019
#     + Windows 10 SDK
#     + C++ CMake tools
win() {
    fetch_openssl
    fetch_sqlcipher
    win_vcvarsall
    win_install_nasm
    win_build_openssl
    win_build_sqlcipher

    SQLCIPHER_STATIC="1"
    SQLCIPHER_LIB_DIR="${OPT_WIN_DIR}\lib"
    SQLCIPHER_INCLUDE_DIR="${OPT_WIN_DIR}\include"
    export SQLCIPHER_STATIC SQLCIPHER_LIB_DIR SQLCIPHER_INCLUDE_DIR

    #CARGO_TARGET_DIR="$HOME/target" \
    RUSTFLAGS="-Ctarget-feature=+crt-static -Clink-args=libcrypto.lib -L${OPT_WIN_DIR}\lib" \
    RUSTUP_TOOLCHAIN="stable-x86_64-pc-windows-msvc" \
    cargo build --release

    ls -al ../../target/release
}

case "$OSTYPE" in
  linux*)  linux ;;
  darwin)  macos ;; 
  msys)    win ;;
  *)       echo "Unsupported OS: $OSTYPE"
           exit 1
           ;;
esac
