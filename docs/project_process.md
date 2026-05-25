# ViClip — 开发日志

## 当前阶段

Phase 1 — 核心功能闭环 ✅ (已完成)
Phase 2 — 完善与优化 🔄 (进行中)

## 项目状态

| 维度 | 状态 |
|------|------|
| PRD | 已完成 |
| 产品架构 | 已完成 |
| 技术选型 | Tauri 2.x + React 19 + TypeScript + Zustand + SQLite |
| UI 框架 | iOS 风格纯 CSS + CSS 变量，5 款主题 |
| 项目脚手架 | 已搭建，前后端编译通过 |
| 数据库 | 5 张表已建，CRUD 命令已注册 |
| 翻译引擎 | Google（免费/官方 API）+ AI（OpenAI 兼容）两种引擎 |
| 粘贴模拟 | 已实现，聚焦可靠，闪烁已消除 |
| 图片剪切板 | 已支持监控、存储（PNG 文件）、展示、粘贴、悬浮预览 |
| 剪切板分类 | 文本 / 图片 / 链接 / 文件 四类自动识别 |
| 导航栏 | iOS 风格侧边栏，可拖拽调整宽度（60~130px），折叠/展开 |
| 设置页 | 完整设置面板：基础设置 + 快捷键 + 存储 + 翻译 + 关于 |
| 径向菜单 | Ctrl+Alt+右键长按弹出，hover 检测粘贴 |
| 图片预览 | 独立多窗口预览，钉住置顶，宽高比锁定 |
| Toast 通知 | 粘贴成功右下角弹出提示，可开关 |
| 托盘菜单 | 左键切换窗口，右键完整功能菜单（偏好设置/官网/版本/更新/指南/重启/退出） |
| 窗口管理 | 多窗口架构（main + radial-menu + toast + preview-N） |
| 可运行 | 是（`pnpm tauri dev`） |

## 已完成事项

- [x] PRD 编写
- [x] 产品架构设计
- [x] 技术栈选型与论证
- [x] Rust 环境配置（MSVC 工具链）
- [x] Tauri + React + TypeScript 项目脚手架
- [x] SQLite 数据库建表 + 迁移
- [x] 剪切板监听模块
- [x] 快捷短语 CRUD 全部命令
- [x] 系统托盘（显示/退出菜单）
- [x] 全局快捷键
- [x] i18n 国际化（中/英）
- [x] Google 翻译接入（免费接口 + Cloud Translation API）
- [x] AI 翻译模块（OpenAI 兼容格式）
- [x] Windows/macOS 粘贴模拟（enigo）
- [x] 设置页 Google/AI 双引擎配置
- [x] 设置页语言切换按钮（中/英一键切换）
- [x] UI 改为 iOS 风格 — 无边框透明窗口、磨砂玻璃、纯 CSS 组件
- [x] 前端组件全部去 MUI 化（App + 3 页面 + 设置弹窗）
- [x] 图片剪切板监控 + 存储（RGBA → PNG → 文件，缩略图展示）
- [x] 图片粘贴（PNG 文件 → Image → 剪切板 → Ctrl+V）
- [x] 粘贴闪烁消除（CSS opacity 0 → hide → paste → show → opacity 1）
- [x] 导航栏状态冲突修复（点击设置时主功能按钮取消选中）
- [x] 剪切板/快捷短语卡片重设计（Notification Card 风格：左侧彩色条 + 内容位移动画）
- [x] 剪切板分类功能（文本/图片/链接/文件 四类自动识别 + 分类筛选标签）
- [x] Rust 后端链接/文件类型检测（is_url / is_file_path 函数）
- [x] 图片悬浮预览（hover 400ms 弹出大图，鼠标移开关闭）
- [x] 隐藏横向滚动条（全局 CSS）
- [x] 设置页翻译模块整合
- [x] 设置页存储位置显示 + 自定义文件夹选择（tauri-plugin-dialog）
- [x] 剪切板时间格式改为日期+时分（M/D HH:mm）
- [x] 快捷短语内容在前标题在后，标题小字，左对齐
- [x] 全局快捷键自定义（设置页录制快捷键 + Ctrl+Shift+右键鼠标钩子）
- [x] 过期记录自动清理（启动时 + 每小时定时线程）
- [x] 图片悬浮预览修复（CSS :hover 裁剪问题 → fixed overlay + React state）
- [x] 图片粘贴优化（paste_with_defocus 改为后台线程 hide/paste/show，消除卡顿）
- [x] 短语按钮 100% 不透明度（亮色 #fff / 暗色 #3a3a3c）
- [x] 暗色模式短语卡片左侧竖条颜色修复
- [x] 前端代码重构 — CSS 模块化拆分
- [x] 前端代码重构 — 页面组件拆分
- [x] 前端代码重构 — 设置组件拆分
- [x] 前端代码重构 — 文件夹结构优化
- [x] 前端代码重构 — TypeScript 类型修复
- [x] 径向菜单（Ctrl+Alt+右键长按弹出，hover 检测粘贴）
- [x] 双击粘贴修复（PasteGuard RAII 模式、同步监控器缓存、阻止 WM_RBUTTONUP 传播）
- [x] 径向菜单暗色模式同步
- [x] 快捷键/托盘呼出窗口闪烁修复（过滤 ShortcutState::Pressed / MouseButtonState::Down）
- [x] 关闭按钮和 hide() 权限修复（capabilities 添加 allow-hide）
- [x] Google 翻译 API Key 输入框显示修复（store 默认值与 UI option 同步）
- [x] 过期记录自动删除（启动时调用 + 后台线程每 3600s）
- [x] 图片悬浮放大预览修复（fixed overlay + pointer-events: none）
- [x] 图片粘贴卡顿优化（minimize → hide、后台线程、减少延迟）
- [x] 径向菜单窗口定位修复（移除窗口偏移，popup 100% 填充）
- [x] Win 键快捷键支持（键盘钩子 + GetAsyncKeyState 检测）
- [x] 径向菜单点击模式（clickMode 设置：单击/双击粘贴）
- [x] Toast 粘贴通知（独立窗口，右下角弹出，可开关）
- [x] 开机自启 + 静默启动（tauri-plugin-autostart + --hidden 参数）
- [x] 最小化到托盘（关闭窗口隐藏到托盘，可配置）
- [x] 侧边栏可拖拽调整宽度（60~130px，折叠/展开）
- [x] 多窗口图片预览（WebviewWindowBuilder 动态创建，可同时打开多个）
- [x] 图片预览宽高比锁定（SetWindowSubclass + WM_SIZING 拦截）
- [x] 图片预览钉住（pin）置顶切换
- [x] 主题扩展：浅色 / 深色 / 深蓝 / 深色实底 / 自动（跟随系统）五款主题
- [x] src-web 目录重命名（src → src-web）
- [x] 托盘菜单重构：偏好设置 / ViClip官网 / 版本 / 检测更新 / 使用指南 / 重启 / 退出
- [x] 设置页重构：BasicSettingsSection（主题/点击模式/通知/径向菜单/启动行为合并）

## 粘贴聚焦问题记录

**问题**：点击内容粘贴时，窗口消失/闪烁，且部分应用（浏览器、终端）粘贴不生效。

**根因**：Tauri 浮窗具有键盘焦点时，模拟的 Ctrl+V 会投递到自身窗口，必须先转移焦点到目标应用。转移焦点的不同方式有不同表现：

| 方案 | 效果 | 原因 |
|------|------|------|
| `window.hide()` / `window.show()` | 聚焦可靠 ✓ | Windows 隐藏前台窗口时，系统精确激活上一个焦点窗口 |
| `Alt+Escape`（Z 序推底） | 部分应用失效 ✗ | 激活的是 Z 序下一个窗口，不一定是用户之前使用的应用 |
| `SetForegroundWindow(HWND)` | 不可靠 ✗ | 跨进程前台窗口切换有权限限制，且背景线程追踪 HWND 有 800ms 延迟 |
| `window.minimize()` | 聚焦较可靠 △ | 动画比 hide/show 更平滑但仍有视觉变化 |
| `set_position(-9999,-9999)` | 完全不聚焦 ✗ | 移动窗口不改变焦点 |

**最终方案**：回到 `window.hide()` / `window.show()`（聚焦确定可靠），配合前端 CSS opacity 技巧消除视觉闪烁：

```
点击 → opacity:0 → requestAnimationFrame(等一帧确保重绘) → invoke
→ [后端: write clipboard → hide → sleep 120ms → Ctrl+V → sleep 40ms → show → focus]
→ opacity:1
```

窗口在 hide 之前已经全透明，show 之后才恢复可见。用户看不到 hide/show 过渡，视觉上窗口保持静止。

## 架构一致性审查（2026-05-14）

基于 `ARCHITECTURE.md` 逐项对照审查，发现以下不一致问题（大部分已解决）：

### 已修复

| # | 问题 | 状态 |
|---|------|------|
| 1 | API 凭证硬编码（百度 AppID/Secret） — 已移除 | ✅ |
| 2 | SQL 注入风险（get_clipboard_records 搜索拼接） | ⚠️ 待修复 |
| 3 | 前端未监听 clipboard-update 事件 | ✅ 已实现 |
| 4 | React 版本不一致（文档 18 / 实际 19） | ✅ 文档已更新 |
| 5 | MUI 已安装但未使用 — 依赖已移除 | ✅ |
| 6 | hooks/ 目录缺失 — paste 逻辑在 utils/ 中 | ✅ 结构已确认 |
| 7 | types/index.ts 类型定义不完整 | ✅ 已补全 link/file/google |
| 8 | 剪切板轮询间隔不一致（文档 500ms / 实际 800ms） | ✅ 文档已更新 |
| 9 | 有道翻译未实现 — 实际为 Google 翻译 | ✅ 文档已更新 |
| 10 | 面板路由方式不一致（文档 URL param / 实际 React state） | ✅ 文档已更新 |
| 11 | toggle_always_on_top 未实现 | ✅ 已移除（钉住功能在预览窗口实现） |
| 12 | 托盘图标单击事件未实现 | ✅ 已实现（左键切换窗口） |
| 13 | 过期数据定时清理未启用 | ✅ 已修复 |
| 14 | NavigationButton 死代码 | ✅ 已清理 |
| 15 | 默认快捷键未设置 | ✅ 文档已更新（无默认值） |

## 当前待处理

1. **SQL 注入风险** — `db.rs` 中搜索关键词通过 `format!` 拼接 SQL，需改用参数化查询
2. **剪切板记录来源应用未获取** — `source_app` 字段始终为空
3. **终端 Ctrl+V 兼容性** — SendInput 的 LLMHF_INJECTED 标志被 Electron/Chromium 应用拦截，PostMessageW 方案待测试
4. **快捷键录制后需重启生效** — 更新快捷键后不立即生效
5. **翻译缓存策略单一** — 仅按精确匹配，不支持相似文本复用

## 下一步规划

### Phase 2 — 完善与优化（进行中）

| 任务 | 优先级 | 状态 |
|------|--------|------|
| 修复 SQL 注入风险（参数化查询） | P1 | ⬜ |
| 终端粘贴兼容（PostMessageW 方案测试） | P1 | ⬜ |
| 获取剪切板来源应用名 | P2 | ⬜ |
| 翻译缓存相似文本复用 | P2 | ⬜ |
| 快捷键录制即时生效 | P2 | ⬜ |

### Phase 3 — 发布准备

| 任务 | 优先级 |
|------|--------|
| Windows .msi/.exe 打包测试 | P0 |
| 代码签名配置 | P1 |
| 自动更新（Tauri updater） | P1 |
| 开源准备（LICENSE、README、CONTRIBUTING） | P1 |

## 技术笔记

- Rust 工具链 `stable-x86_64-pc-windows-msvc`
- Tauri 2.x `tray-icon` 需在 Cargo.toml 显式开启 feature
- Tauri 2.x 系统托盘通过 Rust 代码（TrayIconBuilder）创建，tauri.conf.json 中 trayIcon 已废弃
- 数据库路径：默认 Tauri app_data_dir，可迁移至自定义目录
- Google 免费翻译：`translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl=XX&dt=t&q=URL_ENCODED_TEXT`
- enigo 0.3 无需 feature flags，自动根据目标平台选择后端
- iOS 风格窗口：`decorations: false`, `transparent: true`, `shadow: true`
- UI 已完全移除 MUI，使用纯 CSS 变量 + 类名体系
- 剪切板图片处理流程：`read_image()` → `Image` (RGBA) → PNG 编码 → 保存 `app_data/images/{hash}.png` → DB 存路径 → 前端 `get_image_base64` 转 base64 渲染缩略图
- 图片粘贴流程：读取 PNG → 解码 RGBA → `Image::new_owned()` → `write_image()` → Ctrl+V
- 粘贴聚焦唯一可靠方案：`window.hide()` + `window.show()`，配合前端 CSS opacity 消除视觉闪烁
- Notification Card 组件：`isolation: isolate` + `::before` 覆盖层在 WebView2 中会导致内容不可见，改用 `border-left` + `.notibar` 内部元素实现左侧彩色条
- Vite 8 (rolldown) 不支持跨模块 `export type` 导入，需在导入模块本地定义类型
- 剪切板类型检测：文本优先 → `is_url()` 检测 http/https/ftp 开头 → `is_file_path()` 检测盘符路径+存在性 → 兜底 text
- `tauri-plugin-dialog` 用于系统文件夹选择对话框
- `prune_old_records()` 启动时调用一次 + 后台线程每 3600s 执行一次
- 图片悬浮缩放 CSS `:hover` 在祖先容器有 `overflow-y: auto` 时被裁剪，最终方案为 fixed-position overlay + `pointer-events: none`
- 图片粘贴后台线程：将 hide/paste/show 移入 `std::thread::spawn`，command 在 clipboard write 后立即返回
- 前端重构采用单一职责原则：每个组件只负责一个功能
- CSS 模块化策略：按功能域拆分（base/layout/components/clipboard/phrases/translation/settings/radial-menu/image-preview）
- 设置组件拆分：BasicSettings / Language / Shortcut / Storage / Translation / Startup / About

---

## 径向菜单（Radial Menu）开发记录 — 2026-05-16

### 功能概述

径向菜单是一个独立 Tauri 窗口（`radial-menu`），通过 **Ctrl+Alt+右键长按** 触发，在鼠标位置弹出，显示剪切板和快捷短语的内容列表。用户按住右键移动鼠标到目标条目上，松开右键即可将内容粘贴到之前的应用中。

### 架构要点

```
shortcut.rs (WH_MOUSE_LL 低层鼠标钩子)
  ├── Ctrl+Alt+RightButtonDown → show radial-menu window at cursor position
  ├── MouseMove (while right down) → emit "radial-menu-move" (throttled 16ms)
  └── RightButtonUp → emit "radial-menu-up"
        └── 前端 RadialMenu/index.tsx
              ├── 根据坐标做 hover 检测 (document.elementFromPoint)
              ├── Hover 500ms 自动切换 tab/分类 (useHoverSwitch hook)
              └── 调用 pasteRecord/pastePhrase → paste.rs
                    └── paste_with_defocus: hide windows → restore focus → Ctrl+V
```

- 窗口定位：`SetWindowPos(HWND_TOPMOST)` → `SWP_SHOWWINDOW` → `HWND_NOTOPMOST`（瞬时置顶后恢复，避免常驻置顶干扰）
- 坐标转换：`screen_to_css` 函数处理 DPI 缩放（physical → CSS pixels）
- 前端 hover 检测：`document.elementFromPoint` + `closest("[data-radial-item-id]")` / `[data-radial-nav]` / `[data-radial-category]`

### 已解决的问题

#### 1. 窗口位置偏移 / 页面未填充窗口
- **修复**：移除 `calculatePopupPosition` 和 `VIEWPORT_PADDING`，popup 设置 `width: 100%; height: 100%`，移除 `border` 和 `border-radius`

#### 2. 双击粘贴（Double Paste）
- **根因链**：
  1. 剪切板监控器 800ms 轮询，检测到 paste 写入的内容 → 重复记录
  2. `PASTING.swap(false)` 过早清除标志
  3. WM_RBUTTONUP 同时触发 radial-menu-up 和系统右键菜单
- **修复**：
  1. `PasteGuard`（RAII 模式，drop 时重置 PASTING）
  2. 监控器改用 `PASTING.load(Ordering::SeqCst)` 只读
  3. 缓存状态外部化为模块级 `pub static Mutex`，paste 后调用 `sync_monitor_cache()` 同步
  4. WM_RBUTTONUP 返回 `LRESULT(1)` 阻止消息传播

#### 3. 粘贴输出字符 'V' 而非执行粘贴
- **修复**：将 `enigo.key(Key::V, Direction::Click)` 改为 `Press` → 10ms sleep → `Release`

#### 4. 暗色模式同步
- **修复**：每次 `radial-menu-down` 事件中通过 `invoke("get_setting")` 重新读取主题并设置 `data-theme`

### 尚未解决的问题

#### 终端 / Electron 应用粘贴失败

**现状**：enigo 底层调用 Windows `SendInput` API，该 API 设置 `LLMHF_INJECTED` 标志，Chromium/Electron 应用可能检测并忽略合成的键盘输入。

**已尝试的方案**：

| 方案 | 结果 | 原因 |
|------|------|------|
| Ctrl+V (SendInput) | 普通应用 ✅ / 终端 ❌ | SendInput 的 INJECTED 标志被 Electron 拦截 |
| Shift+Insert (SendInput) | 同 Ctrl+V | 同样使用 SendInput，同样被拦截 |
| Ctrl+V + AllowSetForegroundWindow | 焦点恢复改善，但终端仍不工作 | 仍走 SendInput 路径 |
| PostMessageW 直接向目标 HWND 发送 WM_KEYDOWN | **已回滚**（未充分测试） | 绕过 SendInput 和 INJECTED 标志检测，理论可行 |

**下一步方向**：
1. 完成 `PostMessageW` 方案的测试
2. 同时发送 Ctrl+V 和 Shift+Insert 两种按键
3. 使用 Windows UI Automation API
4. 调查 enigo 是否有不使用 `SendInput` 的后端

---

## 图片预览窗口 — 2026-05-23

### 功能概述

点击剪切板中的图片缩略图，弹出独立图片预览窗口。支持同时打开多个预览窗口，每个窗口可独立钉住（置顶）和锁定宽高比。

### 架构要点

```
前端 ClipboardCard.tsx
  └── 点击图片 → invoke store_preview_image(base64) → 获取 token
        └── WebviewWindowBuilder 动态创建 preview-{token} 窗口
              └── ImagePreview/index.tsx
                    ├── 钉住按钮 (toggle_always_on_top)
                    ├── 宽高比锁定 (invoke set_preview_aspect_ratio)
                    └── 关闭按钮 (window.close)
```

- 图片数据通过内存 HashMap 传递（PreviewImageStore），使用 UUID token 索引
- 宽高比锁定通过 `SetWindowSubclass` 拦截 `WM_SIZING` 消息，根据锁定比例自动调整窗口尺寸
- DWM 亚克力背景效果

---

## Toast 粘贴通知 — 2026-05-24

### 功能概述

粘贴成功后屏幕右下角弹出 Toast 提示，告知用户粘贴操作已完成。独立 Tauri 窗口，透明无边框，忽略鼠标事件，2.5 秒后自动消失。

### 架构要点

```
前端 paste 调用 → 成功后 toastStore.show(title, message)
  └── Toast 组件渲染 → setTimeout 2500ms → auto dismiss
        └── toast 窗口 (320×80, always_on_top, set_ignore_cursor_events)
```

- 可在设置中开关（toastEnabled）
- 工作区检测：通过 SPI_GETWORKAREA 定位到右下角任务栏上方
- 动画：CSS fadeIn/fadeOut + translateY 位移

---

## 字体子集化方案 — 2026-05-24

### 背景

项目使用 PingFang（4 个字重）和 SF Pro Display 作为 UI 字体，完整字体文件合计 **44MB**，占据安装包的 77%。通过字符子集化 + WOFF2 压缩，字体总大小降至 **122KB**（减少 99.7%）。

### 工作流

每次 UI 文本有新增或修改后，需重新生成子集字体：

```bash
cd vi-clip

# Step 1: 提取源码中所有唯一字符
find src-web -type f \( -name "*.tsx" -o -name "*.ts" -o -name "*.json" -o -name "*.css" \) \
  -exec cat {} \; | grep -oP '[\x{4e00}-\x{9fff}\x{3000}-\x{303f}\x{ff00}-\x{ffef}\x{0021}-\x{007e}]' \
  | sort -u | tr -d '\n' > /tmp/viclip_chars.txt

# Step 2: 子集化 PingFang 四个字重（WOFF2 格式）
for weight in "Regular" "Medium" "Semibold" "Light"; do
  pyftsubset ".fonts-src/PingFang $weight.ttf" \
    --text-file=/tmp/viclip_chars.txt \
    --flavor=woff2 \
    --output-file="public/fonts/PingFang-$weight.woff2" \
    --layout-features='' \
    --no-subset-tables+=GSUB,GPOS \
    --no-ignore-missing-unicodes \
    --passthrough-tables
done

# Step 3: 子集化 SF Pro Display
pyftsubset ".fonts-src/SF-Pro-Display-Black.otf" \
  --text-file=/tmp/viclip_chars.txt \
  --flavor=woff2 \
  --output-file="public/fonts/SF-Pro-Display-Black.woff2" \
  --layout-features='' \
  --no-subset-tables+=GSUB,GPOS \
  --no-ignore-missing-unicodes \
  --passthrough-tables

# Step 4: 验证
ls -la public/fonts/
```

### 目录结构

```
public/
├── fonts-src/          ← 完整原始字体（不参与构建，仅用于子集化）
│   ├── PingFang Light.ttf
│   ├── PingFang Medium.ttf
│   ├── PingFang Regular.ttf
│   ├── PingFang Semibold.ttf
│   └── SF-Pro-Display-Black.otf
└── fonts/              ← 子集化 WOFF2（参与构建）
    ├── PingFang-Light.woff2
    ├── PingFang-Medium.woff2
    ├── PingFang-Regular.woff2
    ├── PingFang-Semibold.woff2
    └── SF-Pro-Display-Black.woff2
```

### 依赖工具

- [fonttools](https://github.com/fonttools/fonttools) — `pip install fonttools brotli`
- `pyftsubset` 命令随 fonttools 安装

### 注意事项

- `fonts-src/` 存放完整字体源文件，需加入 `.gitignore`（文件过大），或以 Git LFS 管理
- `fonts/` 存放子集化结果，需纳入版本管理
- 子集化仅覆盖 **源码中的静态文本**（UI 标签、设置项、i18n 字符串）。用户输入内容（剪贴板、翻译文本）中的生僻字由 CSS `font-family` fallback 到系统字体
- 正则 `\x{4e00}-\x{9fff}` 覆盖 CJK 统一表意文字，`\x{3000}-\x{303f}` 覆盖 CJK 标点，`\x{ff00}-\x{ffef}` 覆盖全角字符，`\x{0021}-\x{007e}` 覆盖 ASCII 可打印字符

---

## 安装包体积优化记录 — 2026-05-24

### 优化前 vs 优化后

| 优化项 | 优化前 | 优化后 | 节省 |
|--------|--------|--------|------|
| 字体子集化 + WOFF2 | 44 MB | 122 KB | -99.7% |
| `tokio` features `full` → `rt, sync` | - | - | ~3 MB |
| `reqwest` native-tls → rustls-tls | - | - | ~2 MB |
| `[profile.release]` 优化 (lto/opt=z/strip) | - | - | ~5 MB |
| `debug_log!` 宏 gated by `#[cfg(debug_assertions)]` | - | - | 运行时 IO |
| `React.StrictMode` 移除 | - | - | 运行时开销 |
| **NSIS 安装包总大小** | **33 MB** | **2.6 MB** | **-92%** |
| **MSI 安装包总大小** | **36 MB** | **3.3 MB** | **-91%** |

### `[profile.release]` 配置

```toml
[profile.release]
opt-level = "z"       # 体积优先
lto = true            # 链接时优化
codegen-units = 1     # 单一代码生成单元，最大化内联
strip = true          # 剥离调试符号
panic = "abort"       # panic 时直接终止，移除 unwind 代码
```

---

---

## 性能优化审查 — 2026-05-25

基于全代码库审查（Rust 后端 + TypeScript/CSS 前端），识别出以下性能优化点。每个优化点都附带了影响分析和风险评估。

### 关于 GitHub 语言统计的说明

当前统计：TypeScript 38.9% / Rust 35.7% / CSS 25.1%。**Rust 行数不是最多，但承担了全部性能关键路径**（剪贴板监控、图片处理、全局钩子、数据库、粘贴模拟、翻译请求）。UI 层代码天然冗长（React 组件 + 5 套主题 CSS + 双语 i18n），Rust 代码密度高，几行就能干 TypeScript 几十行的活。这是 Tauri 应用的标准比例，不代表性能瓶颈在 Rust 端。

---

### 性能优化 TODO 清单

每项包含：优化内容 → 影响分析 → 风险评估 → 建议

---

#### P0 — 高优先级（建议尽快处理）

##### 1. 列表虚拟化 — 2000 条记录全量渲染 DOM

| 维度 | 内容 |
|------|------|
| **文件** | `src-web/pages/ClipboardPage/index.tsx:125` |
| **现状** | `filtered.map(...)` 渲染全部匹配记录为 `<ClipboardCard>`，2000 条 = 10,000+ DOM 节点 |
| **影响** | 每次打开窗口、切换分类、搜索、滚动都产生可见卡顿 |
| **优化方案** | 引入 `react-virtuoso`（~5KB gzipped），仅渲染可视区域 ~15 张卡片 |
| **UI 风险** | ⚠️ **中等** — 虚拟列表会改变 DOM 结构，可能影响：卡片悬浮预览的 `fixed` 定位、`IntersectionObserver` 的缩略图懒加载（`ImageThumb`）、分类筛选标签栏的 `sticky` 定位 |
| **功能风险** | ⚠️ **中等** — 粘贴/删除操作依赖 `record.id`，虚拟列表需要确保 key 稳定；搜索过滤逻辑需要适配（`filtered` 作为 data source 传给 virtuoso 即可） |
| **回滚难度** | 低 — 移除 `virtuoso` 组件，恢复原有 `.map()` 即可 |
| **建议** | 先引入 virtuoso，在 `ClipboardPage` 中逐步替换 `.clipboard-list` 容器，保留 `ImageThumb` 的懒加载逻辑（virtuoso 自带 `visibilityState` 可替代 IntersectionObserver） |

##### 2. 图片复制时文本/文件处理器也白跑

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/clipboard.rs:688-690` |
| **现状** | 当 `image_recorded` 为 false 且 `image_is_same` 为 false 时（即剪贴板序列号变了但非图片），代码无条件调用 `handle_monitor_text` + `handle_monitor_files`。但如果是图片变化触发的，`read_clipboard_image_raw` 已经在前面读过了，这里又读一次文本和文件列表 |
| **优化方案** | 在图片处理分支（`image_recorded == true`）中已经更新了 `LAST_CLIPBOARD_TEXT` 和 `LAST_CLIPBOARD_FILES_KEY` 缓存，不需要再走到 `handle_monitor_text`。问题在于：当图片变化但 `image_recorded == false`（比如图片相同）时，文本/文件缓存已在上面更新过了，走到 else 分支是多余的 |
| **UI 风险** | ✅ 无 — 纯后端逻辑变更 |
| **功能风险** | ✅ 低 — 核心逻辑不变，只是减少不必要的跨进程剪贴板读取。需确保 `LAST_CLIPBOARD_TEXT` 缓存在图片复制时也被正确同步（目前已在 `image_recorded` 分支中更新） |
| **回滚难度** | 低 — 恢复原有 else 分支即可 |
| **建议** | 重构轮询回调中的分支逻辑，将图片/文本/文件的检测顺序优化，添加 early return 避免 fallthrough |

##### 3. 代理模式下每次翻译新建 HTTP Client

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/translator.rs:175-182` |
| **现状** | 当 `proxy_url` 非空时，每次 `translate()` 调用都 `reqwest::Client::builder().build()`，创建新的 TLS 连接 |
| **优化方案** | 将代理 client 也缓存到 `OnceLock<reqwest::Client>` 中。当用户修改代理设置时，清除缓存强制重建。可以用 `AtomicBool` + `OnceLock` 组合，或者直接用 `std::sync::Mutex<Option<Client>>` |
| **UI 风险** | ✅ 无 — 纯后端变更 |
| **功能风险** | ⚠️ **低** — 用户修改代理设置后需要即时生效。当前每次重建虽然慢但保证了即时性。缓存后需要在设置变更时主动失效。可在 `update_setting` 命令中加一个判断：如果 key 是 `translate_proxy`，清除缓存的 client |
| **回滚难度** | 低 |
| **建议** | 用 `Mutex<Option<(String, Client)>>` 缓存，key 为 proxy_url，变更时失效 |

---

#### P1 — 中优先级（建议本阶段处理）

##### 4. 图片缩略图生成重复编解码

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/clipboard.rs:473,495-497` |
| **现状** | `save_monitor_image` 已有 RGBA 像素缓冲区 `rgba_vec`，却先编码为 PNG（line 473），再 `image::load_from_memory(&png_bytes)` 解码回来（line 495）生成缩略图 |
| **优化方案** | 直接从 RGBA 数据构造 `DynamicImage`：`DynamicImage::ImageRgba8(ImageBuffer::from_raw(w, h, &rgba_vec).unwrap())`，跳过 PNG 编解码 |
| **UI 风险** | ✅ 无 — 缩略图输出格式不变（仍是 PNG），只是内部处理路径改变 |
| **功能风险** | ⚠️ **低** — `ImageBuffer::from_raw` 要求 RGBA 数据是连续内存且尺寸匹配（`w * h * 4 == rgba_vec.len()`），这个条件已经满足。需注意 `from_raw` 接管所有权后 `rgba_vec` 不能再用于 PNG 编码，需调整代码顺序：先编码 PNG 写入磁盘，再复用 RGBA 数据生成缩略图 |
| **回滚难度** | 低 |
| **建议** | 调整 `save_monitor_image` 中代码顺序，让 PNG 编码和缩略图生成都使用 `rgba_vec` |

##### 5. DB 单 Mutex 连接竞争

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/db.rs:8` (`Mutex<Connection>`) |
| **现状** | 整个应用只有一把 DB 锁。剪贴板插入、翻译读写、设置读取、缩略图查询全部竞争这把锁 |
| **优化方案** | 方案 A：使用 `r2d2-sqlite` 连接池（3-5 个连接），允许并发读。方案 B：设置表用 `OnceLock` 缓存到内存，减少 DB 读取频率 |
| **UI 风险** | ✅ 无 |
| **功能风险** | ⚠️ **中等** — SQLite 默认支持多线程但需要开启 WAL 模式 + `PRAGMA busy_timeout`。当前代码隐式依赖单连接的串行化保证。改用连接池需要确保：剪贴板插入和清理任务不冲突、翻译缓存写入不丢数据。建议先用方案 B 减少读取频率，连接池留到 Phase 3 |
| **回滚难度** | 高（连接池）/ 低（设置缓存） |
| **建议** | 先用方案 B — 将高频读取的设置项（`google_api_key`, `ai_api_url`, `ai_api_key`, `ai_model`, `translate_proxy`, `theme`, `language` 等）缓存到 `OnceLock<Mutex<HashMap>>` ，`update_setting` 时间步更新缓存 |

##### 6. `get_clipboard_records` 不做服务端类型过滤

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/db.rs:237-293` |
| **现状** | 前端筛选"仅图片"时，后端仍返回最多 2000 条全部记录，前端再做 `filter(r => r.type === 'image')` |
| **优化方案** | 给 `get_clipboard_records` 加一个可选的 `record_type` 参数，SQL WHERE 子句加 `AND type = ?`，让 SQLite 做过滤 |
| **UI 风险** | ✅ 无 — 前端接口不变（向后兼容，不传则返回全部） |
| **功能风险** | ✅ 极低 — 只是把前端的 `.filter()` 逻辑移到 SQL 层，结果应该一致。需确认 `type` 字段的枚举值和前端一致（`text`, `image`, `link`, `file`） |
| **回滚难度** | 低 |
| **建议** | 同时给 `type` 列建索引（目前没有），配合参数化查询一起做 |

##### 7. 图片去重哈希函数太弱

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/clipboard.rs:8-10` |
| **现状** | `hash_bytes` 使用简单多项式 `acc * 31 + b`，对于 RGBA 图片（百万级字节），`wrapping_mul` 导致大量信息丢失，碰撞概率不低 |
| **优化方案** | 改用 `std::hash::DefaultHasher` 或引入 `xxhash-rust`（~2KB，为图片/大文件优化） |
| **UI 风险** | ✅ 无 |
| **功能风险** | ⚠️ **中等** — **变更哈希算法会导致新老图片的哈希值不一致**。已有图片文件是用旧算法命名的（如 `abc123.png`）。如果改用新算法，同一张图会产生不同的哈希值，导致：① 旧图片无法去重（磁盘浪费）；② 同一个 `images/` 目录下同时存在新旧两种哈希命名的文件。**需要写迁移脚本**或维持双算法兼容 |
| **回滚难度** | 高 — 已有图片文件已按旧哈希命名 |
| **建议** | 推迟到 Phase 3。实施时需兼容策略：新图片用新哈希存储，旧文件保留不动；或者在启动时扫描 `images/` 目录，对旧文件重新哈希并重命名 |

##### 8. 图片缓存最大 300MB

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/paste.rs:21-67` |
| **现状** | `ImageCache` 最多存 30 张，每张存 RGBA（~8MB）+ PNG（~1-3MB），30 张截图 ≈ 300MB |
| **优化方案** | 方案 A：只缓存 PNG 编码后的数据（粘贴时才解码 RGBA，多一次解码但省内存）。方案 B：缩小上限到 10 张，超出时 LRU 逐出 5 张而非 15 张 |
| **UI 风险** | ✅ 无 |
| **功能风险** | ⚠️ **低** — 方案 A 会在粘贴时增加一次 PNG 解码（~50ms for 4K），但极少用户连续粘贴 30 张不同的图。方案 B 减少逐出粒度，降低缓存抖动 |
| **回滚难度** | 低 |
| **建议** | 采用方案 B（缩小缓存 + 减小逐出粒度），简单安全 |

##### 9. 切标签页时数据重新加载

| 维度 | 内容 |
|------|------|
| **文件** | `src-web/App.tsx:207` 及 `src-web/stores/clipboardStore.ts:69-71` |
| **现状** | `PANEL_MAP` 在模块顶层创建 React 元素，每次切面板时组件卸载/重新挂载，`ClipboardPage.init()` → `loadRecords()` 重新拉取 2000 条 + 重注册事件监听器 |
| **优化方案** | 方案 A：不使用 `PANEL_MAP`，改为条件渲染（`{panel === 'clipboard' && <ClipboardPage />}`）+ 用 CSS `display: none` 保持非活动面板的 DOM（不卸载）。方案 B：用 `React.memo` + 状态提升，让面板组件只在第一次挂载时 `init()`，后续切换不重新初始化 |
| **UI 风险** | ⚠️ **低** — 方案 A 保持三个面板同时挂载，会增加内存占用（但每个面板的 DOM 不复杂）。方案 B 改动最小 |
| **功能风险** | ⚠️ **中等** — 事件监听器（`clipboard-update`）如果重复注册会导致重复处理。需要确保 `init()` 中的 `listen()` 调用幂等（检查是否已注册） |
| **回滚难度** | 低 |
| **建议** | 方案 B — 在 store 中加 `_initialized: boolean` 标志，`init()` 检查后跳过重复初始化 |

##### 10. 图片预览窗口无复用，无限创建

| 维度 | 内容 |
|------|------|
| **文件** | `src-web/pages/ClipboardPage/ClipboardCard.tsx:50-70` |
| **现状** | 每次点"预览"都 `new WebviewWindowBuilder()` 创建新窗口（~50-100MB/个），不限制数量 |
| **优化方案** | 方案 A：单例预览窗口 — 再次点击时复用已有窗口，只替换内容。方案 B：限制最多 3 个预览窗口 |
| **UI 风险** | ⚠️ **中等** — 方案 A 会改变用户行为：当前是多窗口模式（可并排比较图片），改为单例会失去这个能力 |
| **功能风险** | ⚠️ **低** — 复用窗口需要更新 `PreviewImageStore` 中的图片数据，`token` 管理需要调整 |
| **回滚难度** | 中等 |
| **建议** | 方案 B — 限制 3 个上限，超出时关闭最早的。保留多窗口比较能力，控制内存上限 |

##### 11. 径向菜单每次弹出重新加载设置

| 维度 | 内容 |
|------|------|
| **文件** | `src-web/components/RadialMenu/index.tsx:124` |
| **现状** | 每次 `radial-menu-down` 事件触发时调用 `useSettingsStore.getState().loadSettings()`，走 IPC + DB |
| **优化方案** | 缓存设置到 `tauri://window` 事件传递，或让 main 窗口在设置变更时通过 Tauri event 广播给 radial-menu/ toast 窗口 |
| **UI 风险** | ✅ 无 |
| **功能风险** | ⚠️ **低** — 需要确保设置同步的及时性。如果用户改了主题后立即打开径向菜单，菜单应该反映最新主题。事件广播延迟 < 1ms，可忽略 |
| **回滚难度** | 低 |
| **建议** | 在 Rust 的 `update_setting` 命令中，写 DB 后 emit 一个 `settings-changed` 事件，radial-menu 窗口监听该事件更新本地 store。去掉 `radial-menu-down` 中的 `loadSettings()` 调用 |

##### 12. 粘贴前等待按键释放的忙循环

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/paste.rs:157-173` |
| **现状** | 发送 Ctrl+V 前，`GetAsyncKeyState` 轮询检测 Ctrl/Alt 释放，sleep 10ms 循环，最多 500ms |
| **优化方案** | 改用 `GetKeyState` 或注册一次性的键盘钩子等待按键释放事件，避免忙轮询 |
| **UI 风险** | ✅ 无 |
| **功能风险** | ⚠️ **中等** — 这段忙循环是为了解决"粘贴输出字符 V 而非执行粘贴"的问题。改用事件驱动需要确保时序正确：必须在 Ctrl/Alt 完全释放后才发送 Ctrl+V，否则目标应用可能收到裸 'V' |
| **回滚难度** | 中等 |
| **建议** | 这是功能性代码，优先保证粘贴可靠性。如果要优化，用 `SetWindowsHookEx(WH_KEYBOARD_LL)` 注册一次性钩子等待 key up 事件，但这会增加复杂度。**暂缓处理**，等粘贴功能完全稳定后再考虑 |

---

#### P2 — 低优先级（可推迟到 Phase 3）

##### 13. `panic = "unwind"` → `panic = "abort"`

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/Cargo.toml:54` |
| **现状** | release profile 写的是 `panic = "unwind"`，与 `project_process.md` 中记录的 `panic = "abort"` 不一致（日志记录有误，实际未改） |
| **优化方案** | 改为 `panic = "abort"`，减少二进制体积 ~200KB，消除 unwinding 代码 |
| **UI 风险** | ✅ 无 |
| **功能风险** | ⚠️ **极低** — `abort` 意味着 panic 时直接终止进程而非 unwind 清理资源。对用户来说，panic 发生时 app 直接崩溃 vs. 可能留下临时文件后崩溃，差异极小。Tauri 官方模板默认用 `abort` |
| **回滚难度** | 极低 — 改一行 |
| **建议** | 直接改，顺带修正 `project_process.md` 中不准确的记录 |

##### 14. Zustand 缓存更新产生 GC 压力

| 维度 | 内容 |
|------|------|
| **文件** | `src-web/stores/clipboardStore.ts:159,177` |
| **现状** | `set({ thumbnailCache: { ...get().thumbnailCache, [id]: url } })` 每次缩略图加载完成都创建新对象 |
| **优化方案** | 使用 zustand 的 `immer` 中间件，或使用 `Map` 代替普通对象，或者批量更新（收集 100ms 内的加载结果一起 set） |
| **UI 风险** | ✅ 无 — immer 中间件透明代理，不改变使用方式 |
| **功能风险** | ⚠️ **极低** — immer 中间件成熟稳定，但需要额外安装 `immer` 依赖（~6KB gzipped） |
| **回滚难度** | 低 |
| **建议** | 低收益（30 个缩略图的对象拷贝在现代 JS 引擎中微不足道），可选做 |

##### 15. 翻译请求无取消机制

| 维度 | 内容 |
|------|------|
| **文件** | `src-web/stores/translationStore.ts:34-49` |
| **现状** | 用户快速修改翻译文本时，前一个 `translate()` 的结果可能后到达，短暂显示旧结果 |
| **优化方案** | 使用递增的 `requestId`，回调中检查是否仍是最新请求，舍弃过期结果 |
| **UI 风险** | ✅ 无 — 只影响状态更新逻辑 |
| **功能风险** | ✅ 极低 — 纯前端逻辑，Tauri `invoke()` 不支持 abort，但可以用 ID 比对舍弃过期响应 |
| **回滚难度** | 低 |
| **建议** | 在 store 中加 `_latestRequestId: number`，`translate()` 前递增，回调中比对，不匹配则跳过 `set()` |

##### 16. `get_storage_dir` 每次调用都执行 `create_dir_all`

| 维度 | 内容 |
|------|------|
| **文件** | `src-tauri/src/db.rs:122` |
| **现状** | `get_storage_dir` 在自定义路径已存在的情况下，每次仍判断 `create_dir_all().is_ok()`，这会触发一次内核态系统调用 |
| **优化方案** | 用 `OnceLock<PathBuf>` 缓存结果，启动时确定好路径后不再变 |
| **UI 风险** | ✅ 无 |
| **功能风险** | ⚠️ **低** — 存储路径在运行时不可变（用户需重启才能切换），缓存安全 |
| **回滚难度** | 低 |
| **建议** | 用 `OnceLock` 缓存 `get_storage_dir` 结果 |

##### 17. 图片数据多次 IPC 往返

| 维度 | 内容 |
|------|------|
| **文件** | `src-web/pages/ClipboardPage/ClipboardCard.tsx:50` |
| **现状** | 预览图片流程：`get_image_base64` (Rust 读文件→base64→IPC)→前端→`store_preview_image` (IPC→Rust 存内存)→预览窗口 `fetch_preview_image` (IPC←Rust)。两次完整 IPC + base64 编解码 |
| **优化方案** | 直接传文件路径给预览窗口，用 `convertFileSrc()` 或 asset protocol 加载 |
| **UI 风险** | ✅ 无 — 预览效果不变 |
| **功能风险** | ⚠️ **低** — 需要确保预览窗口能访问 `images/` 目录的文件（asset scope）。Tauri 2.x 的 `convertFileSrc` 需要配置 `assetProtocol` scope |
| **回滚难度** | 低 |
| **建议** | 配合预览窗口复用一起改 |

---

### 优先级汇总

| # | 优化项 | 优先级 | 预计收益 | 风险等级 | 建议动作 |
|---|--------|--------|----------|----------|----------|
| 1 | 列表虚拟化 | P0 | 高 — 消除滚动/搜索卡顿 | 中 | 本阶段实施 |
| 2 | 图片复制多余处理器 | P0 | 中 — 减少轮询开销 | 低 | 本阶段实施 |
| 3 | 翻译代理 HTTP Client 复用 | P0 | 中 — 减少翻译延迟 | 低 | 本阶段实施 |
| 4 | 缩略图跳过重复编解码 | P1 | 中 — 减少图片处理 CPU | 低 | 本阶段实施 |
| 5 | 设置内存缓存 | P1 | 中 — 减少 DB 锁竞争 | 低 | 本阶段实施 |
| 6 | DB 类型过滤 + 索引 | P1 | 中 — 减少 IPC 传输量 | 极低 | 本阶段实施 |
| 7 | 图片哈希算法升级 | P1 | 中 — 去重可靠性 | 中 | 推迟 Phase 3 |
| 8 | 图片缓存缩小 | P1 | 中 — 减少内存占用 | 低 | 本阶段实施 |
| 9 | 面板切换避免重新初始化 | P1 | 中 — 减少不必要 IPC | 中 | 本阶段实施 |
| 10 | 预览窗口数量限制 | P1 | 中 — 控制内存上限 | 中 | 本阶段实施 |
| 11 | 径向菜单设置同步优化 | P1 | 低 — 减少弹出延迟 | 低 | 本阶段实施 |
| 12 | 粘贴按键释放忙循环 | P1 | 低 — 省电 | 中 | 推迟，粘贴稳定后再改 |
| 13 | `panic = "abort"` | P2 | 低 — 减 ~200KB | 极低 | 随时可改 |
| 14 | Zustand immer 中间件 | P2 | 低 — 减少 GC | 极低 | 可选 |
| 15 | 翻译请求 ID 去重 | P2 | 低 — UX 改善 | 极低 | 可选 |
| 16 | `get_storage_dir` 缓存 | P2 | 低 — 减少 syscall | 低 | 本阶段实施 |
| 17 | 图片预览 IPC 简化 | P2 | 中 — 减少预览延迟 | 低 | 配合 #10 一起改 |

---

---

## 性能优化实施记录 — 2026-05-25

基于审查清单，分批完成了 P0（3项）+ P1（6项）+ P2（3项）共 12 项优化。

### P0 实施（高优先级）

| # | 优化项 | 文件 | 改动 |
|---|--------|------|------|
| 1 | 列表虚拟化 | `ClipboardPage/index.tsx` | 引入 `react-virtuoso`，`<Virtuoso>` 替代 `.map()`，DOM 节点从 10,000+ 降至 ~200 |
| 2 | 缩略图免重复编解码 | `clipboard.rs:495` | RGBA 直构 `DynamicImage` 生成缩略图，跳过 PNG encode→decode |
| 3 | 代理 HTTP Client 复用 | `translator.rs` + `db.rs` | `get_proxy_client()` 缓存代理 client，设置变更时失效 |

### P1 实施（中优先级）

| # | 优化项 | 文件 | 改动 |
|---|--------|------|------|
| 4 | 设置表内存缓存 | `db.rs` | `SETTINGS_CACHE` + `warm_settings_cache()` 启动预热，`get_setting`/`get_all_settings` 走缓存，translator 不再直接查 DB |
| 5 | DB 类型过滤+索引 | `db.rs` + `clipboardStore.ts` | SCHEMA 加 `idx_clipboard_type`，`get_clipboard_records` 加 `record_type` 参数，前端传 category |
| 6 | 图片缓存缩小 | `paste.rs` | 30→10 上限，15→5 逐出粒度 |
| 7 | 面板切换免重初始化 | — | 已有 `initialized` 守卫，无需改动 |
| 8 | 预览窗口数量限制 | `ClipboardCard.tsx` | 最多 3 个，超出关闭最早的，stale 条目自动清理 |
| 9 | 径向菜单设置同步 | `db.rs` + `RadialMenu/index.tsx` | `set_setting` emit `settings-changed` 事件，RadialMenu 监听事件替代每显 IPC |

### P2 实施（低优先级）

| # | 优化项 | 文件 | 改动 |
|---|--------|------|------|
| 10 | `panic = "abort"` | `Cargo.toml` | 减小二进制体积 ~200KB |
| 11 | `get_storage_dir` 缓存 | `db.rs` | `STORAGE_DIR_CACHE` 缓存解析结果，`migrate_storage` 时失效 |
| 12 | 翻译请求 ID 去重 | `translationStore.ts` | `nextRequestId` 递增，回调比对，丢弃过期结果 |
| 13 | 图片预览 IPC 简化 | `ClipboardCard.tsx` + `ImagePreview/index.tsx` + `lib.rs` | URL 直传 path 替代 token→base64→IPC 往返，删除 `PreviewImageStore`、`store_preview_image`、`fetch_preview_image` |

### 代码清理

- 移除 `lib.rs` 中 `PreviewImageStore` 结构体及相关命令（`store_preview_image`、`fetch_preview_image`），连带清理 `HashMap`、`Mutex`、`Uuid` 三个不再需要的 import

### 遗留项（推迟到 Phase 3）

- 图片去重哈希算法升级（需迁移旧文件）
- DB 连接池（需 WAL + busy_timeout 配合）
- 粘贴按键释放忙循环优化（需确保粘贴可靠性）

---

*最后更新：2026-05-25*
