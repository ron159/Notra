<p align="center">
  <img src="crates/otterdive-app/icons/128x128.png" width="96" height="96" alt="OtterDive icon" />
</p>

<h1 align="center">OtterDive</h1>

<p align="center"><strong>Dive into massive text. Surface what matters.</strong></p>

<p align="center"><a href="README.md">简体中文</a> · English</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-3b82f6.svg" alt="MIT License" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-2563eb.svg" alt="Windows, macOS and Linux" />
  <img src="https://img.shields.io/badge/Tauri-2-24c8db.svg" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Monaco-0.53-007acc.svg" alt="Monaco Editor" />
</p>

OtterDive is a local-first desktop tool for text, logs, Markdown, and source code. It brings fast opening and editing, workspace search, regular expressions, Analyse rule processing, and structured result browsing into one workflow: dive into large bodies of text and bring the relevant content back to the surface.

## Key Features

- Open or drag files and folders in single-file or workspace mode.
- Monaco-based editing with syntax highlighting, completion, folding, bracket matching, multiple cursors, and read-only protection for files above the editable-size threshold.
- VS Code and Notepad++ keymap profiles, with grouped command discovery and custom bindings.
- Instant Markdown editing with source and split-preview modes.
- Markdown outlines, tables, task lists, math, remote images, Mermaid, PlantUML, and Vega diagrams.
- Search the current file, all open documents, or an entire workspace with normal, extended, and regular-expression modes, result navigation, and replacement previews.
- Import compatible XML profiles into Analyse, run Normal, Escaped, Regex, and multiline Regex rules, and export merged results, styles, bookmarks, HTML, and RTF.
- Detect and convert UTF-8, UTF-8 BOM, UTF-16, and ANSI encodings, with LF, CRLF, and CR line-ending support.
- Restore drafts, open tabs, window placement, workspaces, search history, Analyse settings, and editor preferences.
- Custom window chrome, light and dark themes, file-type icons, and configurable editor appearance.

## Project Status

OtterDive is under active development. GitHub Releases provide Windows x64, macOS ARM64, and Linux x64 packages. Windows 10/11 is currently the primary test platform; macOS and Linux packages are built by GitHub Actions.

Downloads are available from [GitHub Releases](https://github.com/syscryer/Notra/releases).

## Local Development

Requirements:

- Rust stable
- Node.js 20.19 or later
- npm 8 or later
- Windows WebView2 Runtime

Install frontend dependencies:

```powershell
cd crates\otterdive-app\frontend
npm install
```

Start the desktop app in development mode:

```powershell
cd crates\otterdive-app
cargo tauri dev
```

Build a Windows installer:

```powershell
cd crates\otterdive-app
cargo tauri build
```

Build artifacts are written to `target\release\bundle`.

## Validation

```powershell
cargo test --workspace

cd crates\otterdive-app\frontend
npm run test:keybindings
npm run build
```

## Project Structure

```text
crates/otterdive-app/           Tauri desktop application and frontend
crates/otterdive-core/          Document, encoding, search, and filesystem core
crates/otterdive-app/frontend/  Monaco and Markdown editing experience
scripts/                        Upstream synchronization and development scripts
```

## Upstream Projects and Acknowledgements

OtterDive is released under the [MIT License](LICENSE).

OtterDive is a search-enhanced edition of the upstream [Notra](https://github.com/syscryer/Notra) project. It builds on Notra with a stronger focus on workspace search, regular expressions, Analyse, logs, and large-text workflows. We thank the original Notra authors and contributors for the product and engineering foundation.

The instant Markdown experience evolves from the [MarkText](https://github.com/marktext/marktext) Muya editor. Its pinned upstream revision and licenses are kept in `crates/otterdive-app/frontend/vendor/marktext-muya`. Code editing is powered by [Monaco Editor](https://github.com/microsoft/monaco-editor). We thank these upstream projects and their contributors; all third-party dependencies remain subject to their respective licenses.
