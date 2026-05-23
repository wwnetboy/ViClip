<div align="right">

English | [中文](./README.md)

</div>

<div align="center">

<img src="vi-clip/public/logo.png" alt="ViClip Logo" width="120">

# ViClip

**Desktop Productivity Tool for Windows**

Clipboard Manager · Quick Phrases · Translation

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%2010+-brightgreen.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-ffc131.svg)
![React](https://img.shields.io/badge/React-19-61dafb.svg)

</div>

---

## Overview

ViClip is a lightweight Windows desktop productivity tool that appears as a floating window and minimizes to the system tray when closed. It integrates three core features: clipboard history management, quick phrases, and translation, helping users improve text processing efficiency in their daily work.

## Features

### 📋 Clipboard Manager
- Automatically records text and image copy history
- Keyword search for quick access to historical content
- One-click paste to the current cursor position
- Configurable retention period with automatic cleanup

### ⚡ Quick Phrases
- Organize common phrases and code snippets by scenario groups
- Customizable groups for flexible content organization
- Click to paste directly without manual copying

### 🌐 Translation
- **AI Translation**: Compatible with OpenAI API format, customizable endpoint and model
- **Built-in Translation**: Free translation service, ready to use out of the box
- Local caching of translation results to avoid redundant requests

### ⚙️ System Features
- Global hotkey to show/hide window
- Window always-on-top display
- Light/Dark theme switching
- Launch at system startup

## Tech Stack

| Layer | Technology |
|:---:|:---|
| Desktop Framework | [Tauri 2.x](https://tauri.app/) (Rust) |
| Frontend Framework | React 19 + TypeScript |
| Build Tool | [Vite](https://vitejs.dev/) |
| UI Styling | Pure CSS (iOS-style frosted glass effect) |
| State Management | [Zustand](https://zustand-demo.pmnd.rs/) |
| Local Storage | SQLite (rusqlite, bundled) |
| Internationalization | react-i18next (Simplified Chinese / English) |

## Download

Go to the [Releases](https://github.com/hu-qi-jia/vi-clip/releases) page to download the latest installer:

| Package | Description |
|:---|:---|
| `ViClip_x64-setup.exe` | NSIS Installer |
| `ViClip_x64_zh-CN.msi` | MSI Installer (Chinese) |

**System Requirements**: Windows 11

## Usage Guide

### Getting Started

1. **Launch the App**: Double-click the desktop icon after installation, the app will appear as a floating window
2. **System Tray**: When you close the window, the app automatically minimizes to the system tray and continues running in the background
3. **Show Window**: Use the global hotkey (configurable in settings) to quickly show/hide the window

### Clipboard Feature

1. Copy any text or image, and the system will automatically record it to clipboard history
2. Click the tray icon or use the hotkey to open the main window
3. Switch to the "Clipboard" tab to browse or search history
4. Click any record to paste it directly to the current cursor position

### Quick Phrases Feature

1. Switch to the "Phrases" tab
2. Click "New Group" to create scenario groups (e.g., customer service scripts, code snippets)
3. Add commonly used phrases to the group
4. When needed, click a phrase to paste it to the current input position

### Translation Feature

1. Switch to the "Translation" tab
2. Enter or paste the text to translate
3. Select translation direction (e.g., Chinese → English)
4. Click the translate button to get results
5. For AI translation, please configure the API endpoint and key in settings

### Personalization Settings

- **Hotkeys**: Customize global hotkeys
- **Theme**: Switch between light and dark themes
- **Launch at Startup**: Enable or disable auto-start on boot
- **Storage Management**: Configure clipboard history retention period

## Development Guide

### Prerequisites

- [Node.js](https://nodejs.org/) (18+ recommended)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/)
- [Tauri CLI](https://tauri.app/)

### Local Development

```bash
# Clone the repository
git clone https://github.com/hu-qi-jia/vi-clip.git
cd vi-clip/vi-clip

# Install dependencies
pnpm install

# Start development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Project Structure

```
vi-clip/
├── src-web/                # Frontend source code
│   ├── components/         # React components
│   ├── pages/              # Page components
│   ├── stores/             # Zustand state management
│   ├── styles/             # CSS style files
│   ├── i18n/               # Internationalization config
│   └── types/              # TypeScript type definitions
├── src-tauri/              # Tauri backend source code
│   ├── src/                # Rust source code
│   └── Cargo.toml          # Rust dependency config
├── public/                 # Static assets
└── package.json            # Frontend dependency config
```

## License

This project is licensed under the [MIT License](LICENSE).

---

<div align="center">

If you find this project helpful, feel free to give it a Star!

</div>
