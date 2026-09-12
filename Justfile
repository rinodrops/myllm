set windows-shell := ["sh", "-cu"]

app_name := "My LLM"
exe_name := "myllm"
bundle_id := "jp.emotiongraphics.myllm"
min_macos := "13.0"

version := `awk -F'"' '/^version *=/{print $2; exit}' Cargo.toml`

default: help

help:
    @just --list

dev:
    cargo build -p myllm

clean:
    cargo clean
    rm -rf dist
