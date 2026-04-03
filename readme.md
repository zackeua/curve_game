# Curve Game

A clone of if the games Achtung! die kurwe and Curve Fever.

## Prerequisites

### For Native Builds
- Rust (1.70 or later) - Install from [rustup.rs](https://rustup.rs/)
- System dependencies depend on your OS:
  - **Linux**: SDL2 development libraries
    ```bash
    # Ubuntu/Debian
    sudo apt-get install libsdl2-dev
    
    # Fedora
    sudo dnf install SDL2-devel
    
    # Arch
    sudo pacman -S sdl2
    ```
  - **macOS**: 
    ```bash
    brew install sdl2
    ```
  - **Windows**: MSVC toolchain (included with Visual Studio or Visual Studio Build Tools)

### For WebAssembly Builds
- Rust with the wasm32 target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

## Building

### Native Build

Build the native binary optimized for your platform:

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (optimized for performance)
cargo build --release
```

The compiled binary will be located at:
- Debug: `target/debug/curve_game`
- Release: `target/release/curve_game`

### WebAssembly Build

Build for WebAssembly to run in a browser:

```bash
# Debug build
cargo build --target wasm32-unknown-unknown

# Release build (recommended for distribution)
cargo build --release --target wasm32-unknown-unknown
```

The WASM binary will be located at:
- Debug: `target/wasm32-unknown-unknown/debug/curve_game.wasm`
- Release: `target/wasm32-unknown-unknown/release/curve_game.wasm`

## Running

### Running Native

```bash
# Run debug build
cargo run

# Run release build
cargo run --release

# Or run the compiled binary directly
./target/release/curve_game
```

### Running WebAssembly

To run the WebAssembly version, serve it with an HTTP server. For example:

```bash
# Using Python 3
uv run python -m http.server 8000
```

Then open `http://localhost:8000/index.html` in your web browser.

## Project Structure

- `src/main.rs` - Main game logic and implementation
- `Cargo.toml` - Project configuration and dependencies
- `index.html` - Web page for running the WASM build
- `gl.js` - WebGL bindings for browser
