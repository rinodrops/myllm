# My LLM

A personal LLM toolkit. Select text or type into a named task (`polish`, `translate`, …) and get a streamed result in a floating window.

This repository holds the Rust library and egui GUI. The frozen Classic apps live in [myllm-classic](https://github.com/rinodrops/myllm-classic) (macOS Swift GUI) and [myllm-cli](https://github.com/rinodrops/myllm-cli) (bash CLI for pipes).

## Features

- Floating result window with streamed output and a task picker
- Tray / menu bar (macOS and Windows) plus per-task global hotkeys
- Named tasks from a single `config.toml`
- Translation with automatic source detection (whichlang, 16 languages)
- Local Ollama or cloud OpenAI / Anthropic
- Settings as a separate process, driven by this repository's `schema.toml`
- Window, tray, and Settings labels in any whichlang language (`[general] ui_lang`)

## Requirements

- macOS 13 Ventura or later for the app bundle (Apple Silicon or Intel)
- Rust (stable) to build from source
- A running [Ollama](https://ollama.com) instance, or an OpenAI / Anthropic API key
- [Settings](https://github.com/rinodrops/settings) cloned as a sibling (`../settings`) when bundling the Settings binary

## Install

On macOS, daily use is an unsigned `.app`. Clone Settings first. `just install` builds Settings from this repository's [`schema.toml`](schema.toml) and copies `settings` next to `myllm` in `My LLM.app/Contents/MacOS/`. After install, tray **Settings…** opens that binary against the user config.

```bash
git clone https://github.com/rinodrops/myllm.git
cd myllm
just install   # dist/darwin-arm64/My LLM.app → /Applications
```

`just darwin-build-arm64` writes the unsigned Apple Silicon bundle under `dist/`. Intel Macs use `just darwin-build-x86_64`.

Signed, notarized DMGs and zips:

```bash
just darwin-notarize-arm64   # dist/My-LLM-vVERSION-darwin-arm64.dmg
just darwin-zip-arm64        # notarized .app as a zip
just darwin-zip-x86_64       # Intel Mac
```

Version tags (`v*`) run `just darwin-zip-*` for both architectures on GitHub Actions, and `just win-zip` for Windows. Unsigned `just install` stays the Apple Silicon daily path.

On Windows, `just install` copies `myllm.exe` and `settings.exe` to `%LOCALAPPDATA%\Programs\myllm\`. Tag CI uploads `My-LLM-vVERSION-windows-x86_64.zip` (unsigned).

## Build

```bash
just dev
```

The debug GUI binary is `target/debug/myllm`. `just dev` does not copy Settings beside it, so the tray item stays disabled there.

```bash
# Persistent tray / menu (macOS and Windows)
target/debug/myllm

# Single-shot from a compositor shortcut, Alfred, or a terminal
target/debug/myllm --task polish
target/debug/myllm --task translate --to ja
```

On Wayland, assign those argv invocations in the compositor. In-process global hotkeys are not the primary entry point there.

## Usage

Launch the app (or `myllm` with no arguments) to keep the tray resident. Use a task hotkey, **Open Window**, or `myllm --task <id>` to run.

The window has Input (top) and Output (bottom). **Run** / **Copy** sit in the bottom bar with the task picker. Responses stream into Output; **Copy** and optional `auto_copy` write the result to the clipboard.

Tray items:

- **Open Window** — show the window without grabbing selection
- Configured tasks, plus **Translate** when translation is enabled
- **Reload Config**, **Open Config Folder**, **Settings…**, **Quit My LLM**

## Configuration

The config file is `${XDG_CONFIG_HOME:-$HOME/.config}/myllm/config.toml`. A starter file is copied on first launch from [`config/config.toml`](config/config.toml).

Window, tray, and Settings labels follow `[general] ui_lang` (`os` or a whichlang code: `ar`, `nl`, `en`, `fr`, `de`, `hi`, `it`, `ja`, `ko`, `zh`, `pt`, `ru`, `es`, `sv`, `tr`, `vi`). This is independent of translation `default_target`.

Tray **Settings…** edits the same file. This repository owns `schema.toml`; it does not vendor Settings source.

Model resolution: the task's `model`, else the provider's `default_model`. Provider resolution: the task's `provider`, else `[general] default_provider`. API keys: `api_key`, else `api_key_env` (name must start with `MYLLM_`).

Translation lives under `[translation]`, not `[tasks]`. `engine = "translategemma"` uses the built-in prompt; `engine = "custom"` expands `{SOURCE_LANG}` `{SOURCE_CODE}` `{TARGET_LANG}` `{TARGET_CODE}` `{TEXT}`.

## Release notes

### 1.0.0

First stable release of the Rust library and egui GUI.

- Floating result window, tray, hotkeys, and streamed output
- Ollama, OpenAI, and Anthropic providers
- Translation with TranslateGemma or a custom instruction, plus whichlang detection
- Settings as a bundled separate process
- Chrome (window, tray, Settings labels, GUI notices) in 16 languages
- macOS app bundles for Apple Silicon and Intel, with notarized GitHub tag artifacts
- Windows `x86_64` zip with Settings beside the exe (unsigned)

## License

MIT License. See [LICENSE](LICENSE).
