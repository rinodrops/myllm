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

On macOS, daily use is an unsigned `.app`:

```bash
just install   # dist/darwin-arm64/My LLM.app → /Applications
```

`just darwin-build-arm64` only writes the bundle under `dist/`. Signing and notarization are not included yet.

The macOS `.app` embeds a [Settings](https://github.com/rinodrops/settings) binary built with this repository's [`schema.toml`](schema.toml). Clone Settings as a sibling of this repository (`../settings`) before `just install`. After install, tray **Settings…** opens that binary against the user config. `just dev` does not copy Settings beside `target/debug/myllm`, so the tray item stays disabled there.

## Configuration

The config file is `${XDG_CONFIG_HOME:-$HOME/.config}/myllm/config.toml`. A starter file is copied on first launch from [`config/config.toml`](config/config.toml).

This repository owns `schema.toml`; it does not vendor Settings source.

## License

MIT License. See [LICENSE](LICENSE).
