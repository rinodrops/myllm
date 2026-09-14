set windows-shell := ["sh", "-cu"]

app_name := "My LLM"
exe_name := "myllm"
bundle_id := "jp.emotiongraphics.myllm"
min_macos := "13.0"

version := `awk -F'"' '/^version *=/{print $2; exit}' Cargo.toml`

rust_target_arm64 := "aarch64-apple-darwin"
icon_src := "crates/myllm/assets/appicon.png"
settings_repo := "../settings"

default: help

help:
    @just --list

dev:
    cargo build -p myllm

# ---------------------------------------------------------------------------
# macOS
# ---------------------------------------------------------------------------

[macos]
darwin-build-arm64:
    just _darwin-bundle darwin-arm64 {{rust_target_arm64}}
    @echo "App bundle: dist/darwin-arm64/{{app_name}}.app"

[macos]
install: darwin-build-arm64
    rm -rf "/Applications/{{app_name}}.app"
    cp -r "dist/darwin-arm64/{{app_name}}.app" "/Applications/"

clean:
    cargo clean
    rm -rf dist

# ---------------------------------------------------------------------------
# Internal
# ---------------------------------------------------------------------------

[macos]
_darwin-bundle arch rust_target:
    MACOSX_DEPLOYMENT_TARGET={{min_macos}} cargo build --release -p myllm
    mkdir -p "dist/{{arch}}/{{app_name}}.app/Contents/MacOS"
    mkdir -p "dist/{{arch}}/{{app_name}}.app/Contents/Resources"
    cp "target/release/{{exe_name}}" \
        "dist/{{arch}}/{{app_name}}.app/Contents/MacOS/{{exe_name}}"
    just _bundle-settings \
        "dist/{{arch}}/{{app_name}}.app/Contents/MacOS/settings"
    just _plist "dist/{{arch}}/{{app_name}}.app/Contents"
    just _icns "dist/{{arch}}/{{app_name}}.app/Contents/Resources/AppIcon.icns"

[macos]
_bundle-settings dest:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f "{{settings_repo}}/Justfile" ]; then
        echo "error: Settings clone not found at {{settings_repo}}"
        echo "Clone https://github.com/rinodrops/settings as a sibling of this repository."
        exit 1
    fi
    ROOT="$(pwd)"
    (
        cd "{{settings_repo}}"
        SETTINGS_SCHEMA="${ROOT}/schema.toml" just binary
    )
    cp "{{settings_repo}}/target/release/settings" "{{dest}}"
    chmod +x "{{dest}}"

[macos]
_plist contents_dir:
    sed 's/@VERSION@/{{version}}/g' assets/darwin/Info.plist > "{{contents_dir}}/Info.plist"

[macos]
_icns icns_out:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f "{{icon_src}}" ]; then
        echo "Note: {{icon_src}} not found — skipping icon generation."
        exit 0
    fi
    ICONSET_WORK="$(mktemp -d)"
    ICONSET="${ICONSET_WORK}/AppIcon.iconset"
    mkdir -p "${ICONSET}"
    SRC_NORM="${ICONSET_WORK}/source-1024.png"
    sips -z 1024 1024 "{{icon_src}}" --out "${SRC_NORM}" >/dev/null
    sips --deleteColorManagementProperties "${SRC_NORM}" >/dev/null 2>&1 || true
    sips -z 16   16   "${SRC_NORM}" --out "${ICONSET}/icon_16x16.png"
    sips -z 32   32   "${SRC_NORM}" --out "${ICONSET}/icon_16x16@2x.png"
    sips -z 32   32   "${SRC_NORM}" --out "${ICONSET}/icon_32x32.png"
    sips -z 64   64   "${SRC_NORM}" --out "${ICONSET}/icon_32x32@2x.png"
    sips -z 128  128  "${SRC_NORM}" --out "${ICONSET}/icon_128x128.png"
    sips -z 256  256  "${SRC_NORM}" --out "${ICONSET}/icon_128x128@2x.png"
    sips -z 256  256  "${SRC_NORM}" --out "${ICONSET}/icon_256x256.png"
    sips -z 512  512  "${SRC_NORM}" --out "${ICONSET}/icon_256x256@2x.png"
    sips -z 512  512  "${SRC_NORM}" --out "${ICONSET}/icon_512x512.png"
    cp "${SRC_NORM}" "${ICONSET}/icon_512x512@2x.png"
    mkdir -p "$(dirname "{{icns_out}}")"
    ICNS_OUT_ABS="$(cd "$(dirname "{{icns_out}}")" && pwd)/$(basename "{{icns_out}}")"
    iconutil -c icns "${ICONSET}" -o "${ICNS_OUT_ABS}"
    rm -rf "${ICONSET_WORK}"
