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

The GUI binary is `target/debug/myllm`.

```bash
# Persistent tray / menu (macOS and Windows)
target/debug/myllm

# Single-shot from a compositor shortcut, Alfred, or a terminal
target/debug/myllm --task polish
target/debug/myllm --task translate --to ja
```

On Wayland, assign those argv invocations in the compositor. In-process global hotkeys are not the primary entry point there.

## Configuration

The config file is `${XDG_CONFIG_HOME:-$HOME/.config}/myllm/config.toml`. A starter file is copied on first launch from [`config/config.toml`](config/config.toml).

Settings are edited by spawning the separate [Settings](https://github.com/rinodrops/settings) process. This repository owns `schema.toml`; it does not vendor Settings source.

## License

MIT License. See [LICENSE](LICENSE).
