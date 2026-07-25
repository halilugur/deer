#!/usr/bin/env bash
set -e

echo "===================================================="
echo "🦌 DEER — Platform Ready Release Build Script"
echo "===================================================="

# 1. Clean previous build artifacts
echo "🧹 Cleaning previous dist artifacts..."
rm -rf dist
mkdir -p dist/DEER.app/Contents/MacOS
mkdir -p dist/DEER.app/Contents/Resources

# 2. Build Rust release binary
echo "⚡ Building Rust release binary (cargo build --release)..."
cargo build --release

# Copy binary into app bundle
cp target/release/deer dist/DEER.app/Contents/MacOS/deer
chmod +x dist/DEER.app/Contents/MacOS/deer

# 3. Create macOS AppIcon.icns directly from vector assets/logo.svg
if command -v qlmanage >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
    echo "🎨 Rendering vector assets/logo.svg to native macOS AppIcon.icns..."
    ICONSET="dist/AppIcon.iconset"
    mkdir -p "$ICONSET"
    TMP_DIR=$(mktemp -d)
    qlmanage -t -s 1024 -o "$TMP_DIR" assets/logo.svg >/dev/null 2>&1 || true
    SVG_PNG="$TMP_DIR/logo.svg.png"
    if [ ! -f "$SVG_PNG" ]; then
        SVG_PNG="assets/logo.png"
    fi
    sips -s format png -z 16 16     "$SVG_PNG" --out "$ICONSET/icon_16x16.png" >/dev/null 2>&1 || true
    sips -s format png -z 32 32     "$SVG_PNG" --out "$ICONSET/icon_16x16@2x.png" >/dev/null 2>&1 || true
    sips -s format png -z 32 32     "$SVG_PNG" --out "$ICONSET/icon_32x32.png" >/dev/null 2>&1 || true
    sips -s format png -z 64 64     "$SVG_PNG" --out "$ICONSET/icon_32x32@2x.png" >/dev/null 2>&1 || true
    sips -s format png -z 128 128   "$SVG_PNG" --out "$ICONSET/icon_128x128.png" >/dev/null 2>&1 || true
    sips -s format png -z 256 256   "$SVG_PNG" --out "$ICONSET/icon_128x128@2x.png" >/dev/null 2>&1 || true
    sips -s format png -z 256 256   "$SVG_PNG" --out "$ICONSET/icon_256x256.png" >/dev/null 2>&1 || true
    sips -s format png -z 512 512   "$SVG_PNG" --out "$ICONSET/icon_256x256@2x.png" >/dev/null 2>&1 || true
    sips -s format png -z 512 512   "$SVG_PNG" --out "$ICONSET/icon_512x512.png" >/dev/null 2>&1 || true
    sips -s format png -z 1024 1024 "$SVG_PNG" --out "$ICONSET/icon_512x512@2x.png" >/dev/null 2>&1 || true
    iconutil -c icns "$ICONSET" -o "dist/DEER.app/Contents/Resources/AppIcon.icns" || true
    rm -rf "$ICONSET" "$TMP_DIR"
fi

# 4. Generate macOS Info.plist
echo "📄 Generating Info.plist for DEER.app..."
cat << 'EOF' > dist/DEER.app/Contents/Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>deer</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.deer.visualprogramming</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DEER</string>
    <key>CFBundleDisplayName</key>
    <string>DEER — Diagram Execution Engine</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 5. Create single ZIP distribution package (app only, examples are embedded)
echo "📦 Packaging release archive (dist/DEER-macOS.zip)..."
(cd dist && zip -r DEER-macOS.zip DEER.app >/dev/null)

# 6. Flush macOS icon cache so Finder picks up the new icon
echo "🔄 Flushing macOS icon cache..."
touch dist/DEER.app
/usr/bin/xattr -cr dist/DEER.app 2>/dev/null || true
/usr/bin/killall Dock 2>/dev/null || true

echo ""
echo "===================================================="
echo "✅ Release build successful!"
echo "📍 macOS Bundle: dist/DEER.app"
echo "📍 ZIP Package:  dist/DEER-macOS.zip"
echo ""
echo "ℹ️  Example diagrams are embedded directly in the"
echo "    application — no external files needed."
echo "===================================================="
