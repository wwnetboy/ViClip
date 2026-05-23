# ViClip

PC 端效率辅助工具 —— 剪切板管理、快捷短语、翻译，桌面悬浮窗形态，关闭后驻留系统托盘。

## 功能

- **剪切板管理** — 自动记录文本/图片复制历史，支持搜索和一键粘贴到当前光标位置，可设置保留时长自动清理
- **快捷短语** — 按场景分组管理常用话术/代码片段，点击即粘贴
- **翻译** — 支持 AI 翻译（兼容 OpenAI API 格式，可自定义端点和模型）和内置免费翻译，翻译结果本地缓存
- **系统功能** — 全局快捷键唤起/隐藏、窗口置顶、亮色/暗色主题、开机自启

## 技术栈

| 层 | 选型 |
|---|---|
| 桌面框架 | [Tauri 2.x](https://tauri.app/) (Rust) |
| 前端 | React 19 + TypeScript + Vite |
| UI | 纯 CSS（iOS 风格磨砂玻璃） |
| 状态管理 | [Zustand](https://zustand-demo.pmnd.rs/) |
| 本地存储 | SQLite (rusqlite, bundled) |
| 国际化 | react-i18next（简体中文 / English） |

## 开发

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm tauri dev

# 构建
pnpm tauri build
```

## 许可

MIT
