# Building My LLM

End-user install is on [Releases](https://github.com/rinodrops/myllm/releases/latest). This file is for building from source.

## Requirements

- Rust (stable)
- [just](https://github.com/casey/just)
- [Settings](https://github.com/rinodrops/settings) cloned as a sibling (`../settings`) when bundling the Settings binary. Override with `SETTINGS_REPO`.
- macOS 13 or later to produce app bundles (Apple Silicon or Intel)
- [ImageMagick](https://imagemagick.org) `magick` for the Windows `.ico`
- [dmgbuild](https://github.com/cdgriffith/dmgbuild) (`pipx install dmgbuild`) for macOS DMGs
- Apple Developer certificate and notarization credentials for signed macOS artifacts

## Development

```bash
just dev
```

The debug GUI binary is `target/debug/myllm`. `just dev` does not copy Settings beside it, so the tray **Settings…** item stays disabled there.

```bash
# Persistent tray / menu (macOS and Windows)
target/debug/myllm

# Single-shot from a compositor shortcut, Alfred, or a terminal
target/debug/myllm --task polish
target/debug/myllm --task translate --to ja
```

On Wayland, assign those argv invocations in the compositor. In-process global hotkeys are not the primary entry point there.

## Install from a local build

On macOS, clone Settings first. `just install` builds Settings from this repository's [`schema.toml`](schema.toml) and copies `settings` next to `myllm` in `My LLM.app/Contents/MacOS/`. After install, tray **Settings…** opens that binary against the user config.

```bash
git clone https://github.com/rinodrops/myllm.git
cd myllm
just install   # dist/darwin-arm64/My LLM.app → /Applications
```

`just darwin-build-arm64` writes the unsigned Apple Silicon bundle under `dist/`. Intel Macs use `just darwin-build-x86_64`.

On Windows, `just install` copies `myllm.exe` and `settings.exe` to `%LOCALAPPDATA%\Programs\myllm\`.

## Signed macOS artifacts

```bash
just darwin-notarize-arm64   # dist/My-LLM-vVERSION-darwin-arm64.dmg
just darwin-zip-arm64        # notarized .app as a zip
just darwin-zip-x86_64       # Intel Mac
```

Version tags (`v*`) run `just darwin-zip-*` for both architectures on GitHub Actions, and `just win-zip` for Windows. Unsigned `just install` stays the Apple Silicon daily path.
