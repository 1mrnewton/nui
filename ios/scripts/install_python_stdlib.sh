#!/bin/bash
# Installs the embedded Python standard library into the app bundle at build time.
#
# - Simulator: plain .so modules can be dlopen-ed directly and signing isn't
#   required, so we just copy the stdlib (fast, no signing).
# - Device: iOS forbids dlopen of loose .so files, so each extension module must
#   be repackaged as an individual signed .framework (PEP 730 ".fwork" indirection).
#   BeeWare's install_python (in the xcframework) does exactly that.
set -e

XCF_REL="Frameworks/Python.xcframework"
XCF="$PROJECT_DIR/$XCF_REL"

if [ "$EFFECTIVE_PLATFORM_NAME" = "-iphonesimulator" ]; then
    DEST="$CODESIGNING_FOLDER_PATH/python/lib"
    SLICE="ios-arm64_x86_64-simulator"
    mkdir -p "$DEST"
    rsync -a --delete "$XCF/lib/" "$DEST/" --exclude 'libpython*.dylib'
    if [ -d "$XCF/$SLICE/lib-$ARCHS" ]; then
        rsync -a "$XCF/$SLICE/lib-$ARCHS/" "$DEST/" --exclude 'libpython*.dylib'
    fi
    echo "nui: installed Python stdlib (simulator, arch $ARCHS)"
else
    # Device: copy stdlib + convert .so -> signed frameworks.
    source "$XCF/build/utils.sh"
    install_python "$XCF_REL"
    echo "nui: installed + signed Python stdlib (device, arch $ARCHS)"
fi
