#!/bin/bash
# based on https://github.com/vector-im/riot-desktop/tree/master/hak/matrix-seshat

set -e

NASM_VERSION="2.14.02"
OPENSSL_VERSION="1.1.1f"
SQLCIPHER_VERSION="4.3.0"

mkdir -p target/release
mkdir -p build/native/opt

OPT_DIR="$PWD/build/native/opt"
export OPT_DIR

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

native_manifest() {
    cat <<-END
{
    "name": "radical.native",
    "description": "Radical Native",
    "path": "$1",
    "type": "stdio",
    "allowed_extensions": [ "@radical-native" ]
}
END
}

linux() {
    native_manifest "/usr/bin/radical-native" > target/release/radical.native.json
    cargo test
    cargo deb -p radical-native --output target/release/radical-native.deb
}

macos_build_sqlcipher() {
    cd sqlcipher-${SQLCIPHER_VERSION} || exit 1

    CFLAGS=-DSQLITE_HAS_CODEC \
    LDFLAGS="-framework Security -framework Foundation" \
    ./configure \
        --prefix="${OPT_DIR}" \
        --enable-tempstore=yes \
        --enable-shared=no \
        --with-crypto-lib=commoncrypto
    
    make
    make install
    
    cd ..
}

macos() {
    pushd build/native || exit 1
    fetch_sqlcipher
    macos_build_sqlcipher
    popd

    RUSTFLAGS="-L${OPT_DIR}/lib"
    export RUSTFLAGS

    cargo test
    cargo build --release

    NATIVE_RES_PATH="./native/res/darwin"
    PKG_PATH="./build/native/pkg"
    PKG_ROOT_PATH="${PKG_PATH}/root"
    PKG_INST_PATH="${PKG_ROOT_PATH}/Library/RadicalNative"
    PKG_PACKAGE_PATH="${PKG_PATH}/package"
    rm -rf "${PKG_PATH}"
    mkdir -p "${PKG_PATH}" "${PKG_INST_PATH}" "${PKG_PACKAGE_PATH}"

    native_manifest "/Library/RadicalNative/radical-native" > "${PKG_INST_PATH}/radical.native.json"
    cp ./target/release/radical-native "${PKG_INST_PATH}"

    pkgbuild --identifier io.github.stoically.radical-native \
      --version 0.1.0 \
      --scripts "${NATIVE_RES_PATH}/scripts" \
      --root "${PKG_ROOT_PATH}" \
      "${PKG_PACKAGE_PATH}/radical-native.pkg"

    productbuild --distribution "${NATIVE_RES_PATH}/Distribution" \
         --resources "${NATIVE_RES_PATH}/Resources" \
        --package-path "${PKG_PACKAGE_PATH}" \
        ./target/release/radical-native.pkg
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

# locally tested with
# - Windows 10 Pro VM on x86_amd64
# - Git for Windows bash (MINGW64)
# - Visual Studio Build Tools 2019
#   + C++ build tools
#     + MSVC VS 2019
#     + Windows 10 SDK
#     + C++ CMake tools
# - NSIS 3
win() {
    # https://stackoverflow.com/a/13701495
    OPT_WIN_DIR=$(echo "${OPT_DIR}" | sed -e 's/^\///' -e 's/\//\\/g' -e 's/^./\0:/')
    export OPT_WIN_DIR

    pushd build/native || exit 1
    fetch_openssl
    fetch_sqlcipher
    win_vcvarsall
    win_install_nasm
    win_build_openssl
    win_build_sqlcipher
    popd

    SQLCIPHER_STATIC="1"
    SQLCIPHER_LIB_DIR="${OPT_WIN_DIR}\lib"
    SQLCIPHER_INCLUDE_DIR="${OPT_WIN_DIR}\include"
    export SQLCIPHER_STATIC SQLCIPHER_LIB_DIR SQLCIPHER_INCLUDE_DIR

    RUSTFLAGS="-Ctarget-feature=+crt-static -Clink-args=libcrypto.lib -L${OPT_WIN_DIR}\lib"
    RUSTUP_TOOLCHAIN="stable-x86_64-pc-windows-msvc"
    export RUSTFLAGS RUSTUP_TOOLCHAIN

    cargo test
    cargo build --release

    /c/Program\ Files\ \(x86\)/NSIS/Bin/makensis.exe native/res/win.nsi
}

case "$OSTYPE" in
  linux*)  linux ;;
  darwin*)  macos ;; 
  msys)    win ;;
  *)       echo "Unsupported OS: $OSTYPE"
           exit 1
esac
