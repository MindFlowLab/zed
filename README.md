# Zed 汉化版 (ZH_CN)

[![Zed](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/zed-industries/zed/main/assets/badge/v0.json)](https://zed.dev)
[![CI](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml/badge.svg)](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml)

本仓库是 [Zed](https://github.com/zed-industries/zed) 的**简体中文汉化分支**，在保持与上游同步的基础上，提供完整的中文界面翻译。

Zed 是一款高性能、多人协作代码编辑器，由 [Atom](https://github.com/atom/atom) 和 [Tree-sitter](https://github.com/tree-sitter/tree-sitter) 的创建者开发。

---

## 汉化说明

### 覆盖范围

汉化通过 `crates/zed_i18n` 实现，当前已翻译 **52 个模块**的界面文本，涵盖：

- 编辑器核心 UI（命令面板、文件查找、诊断、状态栏等）
- AI / Agent 面板（agent_ui、assistant 等）
- 调试器界面（debugger_ui、debugger_tools）
- Git 相关（git_ui、collab_ui）
- 设置界面（settings_ui、keymap_editor）
- 扩展管理、反馈、自动更新等

### 使用方式

在 Zed 设置中添加：

```json
{
  "ui_language": "zh-CN"
}
```

即可切换为中文界面。设为 `"en"` 或不配置则使用英文。

### 翻译文件结构

```
crates/zed_i18n/locales/
├── en/            # 英文（上游原始文本）
├── zh-CN/         # 简体中文翻译（52 个 .toml 文件）
├── en.toml
└── zh-CN.toml
```

每个 crate 对应一个 `<crate_name>.zh-CN.toml` 文件，通过 `t!` 宏在运行时按 key 查找译文。

### 参与翻译

1. 在 `crates/zed_i18n/locales/zh-CN/` 下找到对应模块的 `.toml` 文件
2. 补全或修正翻译 key
3. 确保与 `en/` 目录下的 key 保持一一对应
4. 提交 PR 到本仓库

---

### 安装

macOS、Linux 和 Windows 可直接[下载 Zed](https://zed.dev/download)，或通过包管理器安装（[macOS](https://zed.dev/docs/installation#macos)/[Linux](https://zed.dev/docs/linux#installing-via-a-package-manager)/[Windows](https://zed.dev/docs/windows#package-managers)）。

> 本汉化版需自行从源码构建，构建方式见下方开发指南。

其他平台暂不可用：

- Web（[跟踪讨论](https://github.com/zed-industries/zed/discussions/26195)）

### 开发 Zed

- [macOS 构建指南](./docs/src/development/macos.md)
- [Linux 构建指南](./docs/src/development/linux.md)
- [Windows 构建指南](./docs/src/development/windows.md)

### 贡献

参见 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解贡献方式。

汉化相关贡献请直接提交 PR 至本仓库（[MindFlowLab/zed-ZH_CN](https://github.com/MindFlowLab/zed-ZH_CN)）。

### 许可证

Zed 源代码主要采用 GPL-3.0-or-later 许可，标注部分采用 Apache-2.0。

第三方依赖的许可信息必须正确提供以通过 CI。

我们使用 [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) 自动合规开源许可。如果 CI 失败，检查以下情况：

- 你创建的 crate 报 `no license specified`？在 Cargo.toml 的 `[package]` 下添加 `publish = false`。
- 依赖报 `failed to satisfy license requirements`？确认该项目的许可证及合规方式，然后将 SPDX 标识符添加到 `script/licenses/zed-licenses.toml` 的 `accepted` 数组。
- `cargo-about` 找不到依赖的许可证？在 `script/licenses/zed-licenses.toml` 末尾添加 clarification 字段，参见 [cargo-about 文档](https://embarkstudios.github.io/cargo-about/cli/generate/config.html#crate-configuration)。

## 赞助

Zed 由 **Zed Industries, Inc.** 开发。

如需经济支持该项目，可通过 GitHub Sponsors 赞助。
赞助直接进入 Zed Industries，作为公司通用收入，不附带任何特权。

