# Path Intellisense for Zed

Provides file path autocompletion when typing `'` or `"` quotes in code — similar to the VS Code

## Features

- Autocomplete file paths inside quotes: `"` and `'`
- Works with relative paths (`./`, `../`) and absolute paths (`/`)
- Nested directory drill-down via `/` trigger
- Two-level deep scanning — files in subdirectories appear directly in the list
- Supports JavaScript, TypeScript, Rust, Python, Go, CSS, HTML, and [many more](#supported-languages)

## Installation

### Prerequisites

- [Zed](https://zed.dev) editor
- [Rust toolchain](https://rustup.rs) (to compile the LSP server)

### Step 1: Install the LSP Server

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/zed-path-intellisense.git
cd zed-path-intellisense

# Build the LSP server
cd zed-path-intellisense-lsp && cargo build --release

# Copy to a directory in your PATH
# macOS / Linux:
cp target/release/zed-path-intellisense-lsp /usr/local/bin/

# Windows (PowerShell as Administrator):
# copy target\release\zed-path-intellisense-lsp.exe C:\Windows\System32\
```

Or install via Cargo directly:

```bash
cargo install --path zed-path-intellisense-lsp
```

### Step 2: Install the Extension

1. Open Zed
2. Press `Cmd+Shift+P` (macOS) / `Ctrl+Shift+P` (Windows/Linux)
3. Type `zed: extensions` and press Enter
4. Click the gear icon → **Install Dev Extension**
5. Select the `zed-path-intellisense` folder from the cloned repo

### Step 3: Reload

Restart Zed or reload the current window. Open a file and type `"./` inside quotes — file paths should autocomplete.

## Usage

| Action | Result |
|---|---|
| Type `"` or `'` | Triggers completion |
| Type `./` | Shows files and directories in current dir |
| Type `../` | Shows files in parent directory |
| Type `/` | Shows files from workspace root |
| Select a directory | Inserts directory name (no trailing slash) |
| Type `/` after directory name | Shows contents of that directory |
| Select a nested file | Inserts `dir/filename.ext` directly |

## Supported Languages

TypeScript, JavaScript, TSX, JSX, CSS, SCSS, HTML, Rust, Python, Go, Ruby, PHP, C, C++

## How It Works

The extension consists of two components:

1. **WASM Extension** (`zed-path-intellisense/`) — registered as a Zed language server extension. Launches the native LSP binary.
2. **LSP Server** (`zed-path-intellisense-lsp/`) — a native Rust binary that implements the Language Server Protocol. Listens for completion requests and returns file path suggestions.

The WASM extension uses `worktree.which()` to find the LSP binary in your `PATH`, so the binary must be installed separately.

## Publishing to the Zed Extension Registry

To make the extension available to all Zed users from the built-in extensions panel:

### 1. Prepare the Repository

- Push the code to a public GitHub repository
- Update the `repository` field in `extension.toml` to point to your repo
- Update `authors` in `extension.toml` with your name/email

### 2. Build Pre-compiled Binaries (Recommended)

Set up GitHub Actions to build the LSP server for all platforms. Create `.github/workflows/release.yml`:

```yaml
name: Build and Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --manifest-path zed-path-intellisense-lsp/Cargo.toml
      - uses: softprops/action-gh-release@v1
        with:
          files: zed-path-intellisense-lsp/target/release/zed-path-intellisense-lsp*
```

### 3. Submit to the Zed Registry

1. Fork [zed-industries/extensions](https://github.com/zed-industries/extensions)
2. Add a submodule pointing to your extension repository
3. Create a Pull Request

The Zed team will review and merge it. Once accepted, users can install the extension directly from the extensions panel.

## Building from Source

```bash
# Build everything
git clone https://github.com/YOUR_USERNAME/zed-path-intellisense.git
cd zed-path-intellisense
make all

# Or build individually:
make build-lsp        # Build the LSP server
make build-extension  # Build the WASM extension
```

## Development

To test changes locally:

1. Make changes to the LSP server (`zed-path-intellisense-lsp/`)
2. Run `make build-lsp` to compile
3. Copy the binary: `cp zed-path-intellisense-lsp/target/release/zed-path-intellisense-lsp /usr/local/bin/`
4. Make changes to the WASM extension (`zed-path-intellisense/`)
5. Run `make build-extension` to compile
6. Reinstall the dev extension in Zed: Extensions panel → Install Dev Extension → select `zed-path-intellisense/`
7. Restart Zed

## Troubleshooting

**No completions appear:**
- Ensure the LSP binary is in your PATH: `which zed-path-intellisense-lsp`
- Check Zed logs: `tail -f ~/Library/Logs/Zed/Zed.log | grep zed-path-intellisense`
- Restart Zed completely (`Cmd+Q` / close all windows)
- Make sure the dev extension is correctly installed (visible in extensions list)

**Directories insert with trailing slash:**
- This is the standard behavior — type `/` manually after a directory name to trigger the next level of completions

## License

MIT
