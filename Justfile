set windows-shell := ["sh", "-cu"]

app_name := "My LLM"
pkg_name := "My-LLM"
exe_name := "myllm"
bundle_id := "jp.emotiongraphics.myllm"
min_macos := "13.0"

version := `awk -F'"' '/^version *=/{print $2; exit}' Cargo.toml`

rust_target_arm64 := "aarch64-apple-darwin"
rust_target_x86 := "x86_64-apple-darwin"
icon_src := "crates/myllm/assets/appicon.png"
win_icon_src := "crates/myllm/assets/appicon-windows.png"
settings_repo := env_var_or_default("SETTINGS_REPO", "../settings")
entitlements := "assets/darwin/entitlements.plist"
dmg_settings := "assets/darwin/dmg_settings.py"
dmg_background := justfile_directory() + "/assets/darwin/dmg-background.png"

default: help

help:
    @just --list

dev:
    cargo build -p myllm

# ---------------------------------------------------------------------------
# macOS
# ---------------------------------------------------------------------------

[macos]
darwin-build: darwin-build-arm64 darwin-build-x86_64

[macos]
darwin-build-arm64:
    just _darwin-bundle darwin-arm64 {{rust_target_arm64}}
    @echo "App bundle: dist/darwin-arm64/{{app_name}}.app"

[macos]
darwin-build-x86_64:
    just _darwin-bundle darwin-x86_64 {{rust_target_x86}}
    @echo "App bundle: dist/darwin-x86_64/{{app_name}}.app"

[macos]
darwin-sign-arm64: darwin-build-arm64
    just _darwin-sign darwin-arm64

[macos]
darwin-sign-x86_64: darwin-build-x86_64
    just _darwin-sign darwin-x86_64

[macos]
darwin-dmg-arm64: darwin-build-arm64
    just _darwin-create-dmg darwin-arm64
    @echo "DMG created: dist/{{pkg_name}}-v{{version}}-darwin-arm64.dmg"

[macos]
darwin-dmg-x86_64: darwin-build-x86_64
    just _darwin-create-dmg darwin-x86_64
    @echo "DMG created: dist/{{pkg_name}}-v{{version}}-darwin-x86_64.dmg"

[macos]
darwin-notarize-arm64: darwin-sign-arm64
    just _darwin-notarize darwin-arm64

[macos]
darwin-notarize-x86_64: darwin-sign-x86_64
    just _darwin-notarize darwin-x86_64

[macos]
darwin-zip-arm64: darwin-notarize-arm64
    just _darwin-zip darwin-arm64

[macos]
darwin-zip-x86_64: darwin-notarize-x86_64
    just _darwin-zip darwin-x86_64

[macos]
darwin-release: darwin-notarize-arm64 darwin-notarize-x86_64

[macos]
install: darwin-build-arm64
    rm -rf "/Applications/{{app_name}}.app"
    cp -r "dist/darwin-arm64/{{app_name}}.app" "/Applications/"

# ---------------------------------------------------------------------------
# Windows
# ---------------------------------------------------------------------------

win_target := "x86_64-pc-windows-gnu"
win_target_dir := "/tmp/myllm-win"
ico_out := "crates/myllm/assets/appicon.ico"

[windows]
win-build:
    just _appicon-ico
    cargo build --release -p myllm
    mkdir -p dist/windows-x86_64
    cp "target/release/{{exe_name}}.exe" "dist/windows-x86_64/{{exe_name}}.exe"
    just _bundle-settings-win dist/windows-x86_64
    @echo "Windows build: dist/windows-x86_64/{{exe_name}}.exe"

[macos]
win-build:
    just _appicon-ico
    CARGO_TARGET_DIR="{{win_target_dir}}" cargo build --release -p myllm --target {{win_target}}
    mkdir -p dist/windows-x86_64
    cp "{{win_target_dir}}/{{win_target}}/release/{{exe_name}}.exe" \
        "dist/windows-x86_64/{{exe_name}}.exe"
    just _bundle-settings-win dist/windows-x86_64
    @echo "Windows build: dist/windows-x86_64/{{exe_name}}.exe"

win-zip: win-build
    #!/usr/bin/env bash
    set -euo pipefail
    ZIP="dist/{{pkg_name}}-v{{version}}-windows-x86_64.zip"
    rm -f "${ZIP}"
    (
        cd dist/windows-x86_64
        zip "../$(basename "${ZIP}")" "{{exe_name}}.exe" settings.exe
    )
    echo "Zip created: ${ZIP}"

win-release: win-zip

[windows]
install: win-build
    #!/usr/bin/env bash
    set -euo pipefail
    DEST="${LOCALAPPDATA}/Programs/{{exe_name}}"
    mkdir -p "${DEST}"
    cp "dist/windows-x86_64/{{exe_name}}.exe" "${DEST}/"
    cp "dist/windows-x86_64/settings.exe" "${DEST}/"

clean:
    cargo clean
    rm -rf dist "{{win_target_dir}}"

# ---------------------------------------------------------------------------
# Internal
# ---------------------------------------------------------------------------

[macos]
_darwin-bundle arch rust_target:
    MACOSX_DEPLOYMENT_TARGET={{min_macos}} cargo build --release -p myllm --target {{rust_target}}
    mkdir -p "dist/{{arch}}/{{app_name}}.app/Contents/MacOS"
    mkdir -p "dist/{{arch}}/{{app_name}}.app/Contents/Resources"
    cp "target/{{rust_target}}/release/{{exe_name}}" \
        "dist/{{arch}}/{{app_name}}.app/Contents/MacOS/{{exe_name}}"
    just _plist "dist/{{arch}}/{{app_name}}.app/Contents"
    just _icns "dist/{{arch}}/{{app_name}}.app/Contents/Resources/AppIcon.icns"
    just _bundle-settings "dist/{{arch}}/{{app_name}}.app/Contents/MacOS" {{rust_target}}

[macos]
_bundle-settings dest_macos rust_target:
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
    case "{{rust_target}}" in
        aarch64-apple-darwin) SETTINGS_RECIPE="binary-arm64" ;;
        x86_64-apple-darwin) SETTINGS_RECIPE="binary-x86_64" ;;
        *)
            echo "Error: unsupported Settings target {{rust_target}}" >&2
            exit 1
            ;;
    esac
    echo "Building Settings from ${ROOT}/schema.toml  ({{rust_target}})"
    (cd "${SETTINGS_DIR}" && \
        SCHEMA="${ROOT}/schema.toml" \
        MACOSX_DEPLOYMENT_TARGET="{{min_macos}}" \
        just "${SETTINGS_RECIPE}")
    cp "${SETTINGS_DIR}/target/{{rust_target}}/release/settings" "{{dest_macos}}/settings"
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
_darwin-sign arch:
    just _require-cert
    xattr -cr "dist/{{arch}}/{{app_name}}.app"
    codesign --deep --force --options runtime \
        --entitlements "{{entitlements}}" \
        --sign "${APPLE_DEVELOPER_CERTIFICATE_NAME}" \
        "dist/{{arch}}/{{app_name}}.app"
    @echo "Signed: dist/{{arch}}/{{app_name}}.app"

[macos]
_darwin-create-dmg arch:
    mkdir -p dist
    just _require-dmgbuild
    dmgbuild \
        -s "{{dmg_settings}}" \
        -D app="dist/{{arch}}/{{app_name}}.app" \
        -D background="{{dmg_background}}" \
        "{{app_name}}" \
        "dist/{{pkg_name}}-v{{version}}-{{arch}}.dmg"

[macos]
_darwin-notarize arch:
    just _require-notarize-env
    just _darwin-create-dmg {{arch}}
    xcrun notarytool submit \
        "dist/{{pkg_name}}-v{{version}}-{{arch}}.dmg" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_DEVELOPER_APP_PASSWORD}" \
        --team-id "${APPLE_DEVELOPER_TEAM_ID}" \
        --wait
    xcrun stapler staple "dist/{{pkg_name}}-v{{version}}-{{arch}}.dmg"
    @echo "Notarized: dist/{{pkg_name}}-v{{version}}-{{arch}}.dmg"

[macos]
_darwin-zip arch:
    ditto -c -k --keepParent \
        "dist/{{arch}}/{{app_name}}.app" \
        "dist/{{pkg_name}}-v{{version}}-{{arch}}.zip"
    @echo "Zip created: dist/{{pkg_name}}-v{{version}}-{{arch}}.zip"

_appicon-ico:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v magick >/dev/null 2>&1 || \
        { echo "Error: ImageMagick magick not found" >&2; exit 1; }
    magick "{{win_icon_src}}" -define icon:auto-resize=256,48,32,16 "{{ico_out}}"
    echo "Generated: {{ico_out}}"

_bundle-settings-win dest_dir:
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
    echo "Building Settings from ${ROOT}/schema.toml  (windows-x86_64)"
    (cd "${SETTINGS_DIR}" && SCHEMA="${ROOT}/schema.toml" just settings-win-build)
    mkdir -p "{{dest_dir}}"
    cp "${SETTINGS_DIR}/dist/settings/windows-x86_64/Settings.exe" \
        "{{dest_dir}}/settings.exe"
    echo "Bundled Settings: {{dest_dir}}/settings.exe"

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
