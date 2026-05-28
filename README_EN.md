<div align="right">

[中文](./README.md) | English

</div>

<div align="center">

<img src="vi-clip/public/logo.png" alt="ViClip Logo" width="120">

# ViClip

**Lightweight Desktop Productivity Suite**

Clipboard Manager · Quick Phrases · Multi-Engine Translation · Image Preview

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%2010+-brightgreen.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-ffc131.svg)
![React](https://img.shields.io/badge/React-19-61dafb.svg)

</div>

---

## Overview

ViClip is a lightweight desktop productivity suite that appears as a floating window and minimizes to the system tray when closed. It integrates clipboard history management, quick phrases, multi-engine translation, radial menu, and image preview into one seamless workflow.

## Features

### Clipboard Manager

- Automatically records text, image, link, and file copy history
- Content hash deduplication with intelligent duplicate filtering
- Keyword search and type filtering
- Single-click / double-click to paste at cursor position (configurable)
- Right-click context menu: copy content, delete records
- Configurable retention period (1 day / 1 week / 1 month / 3 months / forever) with auto cleanup
- Image thumbnail preview with hover action buttons

### Image Preview

- Click any image to open a standalone preview window
- Pin window on top for side-by-side reference
- Lock aspect ratio when resizing
- DWM acrylic backdrop effect matching system aesthetics

### Quick Phrases

- Organize frequently used scripts, code snippets, and templates by groups
- Create, rename, reorder, and delete groups
- Click to paste instantly — no manual copying needed

### Translation

- **AI Translation**: OpenAI-compatible API format (supports `/v1/chat/completions`), customizable endpoint and model
- **Google Translation**: Built-in free API ready out of the box; official API key also supported
- **Baidu Translation**: Professional API, ideal for Chinese language scenarios
- **Youdao Translation**: Multi-language pairs with stable domestic access
- **Tencent Cloud TMT**: Tencent Cloud machine translation with billion-level corpus
- **Volctrans**: ByteDance neural machine translation engine
- Local SQLite caching of translation results to avoid redundant requests
- Custom proxy server support

### Radial Menu

- `Ctrl + Alt + Right Click` to summon a popup at cursor position
- Contains Clipboard, Quick Phrases, and Translation tabs
- Category filtering, hover-tab-switching, scroll-wheel selection
- Click to paste without opening the main window
- Can be toggled in settings

### System Features

- **Global Hotkey**: Customizable keyboard shortcut to summon the main window, supports Win key combos and Win+V shortcut
- **Mouse Gesture**: `Ctrl + Shift + Right Click` to toggle main window visibility
- **System Tray**: Left-click toggles window, right-click opens menu
- **Five Themes**: Light / Dark Solid / Deep Blue / Dark Translucent / Follow System
- **Auto-start**: Enabled by default, silent startup with `--hidden` flag, minimizes to tray on boot
- **Minimize to Tray**: Closing the window hides to tray instead of exiting
- **Paste Notification**: Toast popup at bottom-right corner on successful paste (toggleable)
- **In-app Updates**: Built-in update detection with one-click download and install
- **Storage Migration**: Move database and image files to a custom directory

## Tech Stack

| Layer | Technology |
|:---:|:---|
| Desktop Framework | [Tauri 2.x](https://tauri.app/) (Rust) |
| Frontend Framework | React 19 + TypeScript |
| Build Tool | [Vite](https://vitejs.dev/) |
| UI Styling | Pure CSS — iOS frosted glass + DWM acrylic backdrop |
| State Management | [Zustand](https://zustand-demo.pmnd.rs/) |
| Local Storage | SQLite (rusqlite, bundled) |
| Internationalization | react-i18next (17 languages) |
| Desktop Capabilities | Global hotkey, mouse hook, clipboard monitor, input simulation, auto-start |

## Download

Go to the [Releases](https://github.com/wwnetboy/ViClip/releases) page to download the latest installer:

| Package | Description |
|:---|:---|
| `ViClip_x.x.x_x64-setup.exe` | NSIS Installer |
| `ViClip_x.x.x_x64_zh-CN.msi` | MSI Installer (Chinese) |

**System Requirements**: Windows 10+

## Usage Guide

### Clipboard

1. Copy any text, image, link, or file — it is automatically recorded to clipboard history
2. Open the main window and switch to the "Clipboard" tab
3. Browse or search history, click any record to paste at the current cursor position

### Quick Phrases

1. Switch to the "Phrases" tab
2. Create a new group (e.g., customer service scripts, code snippets)
3. Add phrases to the group
4. Click a phrase to paste instantly

### Translation

1. Switch to the "Translation" tab
2. Enter or paste the text to translate
3. Select target language and translation engine (AI / Google / Baidu / Youdao / Tencent / Volctrans)
4. For third-party engines, configure the corresponding API key in Preferences

### Tray Menu

Right-click the system tray icon:

- **Preferences** — Open settings panel
- **ViClip Website** — Open project homepage
- **Version** — Show current version
- **Check for Updates** — Check and download new version
- **User Guide** — Open Wiki documentation
- **Restart** — Restart the application
- **Exit** — Completely exit

### Personalization Settings

- **Hotkey**: Customize the global summon hotkey, supports Win+V
- **Theme**: Light / Dark Solid / Deep Blue / Dark Translucent / Follow System
- **Click Mode**: Single-click paste / Double-click paste
- **Auto-start**: Launch on system boot and minimize to tray
- **Minimize to Tray**: Hide to tray instead of exiting when closing the window
- **Paste Notification**: Show Toast notification on successful paste
- **Storage Management**: Configure retention period and migrate storage path

## Development Guide

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) 1.77+
- Windows 10+ (depends on Win32 API)

### Local Development

```bash
cd vi-clip

# Install dependencies
pnpm install

# Start development mode (Vite + Tauri)
pnpm tauri dev

# Frontend-only development
pnpm dev

# Type-check
pnpm build

# Build for production
pnpm tauri build

# Lint
pnpm lint
```

## Project Structure

```
vi-clip/
├── src-web/                # Frontend source
│   ├── components/         # React components
│   │   ├── ImagePreview/   # Image preview window
│   │   ├── RadialMenu/     # Radial popup menu
│   │   ├── SearchInput/    # Search input
│   │   ├── SettingsContent # Preferences panel
│   │   ├── Toast/          # Toast notification
│   │   └── ...
│   ├── pages/              # Pages: Clipboard / Phrase / Translation
│   ├── stores/             # Zustand state management
│   ├── styles/             # CSS styles
│   ├── i18n/               # Internationalization (17 languages)
│   └── types/              # TypeScript types
├── src-tauri/              # Tauri backend (Rust)
│   ├── src/
│   │   ├── lib.rs          # App entry, window creation, command registration
│   │   ├── db.rs           # SQLite CRUD, settings, storage migration
│   │   ├── clipboard.rs    # Clipboard monitor (800ms polling)
│   │   ├── paste.rs        # Paste (text/image/file)
│   │   ├── shortcut.rs     # Global hotkey + mouse hook
│   │   ├── translator.rs   # Translation engines (AI / Google / Baidu / Youdao / Tencent / Volctrans)
│   │   ├── tray.rs         # System tray menu
│   │   └── preview_lock.rs # Image preview aspect ratio lock
│   └── Cargo.toml
├── public/                 # Static assets (fonts, icons, etc.)
└── package.json
```

## Open Source

ViClip is open sourced under the [MIT License](LICENSE), hosted on GitHub.

### Contributing

Contributions are welcome:

- **Report Issues**: Submit bug reports or feature requests via [Issues](https://github.com/wwnetboy/ViClip/issues)
- **Submit Code**: Fork the repo → create a branch → submit a PR
- **Improve Docs**: Help improve the Wiki and translations

### Built With

| Project | Purpose |
|:---|:---|
| [Tauri](https://tauri.app/) | Desktop framework |
| [React](https://react.dev/) | Frontend UI |
| [react-i18next](https://react.i18next.com/) | Internationalization |
| [Zustand](https://zustand-demo.pmnd.rs/) | State management |
| [rusqlite](https://github.com/rusqlite/rusqlite) | SQLite database |

---

If you find this project helpful, please give it a ⭐ Star!
