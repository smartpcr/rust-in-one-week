# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust learning repository organized as a Cargo workspace containing multiple independent example projects demonstrating various Rust concepts.

## Build Commands

```bash
# Build entire workspace
cargo build

# Build specific project
cargo build -p <project-name>

# Run specific project
cargo run -p <project-name>

# Run tests for entire workspace
cargo test

# Run tests for specific project
cargo test -p <project-name>

# Check code without building
cargo check
```

## Workspace Structure

The workspace contains these independent crates:

- **basics** - Core Rust language fundamentals
- **hello-rust** - Introductory examples with ferris-says and chrono
- **use-mod** - Module system examples including local path dependencies (math subcrate)
- **cmd** - Actix-web server with module visibility examples
- **quick-replace** - CLI text replacement tool using regex
- **mandelbrot** - Parallel Mandelbrot set renderer using crossbeam for multi-threading
- **seesaw** - Trait and struct examples with rand
- **web-with-prometheus** - Nickel web server with Prometheus metrics
- **api** - Actix-web API with SQLite, OpenSSL, and authentication support

## Architecture Notes

- Each crate is self-contained with its own `Cargo.toml`
- The `use-mod` crate demonstrates local path dependencies with its nested `math` subcrate
- The `mandelbrot` crate shows a nested module structure: `library/common/environment.rs`, `library/math/mandelbrot.rs`
- Web projects use different frameworks: `cmd` and `api` use actix-web, `web-with-prometheus` uses nickel
