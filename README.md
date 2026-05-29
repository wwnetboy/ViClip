<div align="right">

[中文](./README.md) | [English](./README_EN.md)

</div>

<div align="center">

<img src="vi-clip/public/logo.png" alt="ViClip Logo" width="120">

# ViClip

**轻量级桌面效率套件**

剪贴板管理 · 快捷短语 · 多引擎翻译 · 图片预览

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%2010%2B%20%7C%20macOS%2011%2B-brightgreen.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-ffc131.svg)
![React](https://img.shields.io/badge/React-19-61dafb.svg)

</div>

---

## 项目简介

ViClip 是一款轻量级桌面效率套件，以悬浮窗形式呈现，关闭后自动驻留系统托盘，集剪贴板历史管理、快捷短语、多引擎翻译、快捷弹出窗口和图片预览等功能于一体。

## 主要功能

### 剪切板管理

- 自动记录文本、图片、链接和文件的复制历史
- 内容哈希去重，智能过滤重复记录
- 支持关键词搜索和类型筛选
- 单击/双击粘贴到当前光标位置（可在设置中切换）
- 右键菜单：复制内容、删除记录
- 可设置保留时长（1天/1周/1月/3月/永不过期），自动清理过期记录
- 图片缩略图预览、悬停显示操作按钮

### 图片预览

- 点击图片即可弹出独立预览窗口
- 支持窗口置顶（钉住），方便对照查看
- 可锁定宽高比，调整窗口大小时保持比例
- DWM 亚克力背景效果，与系统风格统一

### 快捷短语

- 按分组管理常用话术、代码片段、模板文本
- 支持分组的创建、重命名、排序和删除
- 点击即粘贴，无需手动复制

### 翻译

- **AI 翻译**：兼容 OpenAI API 格式（支持 /v1/chat/completions），可自定义端点和模型
- **Google 翻译**：内置免费接口，开箱即用；也可配置官方 API Key
- **百度翻译**：专业版 API，适合中文场景
- **有道翻译**：支持多种语言对，国内访问稳定
- **腾讯云 TMT**：腾讯云机器翻译，支持十亿级语料
- **火山翻译**：字节跳动旗下，神经机器翻译引擎
- 翻译结果本地 SQLite 缓存，避免重复请求
- 支持自定义代理服务器

### 简约窗口

- `Ctrl + Alt + 右键` 在鼠标位置唤起快捷弹出窗口
- 包含剪切板、快捷短语、翻译三个标签页
- 支持分类筛选、悬停切换标签、滚轮滚动选择
- 点击即粘贴，无需打开主窗口
- 可在设置中开关

### 系统功能

- **全局快捷键**：可自定义键盘快捷键唤起主窗口，支持 Win 键组合和 Win+V 快捷键
- **鼠标手势**：`Ctrl + Shift + 右键` 切换主窗口显隐
- **系统托盘**：左键切换窗口，右键展开功能菜单
- **五款主题**：浅色 / 暗色 / 深蓝 / 透明 / 跟随系统
- **开机自启**：默认开启，支持静默启动（--hidden 参数），启动后最小化到托盘
- **最小化到托盘**：关闭窗口时隐藏到托盘而非退出
- **粘贴通知**：粘贴成功后在屏幕右下角弹出 Toast 提示，可开关
- **应用内更新**：内置更新检测，支持一键下载安装
- **存储路径迁移**：支持将数据库和图片文件迁移到自定义目录

## 技术栈

| 层级 | 技术选型 |
|:---:|:---|
| 桌面框架 | [Tauri 2.x](https://tauri.app/) (Rust) |
| 前端框架 | React 19 + TypeScript |
| 构建工具 | [Vite](https://vitejs.dev/) |
| UI 样式 | 纯 CSS — iOS 风格磨砂玻璃 + DWM 亚克力背景 |
| 状态管理 | [Zustand](https://zustand-demo.pmnd.rs/) |
| 本地存储 | SQLite (rusqlite, bundled) |
| 国际化 | react-i18next（简体中文 / 繁体中文 / English / 日本語 / Deutsch / Français / Español / Italiano / Português / Русский / 한국어 / ไทย / Tiếng Việt / Bahasa Indonesia / Melayu / हिन्दी） |
| 桌面能力 | 全局热键、鼠标钩子、剪贴板监听、模拟输入、自启动 |

## 下载安装

### macOS (Apple Silicon, macOS 11+)

**方式一：Homebrew Cask（推荐）**
```bash
brew tap wwnetboy/viclip https://github.com/wwnetboy/ViClip
brew install --cask viclip
```

**方式二：手动安装**
1. 前往 [Releases](https://github.com/wwnetboy/ViClip/releases) 下载 `ViClip.app.zip`
2. 解压后将 `ViClip.app` 拖到 `/Applications`
3. 首次打开如提示"已损坏"，终端运行 `xattr -cr /Applications/ViClip.app`
4. 前往「系统设置 → 隐私与安全性 → 辅助功能」添加 ViClip（粘贴功能需要）

### Windows (Windows 10+)

前往 [Releases](https://github.com/wwnetboy/ViClip/releases) 页面下载：

| 安装包 | 说明 |
|:---|:---|
| `ViClip_x.x.x_x64-setup.exe` | NSIS 安装包 |
| `ViClip_x.x.x_x64_zh-CN.msi` | MSI 安装包（中文） |

## 操作说明

### 剪切板

1. 复制任意文本、图片、链接或文件，系统自动记录到剪切板历史
2. 打开主窗口，切换到「剪切板」标签页
3. 浏览或搜索历史记录，点击即可粘贴到当前光标位置

### 快捷短语

1. 切换到「短语」标签页
2. 新建分组（如：客服话术、代码片段）
3. 在分组中添加短语内容
4. 点击短语即可粘贴

### 翻译

1. 切换到「翻译」标签页
2. 输入或粘贴需要翻译的文本
3. 选择目标语言和翻译引擎（AI / Google / 百度 / 有道 / 腾讯 / 火山）
4. 使用第三方引擎需在偏好设置中配置对应的 API 密钥

### 托盘菜单

右键系统托盘图标：

- **偏好设置** — 打开设置面板
- **ViClip官网** — 打开项目主页
- **版本** — 显示当前版本号
- **检测更新** — 检查并下载新版本
- **使用指南** — 打开 Wiki 文档
- **重启** — 重启应用
- **退出** — 完全退出

### 个性化设置

- **快捷键**：自定义全局唤起快捷键，支持 Win+V
- **主题**：浅色 / 暗色 / 深蓝 / 透明 / 跟随系统
- **点击模式**：单击粘贴 / 双击粘贴
- **开机自启**：开机自动启动并最小化到托盘
- **最小化到托盘**：关闭窗口时隐藏到托盘
- **粘贴通知**：粘贴成功时显示 Toast 提示
- **存储管理**：配置保留时长、迁移存储路径

## 开发指南

### 环境准备

- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) 1.77+
- Windows 10+（依赖 Win32 API）

### 本地开发

```bash
cd vi-clip

# 安装依赖
pnpm install

# 启动开发模式（Vite + Tauri）
pnpm tauri dev

# 仅前端开发
pnpm dev

# 类型检查
pnpm build

# 构建生产版本
pnpm tauri build

# 代码检查
pnpm lint
```

## 项目结构

```
vi-clip/
├── src-web/                # 前端源码
│   ├── components/         # React 组件
│   │   ├── ImagePreview/   # 图片预览窗口
│   │   ├── RadialMenu/     # 简约窗口
│   │   ├── SearchInput/    # 搜索输入框
│   │   ├── SettingsContent # 偏好设置面板
│   │   ├── Toast/          # Toast 通知
│   │   └── ...
│   ├── pages/              # 页面：Clipboard / Phrase / Translation
│   ├── stores/             # Zustand 状态管理
│   ├── styles/             # CSS 样式
│   ├── i18n/               # 国际化（17 种语言）
│   └── types/              # TypeScript 类型
├── src-tauri/              # Tauri 后端 (Rust)
│   ├── src/
│   │   ├── lib.rs          # 应用入口、窗口创建、命令注册
│   │   ├── db.rs           # SQLite 数据库 CRUD、设置、存储迁移
│   │   ├── clipboard.rs    # 剪贴板监听（800ms 轮询）
│   │   ├── paste.rs        # 粘贴（文本/图片/文件）
│   │   ├── shortcut.rs     # 全局热键 + 鼠标钩子
│   │   ├── translator.rs   # 翻译引擎（AI / Google / 百度 / 有道 / 腾讯 / 火山）
│   │   ├── tray.rs         # 系统托盘菜单
│   │   └── preview_lock.rs # 图片预览窗口宽高比锁定
│   └── Cargo.toml
├── public/                 # 静态资源（字体、图标等）
└── package.json
```

## 开源

ViClip 采用 [MIT](LICENSE) 开源协议，代码托管在 GitHub。

### 参与贡献

欢迎通过以下方式参与项目：

- **报告问题**：在 [Issues](https://github.com/wwnetboy/ViClip/issues) 中提交 Bug 报告或功能建议
- **提交代码**：Fork 仓库 → 创建分支 → 提交 PR
- **完善文档**：协助完善 Wiki 使用指南和翻译

### 依赖与致谢

ViClip 基于以下开源技术构建：

| 项目 | 用途 |
|:---|:---|
| [Tauri](https://tauri.app/) | 桌面应用框架 |
| [React](https://react.dev/) | 前端 UI |
| [react-i18next](https://react.i18next.com/) | 国际化 |
| [Zustand](https://zustand-demo.pmnd.rs/) | 状态管理 |
| [rusqlite](https://github.com/rusqlite/rusqlite) | SQLite 数据库 |

---

如果这个项目对你有用，欢迎 ⭐ Star 支持！
