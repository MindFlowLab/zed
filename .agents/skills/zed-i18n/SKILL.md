---
name: zed-i18n
description: >-
  Use when localizing Zed UI text to Simplified Chinese, adding or editing i18n
  keys, working with the zed_i18n crate or its locale TOML files, filling missing
  translation keys, creating a locale file for a new crate, syncing the 汉化 with
  upstream changes, or anything involving the t! macro, the ui_language setting,
  or en/zh-CN locale parity. Trigger whenever the user mentions 汉化, 翻译, 国际化,
  i18n, locale, 缺失翻译 key, 译名, or wants to translate a Zed interface string —
  even if they do not name the zed_i18n crate explicitly.
---

# Zed 汉化（i18n）工作流

本仓库是 Zed 的简体中文汉化版。所有界面文本通过 `zed_i18n` crate 国际化，
默认语言 zh-CN，缺失 key 自动回退英文。本技能描述如何**正确、一致地**汉化界面文本，
以及如何处理汉化维护中的常见任务。

## 核心心智模型

- **一个 crate 统筹**：`crates/zed_i18n` 基于 `rust-i18n`，在编译期把 `locales/` 下所有
  TOML 内嵌进二进制。运行时用 `t!` 宏查询。
- **回退链**：`zh-CN -> en -> key 本身`。某个 key 没有中文翻译时显示英文，英文也没有时
  显示 key 字符串本身。**这正是发现漏译的机制**——界面上出现 `xxx.yyy.zzz` 样的裸 key
  就说明缺翻译。
- **en 是源头**：英文 locale 是回退基准，必须始终完整。汉化是“在 en 之上补 zh-CN”，
  永远不要只改 zh-CN 而不补 en。
- **默认中文**：`zed_i18n::init()` 在 `crates/zed/src/main.rs` 启动早期把 locale 设为
  zh-CN；用户可用 `ui_language` 设置切回英文。

## key 命名与 locale 文件结构

### key 规范

`<crate>.<模块>.<条目>`，例如 `go_to_line.current_line`、`app_menus.file.save`。
key 路径完全由 TOML 内部的表/键决定，**文件名只起组织作用**，不参与 key 拼接。

### 文件布局

```
crates/zed_i18n/locales/
├── en.toml                 # 根 en 文件（app_menus 等全局 key）
├── zh-CN.toml              # 根 zh-CN 文件，与 en.toml 一一对应
├── en/
│   ├── go_to_line.en.toml  # 每个 crate 一个分片文件
│   ├── search.en.toml
│   └── ...
└── zh-CN/
    ├── go_to_line.zh-CN.toml
    ├── search.zh-CN.toml
    └── ...
```

- **后缀命名机制**：rust-i18n 按文件主干最后一段判定 locale——`foo.en.toml` 并入 en，
  `foo.zh-CN.toml` 并入 zh-CN。新增分片文件务必遵守 `<crate>.<locale>.toml` 命名。
- **en/ 与 zh-CN/ 必须一一对应**：每个 `en/<crate>.en.toml` 都要有同名 `zh-CN/<crate>.zh-CN.toml`。
- 每个文件以 `_version = 1` 开头。

### 文件格式示例

`crates/zed_i18n/locales/en/go_to_line.en.toml`：
```toml
_version = 1

[go_to_line]
current_line = "Current Line: %{line} of %{total} (column %{column})"
go_to_line = "Go to line %{line}"
go_to_line_column_tooltip = "Go to Line/Column"
```

`crates/zed_i18n/locales/zh-CN/go_to_line.zh-CN.toml`：
```toml
_version = 1

[go_to_line]
current_line = "当前行：第 %{line} 行，共 %{total} 行（第 %{column} 列）"
go_to_line = "跳转到第 %{line} 行"
go_to_line_column_tooltip = "跳转到行/列"
```

要点：
- 占位符用 `%{name}` 格式，en 与 zh-CN 的占位符名必须一致。
- 中文译文应符合中文排版习惯（全角标点、量词），不是逐词硬译。
- **`_title` 后缀避坑**：当键名与子表同名会冲突（如键 `app_menus.file` 与表
  `[app_menus.file]`），给键加 `_title` 后缀（`app_menus.file_title`）。

## 核心流程：汉化一条界面文本

1. **定位硬编码字符串**。在源码里找到要汉化的英文字面量（按钮 label、tooltip、
   aria-label、提示文本等）。

2. **定 key**。按 `<crate>.<模块>.<条目>` 取名，语义化、小写下划线。优先复用同 crate
   已有的表（如 `[go_to_line]`）。

3. **同时写入 en 与 zh-CN 两个文件**。在 `en/<crate>.en.toml` 补英文原文（若该字符串
   原本就是英文硬编码，en 里也要有对应 key 作为回退源），在 `zh-CN/<crate>.zh-CN.toml`
   写中文译文。占位符两边一致。

4. **替换源码中的字面量为 `t!`**。文件顶部加 `use zed_i18n::t;`，然后：

   ```rust
   // 简单查询（返回 Cow<'static, str>，零分配）
   let tooltip = t!("go_to_line.go_to_line_column_tooltip");

   // 变量插值（返回 String；模板里的 %{line} 等被替换）
   let text = t!("go_to_line.current_line", line = line, total = last_line + 1, column = column);
   ```

5. **处理类型转换**。`t!` 简单形式返回 `Cow<str>`，插值形式返回 `String`。GPUI 组件多接受
   `Into<SharedString>`，`Cow`/`String` 通常可直接传；若编译器报类型不符，按需 `.into()`
   或 `.to_string()`。

6. **验证**（见下文“验证”一节）。

### 真实范例（crates/go_to_line/src/go_to_line.rs）

```rust
use zed_i18n::t;

// 插值查询，format! 表达式可作为参数值
t!(
    "go_to_line.go_to_line_relative",
    line = target_line,
    offset = format!("{offset:+}")
)
.into()
```

## 常见任务 playbook

### A. 汉化一个新组件 / crate

1. 确认该 crate 是否已有 locale 文件（`ls crates/zed_i18n/locales/en/ | grep <crate>`）。
2. 没有则**成对新建** `en/<crate>.en.toml` 与 `zh-CN/<crate>.zh-CN.toml`，均以 `_version = 1`
   开头，顶层表名用 crate 名。
3. 在该 crate 的 `Cargo.toml` 加 `zed_i18n.workspace = true`（若尚未依赖）。
4. 逐条把硬编码字符串替换为 `t!`，同步补两个 locale 文件。
5. 一个组件一个提交，提交信息 `feat(i18n): 汉化 <组件名>`。

### B. 补全缺失的翻译 key

界面上出现裸 key（如 `agent_ui.thread.xxx`）即漏译。

1. 用 key 的第一段定位 crate 与文件：`agent_ui.*` → `crates/zed_i18n/locales/{en,zh-CN}/agent_ui.*.toml`。
2. 在 zh-CN 文件补中文（en 文件若也缺，一并补英文原文）。
3. 提交信息 `fix(i18n): 补全 <crate> 缺失的翻译 key`。

### C. 上游同步后补译新字符串

合并 `upstream/main` 后，上游新增的界面字符串是硬编码英文，需要汉化。

1. 合并并解决冲突（汉化文件可能与上游改动重叠，注意保留 `t!` 调用与 locale key）。
2. `cargo check --workspace` 确认编译通过。
3. 运行应用，排查裸 key 与新增英文文本，按“核心流程”补译。
4. 重点检查上游改动过的、已汉化的文件（`t!` 调用可能被上游覆盖回硬编码）。

### D. 统一译名

同一术语在不同地方译法不一（如 Thread 译“线程”还是“对话”）时统一。

1. `grep -rn "<旧译>" crates/zed_i18n/locales/zh-CN/` 找全所有出处。
2. 统一改为目标译名，注意上下文语义。
3. 提交信息 `fix(i18n): 统一 <术语> 译名为 <译名>`。

## 验证

1. **编译**：`cargo check -p <crate>`（或 `--workspace`）。`t!` 的 key 是运行期查询，
   编译期不校验 key 存在性，所以编译过≠翻译全。
2. **查缺译**：构建运行后，界面上任何形如 `a.b.c` 的裸文本都是缺失 key。
3. **切换语言核对**：在 settings.json 设 `"ui_language": "English"` 与 `"Chinese"`
   （`UiLanguage` 枚举，默认 Chinese），确认两种语言下文本都正确、占位符都填上。
4. **i18n 单测注意**：locale 是进程级全局状态，涉及 `set_locale` 的测试必须串行
   （`zed_i18n` 内部用 `LOCALE_LOCK` 互斥）。新写这类测试时照此办理，否则会因并行竞态
   非确定性失败。

## 提交规范

沿用仓库现有约定（中文提交信息，类型前缀）：

- `feat(i18n): 汉化 <范围>` —— 新汉化一批文本 / 新组件
- `fix(i18n): 补全 <crate> 缺失的翻译 key` —— 补漏译
- `fix(i18n): 补建 <crate> locale 文件` —— 新建缺失的 locale 文件
- `fix(i18n): 统一 <术语> 译名为 <译名>` —— 术语一致化
- `perf(i18n): ...` —— i18n 相关性能优化

一次提交聚焦一个组件或一类改动，不要把无关文件的汉化混进同一 commit。

## 易错点清单

- **只改 zh-CN 不补 en**：en 是回退源，必须同步。否则切到英文或 zh-CN 缺 key 时回退落空。
- **en/zh-CN 占位符不一致**：`%{name}` 名字两边必须相同，否则插值漏填。
- **文件名后缀写错**：`foo.zh-CN.toml` 写成 `foo.zh_CN.toml` 或 `foo.toml` 会导致文件不被
  并入对应 locale，翻译静默失效。
- **键名与子表撞名**：用 `_title` 后缀规避（见上）。
- **忘记 `use zed_i18n::t;`** 或 crate 没加 `zed_i18n` 依赖。
- **类型不符**：`Cow`/`String` 传给 UI 组件时按需 `.into()`。
- **locale 测试并行**：切换 locale 的测试要串行加锁。
- **不要运行 `cargo xtask workflows`**：本 fork 的工作流是自定义的，该命令会用上游模板覆盖
  （此条虽非 i18n，但属本 fork 通用禁忌，合并上游后尤需注意）。
