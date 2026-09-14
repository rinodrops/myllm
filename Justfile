set windows-shell := ["sh", "-cu"]

app_name := "My LLM"
pkg_name := "My-LLM"
exe_name := "myllm"
bundle_id := "jp.emotiongraphics.myllm"
min_macos := "13.0"

version := `awk -F'"' '/^version *=/{print $2; exit}' Cargo.toml`

rust_target_arm64 := "aarch64-apple-darwin"
icon_src := "crates/myllm/assets/appicon.png"
settings_repo := env_var_or_default("SETTINGS_REPO", "../settings")
entitlements := "assets/darwin/entitlements.plist"
dmg_settings := "assets/darwin/dmg_settings.py"
app_bundle := "dist/darwin-arm64/" + app_name + ".app"
dmg_path := "dist/" + pkg_name + "-v" + version + "-darwin-arm64.dmg"

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
    @echo "App bundle: {{app_bundle}}"

[macos]
darwin-sign-arm64: darwin-build-arm64
    just _require-cert
    xattr -cr "{{app_bundle}}"
    codesign --deep --force --options runtime \
        --entitlements "{{entitlements}}" \
        --sign "${APPLE_DEVELOPER_CERTIFICATE_NAME}" \
        "{{app_bundle}}"
    @echo "Signed: {{app_bundle}}"

[macos]
darwin-dmg-arm64: darwin-build-arm64
    just _darwin-create-dmg
    @echo "DMG created: {{dmg_path}}"

[macos]
darwin-notarize-arm64: darwin-sign-arm64
    just _require-notarize-env
    just _darwin-create-dmg
    xcrun notarytool submit \
        "{{dmg_path}}" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_DEVELOPER_APP_PASSWORD}" \
        --team-id "${APPLE_DEVELOPER_TEAM_ID}" \
        --wait
    xcrun stapler staple "{{dmg_path}}"
    @echo "Notarized: {{dmg_path}}"

[macos]
darwin-release: darwin-notarize-arm64

[macos]
install: darwin-build-arm64
    rm -rf "/Applications/{{app_name}}.app"
    cp -r "{{app_bundle}}" "/Applications/"

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
    just _plist "dist/{{arch}}/{{app_name}}.app/Contents"
    just _icns "dist/{{arch}}/{{app_name}}.app/Contents/Resources/AppIcon.icns"
    just _bundle-settings "dist/{{arch}}/{{app_name}}.app/Contents/MacOS"

[macos]
_bundle-settings dest_macos:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="$(cd "{{justfile_directory()}}" && pwd)"
    SETTINGS_DIR="{{settings_repo}}"
    if [[ "${SETTINGS_DIR}" != /* ]]; then
        SETTINGS_DIR="${ROOT}/{{settings_repo}}"
    fi
    if [ ! -f "${SETTINGS_DIR}/Justfile" ]; then
        echo "Error: Settings Justfile not found at ${SETTINGS_DIR}." >&2
        echo "Clone https://github.com/rinodrops/settings as a sibling of this repository." >&2
        exit 1
    fi
    if [ ! -f "${ROOT}/schema.toml" ]; then
        echo "Error: schema.toml not found at ${ROOT}/schema.toml" >&2
        exit 1
    fi
    echo "Building Settings from ${ROOT}/schema.toml"
    (cd "${SETTINGS_DIR}" && \
        SCHEMA="${ROOT}/schema.toml" \
        MACOSX_DEPLOYMENT_TARGET="{{min_macos}}" \
        just binary)
    cp "${SETTINGS_DIR}/target/release/settings" "{{dest_macos}}/settings"
    echo "Bundled Settings: {{dest_macos}}/settings"

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

[macos]
_darwin-create-dmg:
    mkdir -p dist
    just _require-dmgbuild
    dmgbuild \
        -s "{{dmg_settings}}" \
        -D app="{{app_bundle}}" \
        "{{app_name}}" \
        "{{dmg_path}}"

_require-dmgbuild:
    #!/usr/bin/env bash
    command -v dmgbuild >/dev/null 2>&1 || \
        { echo "Error: dmgbuild not found. Run: pipx install dmgbuild" >&2; exit 1; }

_require-cert:
    #!/usr/bin/env bash
    test -n "${APPLE_DEVELOPER_CERTIFICATE_NAME:-}" || \
        { echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" >&2; exit 1; }

_require-notarize-env:
    #!/usr/bin/env bash
    test -n "${APPLE_DEVELOPER_TEAM_ID:-}" || \
        { echo "Error: APPLE_DEVELOPER_TEAM_ID is not set" >&2; exit 1; }
    test -n "${APPLE_ID:-}" || \
        { echo "Error: APPLE_ID is not set" >&2; exit 1; }
    test -n "${APPLE_DEVELOPER_APP_PASSWORD:-}" || \
        { echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" >&2; exit 1; }
