# ViClip 技术文档

## 项目概述

ViClip 是一款 Windows 桌面效率工具，提供剪贴板历史管理、快捷短语、翻译、图片预览等能力。基于 Tauri 2.x 构建，以系统托盘形式常驻后台，支持全局快捷键和鼠标手势呼出。

- **版本**: 1.0.6
- **平台**: Windows 10+

---

## 技术栈

| 角色 | 选型 |
|------|------|
| 桌面框架 | Tauri 2.11 |
| 前端 | React 19 + TypeScript |
| 状态管理 | Zustand 5 |
| 国际化 | react-i18next |
| 虚拟列表 | react-virtuoso |
| 构建工具 | Vite 8 |
| 样式方案 | 纯 CSS (CSS 变量 + 5 套主题) |
| 后端语言 | Rust (GNU 工具链) |
| 数据库 | SQLite (rusqlite, bundled) |
| HTTP | reqwest |
| 包管理 | pnpm |

---

## 项目结构

```
vi-clip/
├── index.html                     # HTML 入口
├── package.json                   # 前端依赖 (React 19, Zustand 5, ...)
├── vite.config.ts                 # Vite 构建配置
├── tsconfig.json                  # TS 根配置
├── tsconfig.app.json              # 前端 TS 配置
├── tsconfig.node.json             # 构建工具 TS 配置
├── public/
│   ├── logo.png                   # 应用 Logo 源文件
│   └── fonts/                     # 子集化 WOFF2 字体
├── src-web/                       # 前端源码
│   ├── main.tsx                   # 入口 (按窗口类型条件渲染)
│   ├── App.tsx                    # 主窗口布局 (侧边栏 + 面板)
│   ├── utils.ts                   # 工具函数
│   ├── types/index.ts             # 类型定义
│   ├── hooks/useThemeSync.ts      # 主题同步 Hook
│   ├── i18n/                      # 国际化 (zh-CN, en)
│   ├── stores/                    # Zustand 状态管理 (5 个 store)
│   ├── components/                # 通用组件
│   ├── pages/                     # 页面组件 (3 个功能页)
│   └── styles/                    # CSS 样式 (9 个文件)
└── src-tauri/                     # Rust 后端
    ├── Cargo.toml                 # Rust 依赖
    ├── tauri.conf.json            # Tauri 配置
    ├── capabilities/default.json  # 权限配置
    ├── icons/                     # 全平台图标 (51 个文件)
    ├── nsis/                      # NSIS 安装脚本
    └── src/
        ├── main.rs                # Rust 入口
        ├── lib.rs                 # Tauri 启动：窗口创建、插件注册、IPC 注册
        ├── db.rs                  # SQLite 数据库操作
        ├── clipboard.rs           # 剪贴板监控
        ├── paste.rs               # 粘贴模拟
        ├── shortcut.rs            # 全局快捷键与鼠标钩子
        ├── translator.rs          # 翻译引擎 (Google + AI)
        ├── tray.rs                # 系统托盘
        ├── preview_lock.rs        # 图片预览宽高比锁定
        └── updater.rs             # GitHub 更新检测
```

---

## 多窗口架构

应用包含 4 类 Tauri Webview 窗口：

| 窗口 | Label | 尺寸 | 特性 | 触发方式 |
|------|-------|------|------|----------|
| 主窗口 | `main` | 520×600 | 可调大小、无边框、透明背景 | 托盘图标 / 全局快捷键 |
| 径向菜单 | `radial-menu` | 300×420 | 置顶、不显示任务栏、跟随鼠标弹出 | Ctrl+Alt+右键 |
| Toast 通知 | `toast` | 320×80 | 置顶、穿透点击、右下角定位 | 粘贴操作完成 |
| 图片预览 | `image-preview-N` | 动态 | 可钉住置顶、宽高比锁定、支持多窗口 | 点击图片缩略图 |

所有窗口均使用 DWM 亚克力/Mica 背景效果，Win11 使用 Mica Alt 合成，Win10 使用 `SetWindowCompositionAttribute` 模糊效果。

---

## 通信模式

```
┌──────────────────────────────────────────────────────────┐
│ Frontend (React)                  Backend (Rust)         │
│                                                          │
│ invoke("command", args)  ──────►  #[tauri::command]     │
│                                                          │
│ listen("event-name")     ◄──────  app.emit("event", ..) │
└──────────────────────────────────────────────────────────┘
```

### 主要事件

| 事件 | 触发方 | 用途 |
|------|--------|------|
| `clipboard-update` | Rust | 新剪贴板记录通知前端 |
| `clipboard-deleted` | Rust | 记录被删除通知 |
| `phrase-groups-changed` | Rust | 短语组变更通知 |
| `settings-changed` | Rust | 设置变更广播到所有窗口 |
| `toast-show` | Rust | 粘贴成功弹出 Toast 通知 |
| `navigate-panel` | Rust | 托盘"偏好设置"导航到设置面板 |
| `radial-menu-down` | Rust | 通知径向菜单窗口显示并传递主题 |
| `language-changed` | Rust | 语言切换通知所有窗口 |
| `theme-changed` | Rust | 主题切换通知所有窗口 |

---

## 数据库设计

SQLite 数据库，默认路径 `{exe_dir}/data/data.db`，支持用户迁移到自定义目录。

### Schema

```sql
-- 剪贴板记录
CREATE TABLE clipboard_records (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,       -- 'text' | 'image' | 'link' | 'file'
    content TEXT NOT NULL,    -- 文本内容 / 图片相对路径 / 文件路径
    source_app TEXT DEFAULT '',
    created_at TEXT NOT NULL
);

-- 短语分组
CREATE TABLE phrase_groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 短语
CREATE TABLE phrases (
    id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (group_id) REFERENCES phrase_groups(id) ON DELETE CASCADE
);

-- 翻译历史 (作为翻译缓存)
CREATE TABLE translation_history (
    id TEXT PRIMARY KEY,
    source_text TEXT NOT NULL,
    target_text TEXT NOT NULL,
    source_lang TEXT DEFAULT 'auto',
    target_lang TEXT NOT NULL,
    engine TEXT NOT NULL,     -- 'google' | 'ai'
    created_at TEXT NOT NULL
);

-- 设置键值表
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### 索引

- `idx_clipboard_created_at` — 按时间排序
- `idx_clipboard_type` — 按类型筛选
- `idx_translation_created_at` — 翻译历史排序

### PRAGMA 配置

```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-8000;  -- 8MB 缓存
```

### 设置缓存策略

高频读取的设置项（主题、语言、翻译配置等）通过 `SETTINGS_CACHE` 内存缓存，启动时预热。`set_setting` 写入时同步更新缓存并 emit `settings-changed` 事件，避免各窗口重复 IPC 查询。

---

## IPC 命令清单

### 剪贴板

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_clipboard_records` | `search?`, `limit?`, `record_type?` | `ClipboardRecord[]` | 获取记录列表（支持搜索/类型过滤/分页） |
| `delete_clipboard_record` | `id` | — | 删除记录（含图片文件清理） |
| `clear_all_records` | — | — | 清空所有记录和数据表 |
| `get_image_base64` | `path` | `String` | 读取图片完整 Base64 |
| `get_image_thumbnail` | `path`, `max_size` | `String` | 读取/生成图片缩略图 |
| `ensure_thumbnail` | `path` | `String` | 确保缩略图已生成 |

### 短语

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_phrase_groups` | — | `PhraseGroup[]` | 获取所有分组 |
| `create_phrase_group` | `name` | `PhraseGroup` | 创建分组 |
| `update_phrase_group` | `id`, `name` | — | 重命名分组 |
| `delete_phrase_group` | `id` | — | 删除分组（级联删除短语） |
| `get_phrases` | `group_id` | `Phrase[]` | 获取分组下的短语 |
| `create_phrase` | `group_id`, `title`, `content` | `Phrase` | 创建短语 |
| `update_phrase` | `id`, `title`, `content` | — | 编辑短语 |
| `delete_phrase` | `id` | — | 删除短语 |

### 粘贴

| 命令 | 参数 | 说明 |
|------|------|------|
| `paste_text` | `text` | 粘贴文本到前台应用 |
| `paste_image` | `path` | 粘贴图片（含 DIB+PNG 剪贴板） |
| `paste_file` | `path` | 粘贴文件（HDROP 格式） |

### 设置

| 命令 | 参数 | 说明 |
|------|------|------|
| `get_setting` | `key` | 读取单个设置（优先缓存） |
| `get_all_settings` | — | 读取所有设置 |
| `set_setting` | `key`, `value` | 写入单个设置 |
| `set_settings_batch` | `settings: {}` | 批量写入设置 |
| `get_storage_path` | — | 获取当前存储路径 |
| `select_storage_folder` | — | 打开文件夹选择对话框 |

### 翻译

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `translate` | `text`, `target_lang` | `TranslateResponse` | 翻译文本（先查缓存） |
| `get_translation_history` | `limit?` | `TranslationRecord[]` | 获取翻译历史 |
| `clear_translation_history` | — | — | 清空翻译历史 |

### 系统

| 命令 | 参数 | 说明 |
|------|------|------|
| `get_app_info` | — | 返回应用名称、版本、作者 |
| `open_url` | `url` | 打开系统浏览器 |
| `open_file_location` | `path` | 在资源管理器中定位文件 |
| `update_shortcut` | `old_shortcut`, `new_shortcut` | 切换全局快捷键 |
| `set_radial_menu_enabled` | `enabled` | 开关径向菜单 |
| `apply_preview_backdrop` | `window_label` | 为预览窗口应用 DWM 背景 |
| `set_preview_aspect_ratio` | `window_label`, `aspect_ratio` | 锁定预览窗口宽高比 |
| `update_tray_language` | — | 重建托盘菜单（语言切换后） |
| `check_update` | — | 检查 GitHub 最新版本 |
| `download_and_install_update` | `url` | 下载并启动安装包 |

---

## 后端模块详解

### 剪贴板监控 (clipboard.rs)

- **轮询策略**: 800ms 间隔轮询，启动前两次跳过（避免记录启动时的剪贴板状态）
- **变更检测**: Windows `GetClipboardSequenceNumber()` 检测序列号变化
- **粘贴状态互斥**: 检查 `PASTING` 原子标志，粘贴期间跳过监控避免记录自身粘贴内容
- **文本处理**: `read_text()` → URL 检测 → `insert_and_emit`
- **图片处理**:
  - 直接读取 Windows 剪贴板 PNG/DIB 格式（非 plugin 方案，更可靠）
  - 使用 raw clipboard bytes 哈希做稳定去重（比 RGBA 哈希更稳定）
  - PNG 编码 → 保存至 `{storage_dir}/images/{hash}.png`
  - 生成缩略图保存至 `{storage_dir}/images/thumbs/{hash}.png`
- **图片缓存**: RGBA + PNG 数据缓存到 `paste.rs` 的 `IMAGE_CACHE`（LRU 10 条目）
- **去重逻辑**: 1 秒窗口内相同 type+content 视为重复，不插入新记录

### 粘贴模拟 (paste.rs)

- **核心流程**: 保存前台窗口 → 隐藏应用窗口 → 恢复前台 → 等待修饰键释放 → enigo 模拟 Ctrl+V
- **PASTING 标志**: 原子变量防重入，`PasteGuard` RAII 保证异常安全
- **Windows 图片粘贴**: DIB + PNG 双格式写入剪贴板（兼容所有应用）
- **Ctrl/Alt 释放等待**: `GetAsyncKeyState` 轮询（最多 500ms），防止径向菜单手势的物理按键与模拟按键冲突
- **图片缓存**: `IMAGE_CACHE` LRU（10 条目上限，逐出 5 条），避免粘贴时重复解码

### 翻译引擎 (translator.rs)

- **Google 免费接口** (空 API Key): `translate.googleapis.com/translate_a/single?client=gtx`
- **Google Cloud API** (有 API Key): `translation.googleapis.com/language/translate/v2`
- **AI 翻译** (OpenAI 兼容格式): 自定义 URL + API Key + Model
- **缓存策略**: 先查 `translation_history` 表，命中直接返回
- **代理支持**: `translate_proxy` 设置项，代理 client 缓存，设置变更时失效

### 快捷键与钩子 (shortcut.rs)

- **全局快捷键**: Tauri `global-shortcut` 插件（默认 Alt+V）
- **Win+V 钩子**: 低层键盘钩子 `WH_KEYBOARD_LL`，拦截 Win+V 防止打开系统剪贴板
- **鼠标钩子**: `WH_MOUSE_LL` 低层鼠标钩子
  - Ctrl+Shift+右键 → 切换主窗口
  - Ctrl+Alt+右键 → 呼出径向菜单
- **钩子线程**: Windows 要求消息循环，Tauri 主线程自带消息泵，钩子在主线程回调中执行
- **防重入**: `TOGGLING` 原子标志 + `ToggleGuard` RAII

---

## 前端架构

### 状态管理 (5 个 Zustand Store)

| Store | 职责 |
|-------|------|
| `clipboardStore` | 剪贴板记录、缩略图/图片缓存、初始化、搜索、分类、粘贴 |
| `phraseStore` | 短语分组与短语 CRUD、粘贴 |
| `settingsStore` | 全部设置项的读写、开机自启、托盘行为 |
| `translationStore` | 翻译输入/输出状态、loading/error、并发请求去重 |
| `toastStore` | Toast 消息与可见性 |

### 条件渲染 (main.tsx)

```
URL Search Params:
  ?radial=1  → <RadialMenu />      (径向菜单窗口)
  ?toast=1   → <Toast />            (Toast 通知窗口)
  ?preview=1 → <ImagePreview />     (图片预览窗口)
  default    → <App />              (主窗口)
```

### 主窗口布局 (App.tsx)

```
┌──────────────────────────────┐
│  Sidebar (60~130px)  │       │
│  ┌────┐               │       │
│  │ 📋 │ 剪贴板        │ Panel │
│  │ 📝 │ 快捷短语       │ Area  │
│  │ 🌐 │ 翻译          │       │
│  │ ⚙  │ 设置          │       │
│  └────┘               │       │
└──────────────────────────────┘
```

- 侧边栏可拖拽调整宽度（60~130px）
- 设置面板是 Overlay 层，不参与面板路由

### 主题系统

```
data-theme 属性:
  light       — 浅色 (磨砂玻璃)
  dark        — 深色 (磨砂玻璃)
  deep-blue   — 深蓝 (磨砂玻璃)
  dark-solid  — 深色实底
  transparent — 透明 (仅 Win11)

自动模式: matchMedia("prefers-color-scheme: dark") 检测系统主题
```

5 套主题通过 CSS 变量实现，变量定义在 `base.css` 的 `[data-theme="..."]` 选择器中。

---

## 构建与打包

### 开发

```bash
cd vi-clip
pnpm install
pnpm tauri dev
```

### 生产构建

```bash
pnpm tauri build
```

### 安装包配置

- **NSIS**: `currentUser` 安装模式，中英文双语，含更新/降级检测、WiX 卸载兼容
- **WiX**: 备用 MSI 构建路径
- **优化**: `[profile.release]` 配置 `opt-level = "z"`、`lto = true`、`strip = true`、`panic = "abort"`
- **体积**: 字体子集化（44MB → 122KB），安装包总大小约 2.6MB

### 版本号管理

版本号统一在 `tauri.conf.json` 和 `Cargo.toml` 中维护（当前 1.0.6），NSIS 脚本中的 `VERSION` 需手动同步。

### 常见编译问题

#### `dlltool.exe: program not found`

使用 GNU 工具链编译时，`getrandom` 等 crate 需要 MinGW-w64 的 `dlltool.exe`。如果出现此错误，通常是 `target/` 中有旧构建缓存冲突导致：

```bash
cargo clean
cargo build
```

确保 `C:\mingw64\bin` 在系统 PATH 中（包含 `dlltool.exe`）。

#### `link.exe` 与 Git Bash 冲突

从 Git Bash 编译时，MSYS2 的 `link.exe`（coreutils 工具）会优先于 Visual Studio 的 `link.exe`，导致 MSVC 工具链链接失败：

```
error: linking with `link.exe` failed: exit code: 1
  = note: link: extra operand '...'
```

**方案 A（当前使用）**: 切换到 GNU 工具链（MSVC C++ 构建工具非必须）：
```bash
rustup default stable-x86_64-pc-windows-gnu
```

**方案 B**: 如需使用 MSVC 工具链，通过 Visual Studio Installer 安装 "Desktop development with C++" workload。

**前置条件检查**:
| 工具链 | 必需环境 |
|--------|----------|
| GNU (`*-pc-windows-gnu`) | MinGW-w64 (`C:\mingw64\bin` 在 PATH) |
| MSVC (`*-pc-windows-msvc`) | Visual Studio Build Tools (C++ workload)

---

## 数据存储

### 默认路径

```
{exe所在目录}/data/
├── data.db              # SQLite 数据库
├── images/              # 剪贴板图片 (PNG)
│   └── {hash}.png
└── images/thumbs/       # 图片缩略图
    └── {hash}.png
```

### 存储迁移

用户可在设置中切换存储路径。迁移时复制 settings 表到新数据库，旧 DB 中保留 `storage_path` 指向新路径（链式追踪）。**注意：当前版本迁移不复制剪贴板记录和短语数据。**

### 过期清理

- 启动时执行一次 `prune_old_records()`
- 后台线程每 3600 秒执行一次
- 保留时长设置: 1周/1月/3月/6月/1年/永久
- 图片文件清理：删除记录时检查是否仍有其他记录引用该图片（多记录共享同文件），无人引用才删除磁盘文件

---

## 已知问题

1. **存储迁移不复制用户数据** — `migrate_storage` 仅复制设置表，剪贴板记录和短语会丢失
2. **终端/Electron 应用粘贴失败** — `SendInput` 的 `LLMHF_INJECTED` 标志被部分应用拦截
3. **剪贴板来源应用未获取** — `source_app` 字段始终为空
4. **锁竞争风险** — 数据库使用单 Mutex，剪贴板插入、翻译、设置读写竞争同一把锁
5. **监控线程无声崩溃** — Mutex 中毒或线程 panic 时，剪贴板监控静默失效
6. **升级安装不刷新快捷方式图标** — NSIS 安装脚本不重建已有桌面快捷方式

---

*最后更新: 2026-05-27*
