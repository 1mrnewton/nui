#!/usr/bin/env bash
# nui Phase 0 runner: builds the SwiftUI app, boots a simulator, installs &
# launches it. Start the logic separately:  python3 logic/counter.py
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SIM_NAME="${SIM_NAME:-iPhone 17}"
SCHEME="NuiCounter"
BUNDLE_ID="dev.nui.counter"
DERIVED="$ROOT/ios/.build"

echo "==> Generating Xcode project"
( cd "$ROOT/ios" && xcodegen generate )

echo "==> Booting simulator: $SIM_NAME"
xcrun simctl boot "$SIM_NAME" 2>/dev/null || true
open -a Simulator

echo "==> Building for simulator"
xcodebuild \
  -project "$ROOT/ios/NuiCounter.xcodeproj" \
  -scheme "$SCHEME" \
  -configuration Debug \
  -destination "platform=iOS Simulator,name=$SIM_NAME" \
  -derivedDataPath "$DERIVED" \
  build | tail -5

APP_PATH="$DERIVED/Build/Products/Debug-iphonesimulator/NuiCounter.app"

echo "==> Installing & launching"
xcrun simctl install "$SIM_NAME" "$APP_PATH"
xcrun simctl launch "$SIM_NAME" "$BUNDLE_ID"

echo "==> Done. If the badge says 'connecting…', start the logic:"
echo "    python3 logic/counter.py"
