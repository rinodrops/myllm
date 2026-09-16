# My LLM

A personal LLM toolkit. Select text or type into a named task (`polish`, `translate`, …) and get a streamed result in a floating window.

This repository holds the Rust library and egui GUI. The frozen Classic apps live in [myllm-classic](https://github.com/rinodrops/myllm-classic) (macOS Swift GUI) and [myllm-cli](https://github.com/rinodrops/myllm-cli) (bash CLI for pipes).

## Requirements

- Rust (stable)
- A running [Ollama](https://ollama.com) instance, or an OpenAI / Anthropic API key

## Build

```bash
git clone https://github.com/rinodrops/myllm.git
cd myllm
just dev
```

The debug GUI binary is `target/debug/myllm`.

```bash
# Persistent tray / menu (macOS and Windows)
target/debug/myllm

# Single-shot from a compositor shortcut, Alfred, or a terminal
target/debug/myllm --task polish
target/debug/myllm --task translate --to ja
```

On Wayland, assign those argv invocations in the compositor. In-process global hotkeys are not the primary entry point there.

On macOS, daily use is an unsigned `.app`. Clone [Settings](https://github.com/rinodrops/settings) as a sibling of this repository (`../settings`) first. `just install` builds Settings from this repository's [`schema.toml`](schema.toml) and copies `settings` next to `myllm` in `My LLM.app/Contents/MacOS/`. After install, tray **Settings…** opens that binary against the user config.

```bash
just install   # dist/darwin-arm64/My LLM.app → /Applications
```

`just darwin-build-arm64` writes the unsigned Apple Silicon bundle under `dist/`. Intel Macs use `just darwin-build-x86_64`. `just dev` does not copy Settings beside `target/debug/myllm`, so the tray item stays disabled there.

Signed, notarized DMGs and zips:

```bash
just darwin-notarize-arm64   # dist/My-LLM-vVERSION-darwin-arm64.dmg
just darwin-zip-arm64        # notarized .app as a zip
just darwin-zip-x86_64       # Intel Mac
```

Version tags (`v*`) run `just darwin-zip-*` for both architectures on GitHub Actions. Unsigned `just install` stays the Apple Silicon daily path.

## Configuration

The config file is `${XDG_CONFIG_HOME:-$HOME/.config}/myllm/config.toml`. A starter file is copied on first launch from [`config/config.toml`](config/config.toml). Window, tray, and Settings labels follow `[general] ui_lang` (`os` or a whichlang language code).

This repository owns `schema.toml`; it does not vendor Settings source.

## License

MIT License. See [LICENSE](LICENSE).
