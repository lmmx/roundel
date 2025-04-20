# map

A simple integration of MapLibre GL JS with Dioxus - a reactive UI framework for Rust.

## Overview

This project demonstrates how to integrate MapLibre GL JS with Dioxus to create an interactive map application compiled to WebAssembly. It serves as a starting point for developers looking to build map-based applications using Rust for the web.

## Features

- 🌍 Interactive world map with navigation controls
- 🦀 Written in Rust, compiled to WebAssembly
- 🔄 Reactive UI components with Dioxus

## Getting Started

### Development

```bash
dx serve --platform web
```

Then open `http://0.0.0.0:8080/`

### Deployment

```bash
dx build --platform web --release
```

Then copy everything under `target/dx/my-map/release/web/public` to your static site host.
(Everything is client side, for a backend you'd use `bundle`)

Note that things like the subpath of the domain you deploy from are set in Dioxus.toml

### Prerequisites

You'll need the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Dioxus CLI](https://dioxuslabs.com/docs/0.4/guide/en/installation.html)

For building Dioxus projects:
```bash
sudo apt install libgdk3.0-cil libatk1.0-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf2.0-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libwebkit2gtk-4.1-dev libxdo-dev -y
```

### Running the Application

Run the development server:

```bash
dx serve --platform web
```

This will build the project and serve it at http://localhost:8080 by default.

The application works by:

1. Setting up a Dioxus component structure
2. Creating a container for the MapLibre GL map
3. Loading MapLibre GL JS and CSS dynamically
4. Initializing the map when the component mounts
5. Using wasm-bindgen to interact between Rust and JavaScript

The key integration happens in the `Canvas` component, which:
- Creates a properly sized container for the map
- Loads MapLibre's required CSS and JavaScript
- Uses JavaScript interop to initialize the map
- Sets up event handlers between Rust and JavaScript

There's some complexity from Dioxus on top of that (in particular layer registration and simulation
triggering), but overall these are the important parts. As far as possible I've tried to move JS
logic out into Rust via WASM.

## Acknowledgments

Made with [Dioxus](https://dioxuslabs.com), a Reactive UI framework for Rust,
using the [MapLibre GL JS](https://maplibre.org) free and open-source map rendering library
using tiles from [OpenFreeMap](https://openfreemap.org/) (specifically via the map's style URL).
