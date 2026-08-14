# Notra Analyse 功能对齐 AnalysePlugin 技术白皮书

**文档版本：** v2.1

**更新日期：** 2026-08-15

**文档状态：** 实施中，Windows Oracle Golden 移至 Release 验证

**目标工程：** Notra

**兼容对象：** Notepad++ AnalysePlugin 1.14-pre1 / SVN r54
**开发原则：** Feature Parity First、独立实现、Active Document First

---

# 1. 执行摘要

本项目要在 Notra 中独立实现一个完整的 Analyse 子系统。

它不是现有 Find 的“多关键字模式”，也不是简单增加一个 Pattern 列表。开发验收标准是：

> AnalysePlugin 的用户可见核心功能、工作流和 Profile 能力在 Notra 中均有可用实现；搜索、结果、样式和交互具有稳定、可解释的语义。

“功能对齐”不要求：

- 复制 Win32 或 Scintilla UI；
- 复用 AnalysePlugin 的 C++ 结构；
- 像素级模仿旧插件；
- 在 Notra 中引入 AnalysePlugin 的 GPL 源码；
- 继承参考实现的内部缺陷和脆弱数据结构。
- 所有 Regex 边缘语义、错误文本、零长度推进和 Unicode Case 行为逐字节一致。

固定 Windows Binary Oracle 和 Golden Corpus 用于后续 Release 兼容性测试、差异发现和风险分级，不再作为 Phase 2 或 Phase 3 的开发阻塞条件。Release 前必须修复会导致核心功能不可用的差异；不影响功能完成度的行为差异可以记录后发布。

Notra 保持自己的 Tauri、Rust、TypeScript、Monaco 和 MIT 架构。

第一版兼容范围只面向当前活动文档。Open Documents、Workspace、实时日志流、统计、Timeline、AI Pattern 等能力不进入 Compatibility 1.0。

---

# 2. 已核实的代码基线

## 2.1 Notra

| 项目 | 当前基线 |
|---|---|
| Git Commit | `82bf1d7c599897b7c0604c4e17d5e6c3337715ae` |
| 版本 | `0.1.8` |
| 分支 | `main` |
| 远程状态 | `HEAD == origin/main == upstream/main` |
| Commit 日期 | 2026-08-03 |
| Rust Edition | 2024 |
| 桌面框架 | Tauri 2 |
| 编辑器 | Monaco Editor 0.53.0 |
| 前端 | TypeScript 5.9、Vite 7 |
| 发布平台 | Windows x64、macOS arm64、Linux x64 |
| License | MIT |

本次更新前已执行 `git fetch --all --prune`，本地、origin 和 upstream 的 `main` 完全一致。

Notra 当前关键事实：

- `crates/notra-app/frontend/src/main.ts` 为 9,073 行、约 352 KB。
- `crates/notra-app/frontend/src/styles.css` 为 5,438 行、约 105 KB。
- `notra-core` 当前只有 `document`、`fs`、`search` 三个核心模块。
- 当前 Find 支持 Literal、Extended、Regex、Case、Whole Word。
- 当前 Extended 只转换 `\n`、`\r`、`\t`、`\0`、`\\`。
- 当前 Regex 使用 Rust `regex` 1.12.4。
- 当前 Workspace Search 在 Rust 后台执行，Current/Open Documents Search 主要在前端和 Monaco 内完成。
- 当前 Monaco 已分别维护 Search、Active Search、Bookmark Decoration Collection。
- 当前活动持久化实现是 SQLite `notra.db` 中的单条 JSON Session Snapshot。
- `crates/notra-app/src/session.rs` 没有被 `main.rs` 加载，是旧实现，不属于当前运行路径。
- 编辑保护阈值和默认目录搜索上限都是 20 MB。
- `memmap2` 已声明依赖，但当前搜索路径没有实际使用 mmap。

## 2.2 AnalysePlugin

| 项目 | 当前基线 |
|---|---|
| 来源 | [SourceForge SVN HEAD](https://sourceforge.net/p/analyseplugin/code/HEAD/tree/) |
| Revision | `r54` |
| Commit 日期 | 2023-03-03 |
| Commit | `remove double instance to header file to vcproj` |
| 源码版本 | `1.14-pre1` |
| 同步的 Notepad++ 基线 | 8.4.8 |
| 本地只读参考目录 | `/Users/ron/Downloads/github/analyseplugin-code-r54-trunk` |
| License | GPLv3 or later |

本地源码是 SourceForge Snapshot，不包含 `.svn` 元数据，不能直接 `svn update`。

## 2.3 参考证据优先级

AnalysePlugin 的源码、手册和 XSD 彼此存在差异。功能范围判断按以下顺序：

```text
r54 运行时代码
    > r54 manual.txt
    > AnalyseDoc.xsd
```

固定版本的 AnalysePlugin 二进制 + Notepad++ 实际输出是 Release 兼容性测试依据。当它与源码推导结果冲突时记录到 Golden Case，并按“是否影响功能可用性”分级处理。

## 2.4 关键代码证据

| 结论 | 代码证据 |
|---|---|
| Notra Search Mode、Extended、Regex、LineIndex | `crates/notra-core/src/search.rs` |
| 20 MB 编辑保护 | `crates/notra-core/src/document.rs` |
| Workspace Search 与 Tauri Commands | `crates/notra-app/src/app.rs` |
| SQLite Session Snapshot | `crates/notra-app/src/session_store.rs` |
| Monaco、Command、Bookmark、Find State | `crates/notra-app/frontend/src/main.ts` |
| 三平台发布 | `.github/workflows/release.yml` |
| r54 Pattern 字段与默认值 | `tclPattern.h`、`tclPattern.cpp` |
| r54 搜索 Flag、Extended、跨行 Match | `AnalysePlugin.cpp` |
| r54 Dirty Pattern 行为 | `tclResultList.cpp` |
| r54 源行聚合 | `tclFindResultDoc.cpp` |
| r54 Style、Hide、Result、Bookmark | `tclFindResultDlg.cpp` |
| r54 XML Runtime Parser/Writer | `FindConfigDoc.cpp` |
| r54 XSD 差异 | `AnalyseDoc.xsd` |
| r54 用户工作流和过期的 119 说明 | `manual.txt` |
| r54 247 Style 变更 | `changes.txt` |

---

# 3. v1 白皮书的关键事实校正

## 3.1 Pattern Style 上限是 247，不是 119

`manual.txt` 仍写着 119，但 r54 运行时代码已经变更：

```cpp
#define MY_STYLE_MASK 0xff
#define MY_STYLE_COUNT (MY_STYLE_MASK-8)
```

因此实际可分配的自定义 Pattern Style 数量是：

```text
255 - 8 = 247
```

r54 的 `changes.txt` 也明确记录：

```text
Increase the number of supported styles from ~130 to 247
```

Compatibility Mode 的正确语义是：

- 前 247 个 Pattern 可以获得独立样式；
- 第 248 个及之后的 Pattern 仍可参与搜索和结果合并；
- 超出上限的 Pattern 使用 Default Style。

## 3.2 Extended Search 还包含 `\uHHHH` 解析分支

r54 实际搜索路径 `AnalysePlugin::convertExtendedToString()` 包含以下解析分支：

```text
\r
\n
\0
\t
\\
\bBBBBBBBB
\oOOO
\dDDD
\xHH
\uHHHH
```

v1 漏掉了 `\uHHHH` 分支。

`tclPattern` 内还有另一份旧转换函数，它没有 `\u`，但真正执行匹配的是 `AnalysePlugin` 中的版本。因此仍需用固定二进制确认 `\u`、代理项和非法输入的最终行为。

## 3.3 参考插件没有给源编辑器应用 Pattern 颜色

r54 的结果样式只绘制在 Result Scintilla 中。源编辑器侧的 Pattern Style 代码处于停用或注释状态。

参考插件在源文档中实际使用的是 Bookmark Marker。

因此 Compatibility 1.0：

- 必须在源编辑器标记所有结果行；
- 不要求把 Pattern 前景色、背景色、粗体等样式绘制到源文本；
- 源文档彩色 Decoration 属于 Enhanced Mode，不进入兼容验收。

## 3.4 Regex 行为依赖宿主 Notepad++/Scintilla

r54 通过 Scintilla 消息搜索：

```text
SCI_SETSEARCHFLAGS
SCI_SEARCHINTARGET
```

并包含 `BoostRegexSearch.h` 相关 Flag。AnalysePlugin DLL 使用宿主 Notepad++ 的 Scintilla，因此 Regex 语义不只由插件源码决定，还会受到宿主 Notepad++ 版本影响。

所以只下载 AnalysePlugin 源码仍不足以冻结 Regex Oracle。必须再固定：

- AnalysePlugin r54 对应 DLL；
- Notepad++ 版本与架构；
- Windows 测试环境；
- 实际 Golden 输出。

## 3.5 `replaceText` 不是 r54 用户可见功能

r54 `tclPattern` 内部存在 `replaceText` 和 `doReplace` 字段，但当前 UI、XML 和执行路径没有形成完整可用的 Pattern Replace 功能。

Compatibility 1.0 不实现这两个字段。

## 3.6 XML 未知属性不要求 Round-trip 保留

r54 Runtime Parser 会忽略未知属性，但 Writer 不会保存它们。

Notra 必须宽松读取未知属性，不应因未知属性拒绝 Profile；第一版不需要增加扩展属性保存层。

## 3.7 Result 行号开关不需要重新匹配

参考插件因为行号直接嵌入 Result 文本，会触发一次重新搜索。Notra 可以复用已有 PatternResult，只重建 Result Text Model。

用户可见结果一致即可，不复制参考实现的额外计算。

---

# 4. 产品范围

## 4.1 Compatibility 1.0 必须完成

### Pattern

- 任意数量 Pattern；
- Add、Update、Delete、Clear；
- Up、Down、排序、Apply Order；
- Double Click 切换 Do Search；
- Group 启用、禁用和选择；
- Add Selection as Pattern；
- Pattern Hits 和错误状态。

### Search

- Normal；
- Escaped；
- Regex；
- Regex Multiline；
- Match Case；
- Whole Word；
- 跨行 Match；
- 零长度 Match 安全处理；
- 单 Pattern 失败不终止其他 Pattern。

### Visualization

- Selection on Text；
- Selection on Line；
- Hide；
- Foreground；
- Background；
- Bold；
- Italic；
- Underline；
- 后置 Pattern 覆盖前置 Pattern；
- 前 247 个 Pattern 独立样式限制。

### Result

- 源文件行顺序；
- 同一源行只显示一次；
- Matching Patterns；
- 源行号显示；
- Word Wrap；
- Result Font；
- Double Click 跳转；
- Result Find；
- Plain Copy；
- RTF Copy；
- Save Result as Text；
- Bind Result to File；
- Save Result as RTF。

### Profile

- AnalysePlugin XML Import；
- AnalysePlugin XML Export；
- Replace Load；
- Append；
- Prepend；
- Save；
- Save with Hits；
- Recently Used Config Files；
- 拖放 XML Profile 到 Pattern Panel 后加载。

### Behaviour

- Analyse Bookmark；
- Auto Update；
- Cancel；
- 双向 Synchronize Scrolling；
- Enter Action：Just Search、Update Line、Add Line；
- 状态和布局持久化。

## 4.2 Compatibility 1.0 明确不做

- Open Documents Analyse；
- Workspace Analyse；
- Directory Analyse；
- Pattern Replace；
- 源编辑器 Pattern 颜色；
- 结构化日志解析；
- Timeline；
- 统计图；
- AI Pattern；
- 实时 Follow Tail；
- 多个独立 Analyse Result 实例。

这些能力只能在 Compatibility 1.0 通过后，以 Enhanced Mode 增量实现。

---

# 5. 许可证边界

Notra 是 MIT，AnalysePlugin r54 是 GPLv3 or later。

工程规则：

- AnalysePlugin 源码只作为只读行为参考；
- 不复制 GPL 文件到 Notra；
- 不逐行翻译 C++ 到 Rust/TypeScript；
- 不复制 GPL XSD；
- Profile Parser/Writer 根据公开文件格式独立实现；
- Golden Fixture 只保存输入和行为输出；
- 新图标、新 UI 和新资源独立设计；
- Regex 第三方组件必须单独核对许可证和再分发条件。

本地 AnalysePlugin 参考目录必须始终位于 Notra 仓库之外。

---

# 6. Notra 当前能力与缺口

| 能力 | Notra 当前状态 | Analyse 处理 |
|---|---|---|
| Monaco 主编辑器 | 已有 | 复用 |
| 多文档 Model | 已有 | Compatibility 只绑定活动文档 |
| Literal Search | 已有 | 复用语义，不直接复用现有状态机 |
| Extended Search | 只支持 5 类转义 | 新建 Analyse 兼容 Translator |
| Regex | 已有 `fancy-regex` Functional Backend | 通过独立 Matcher Router 使用 |
| Whole Word | 已有 | Golden 校准边界语义 |
| Search Decorations | 已有 | Analyse 使用独立 Collection |
| User Bookmark | 已有 | Analyse Bookmark 必须隔离 |
| Command Registry | 已有 | 注册 `analyse.*` Commands |
| Keybinding | 已有 | 纳入现有配置系统 |
| SQLite Session | 已有 | 只保存轻量 Analyse 设置 |
| Result Editor | 已新增第二个只读 Monaco | 保持独立 Model、Mapping 和 Decoration |
| XML Profile | 没有 | Rust 独立实现 |
| 富文本输出 | 没有 | Rust RTF/HTML Serializer + Clipboard 能力协商 |
| Cancellation | Workspace Search 仅用 Request ID 防陈旧 UI | Analyse 增加后端取消 |
| 大文件编辑 | 超过 20 MB 只读 | 增加独立 Analyse 路径 |
| mmap | 仅有依赖 | 性能阶段按 Benchmark 启用 |

Analyse 不能继续直接写入 9,000 行的 `main.ts`。

本项目只抽取 Analyse 自身模块，不在同一阶段重构整个 Notra 前端。

---

# 7. 目标架构

```text
Main Monaco ───────────────┐
                           │ document snapshot / revision
Pattern Panel ─────────────┤
                           ▼
                 AnalyseController
                           │
                           │ Tauri invoke / event
                           ▼
                 AnalyseTaskRegistry
                           │
                           ▼
                    AnalyseEngine
                 ┌─────────┼─────────┐
                 │         │         │
             Literal   Extended   Regex Compat
                 │         │         │
                 └─────────┴─────────┘
                           │
                    PatternResult[]
                           │
                     Merge By Line
                           │
                     Style Resolver
                           │
          ┌────────────────┴────────────────┐
          ▼                                 ▼
  Result Monaco                     Analyse Bookmarks
```

职责边界：

```text
notra-core
    Matching、LineIndex、Merge、Style、XML、RTF

notra-app
    Tauri Commands、任务取消、文件 IO、Rich Clipboard、Settings

frontend
    Pattern 编辑、状态协调、Monaco Model、导航、滚动同步、菜单
```

核心规则：

> 匹配、合并和最终样式只在 Rust 计算一次。TypeScript 不实现第二套 Style Resolver。

---

# 8. 推荐模块边界

## 8.1 Rust Core

```text
crates/notra-core/src/analyse/
    mod.rs
    model.rs
    matcher.rs
    engine.rs
    extended.rs
    profile.rs
    rtf.rs
```

第一版先保持文件数量小。只有当 `engine.rs` 明显过大时，再拆分 `merge.rs` 和 `style.rs`。

如果 Regex 必须通过 C++ FFI：

```text
crates/notra-regex-compat/
    Cargo.toml
    build.rs
    src/lib.rs
    cpp/bridge.cpp
    cpp/bridge.hpp
```

Bridge 只暴露稳定 C ABI：

```text
compile
find_all
error_message
destroy
```

不得向 Rust 暴露 C++ 对象布局。

## 8.2 Tauri App

```text
crates/notra-app/src/
    analyse_commands.rs
    analyse_tasks.rs
    analyse_clipboard.rs
```

## 8.3 Frontend

```text
crates/notra-app/frontend/src/analyse/
    types.ts
    store.ts
    controller.ts
    patternPanel.ts
    resultPane.ts
    decorations.ts

crates/notra-app/frontend/src/analyse.css
```

`main.ts` 只负责初始化、生命周期接线和现有 Command Registry 注册。

---

# 9. Pattern 领域模型

```rust
pub type PatternId = u64;

pub struct AnalysePattern {
    pub id: PatternId,
    pub order_num: String,
    pub enabled: bool,
    pub search_text: String,
    pub search_type: AnalyseSearchType,
    pub match_case: bool,
    pub whole_word: bool,
    pub selection: AnalyseSelection,
    pub hide: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub comment: String,
    pub group: String,
}

pub enum AnalyseSearchType {
    Normal,
    Escaped,
    Regex,
    RegexMultiline,
}

pub enum AnalyseSelection {
    Text,
    Line,
}
```

r54 默认值：

```text
enabled       true
searchType    normal
matchCase     false
wholeWord     false
selection     line
hide          false
bold          false
italic        false
underline     false
foreground    black
background    white
order/comment/group empty
```

Notra 使用稳定整数 ID，不复制参考实现以 `double` 作为 Pattern ID 的内部结构。

## 9.1 Revision 分类

### Search Revision

以下字段变化必须重新匹配该 Pattern：

```text
enabled
search_text
search_type
match_case
whole_word
```

### Presentation Revision

以下变化只需要重新 Merge/Style/Render：

```text
Pattern list order
selection
hide
bold
italic
underline
foreground
background
```

### Metadata Revision

以下字段本身不需要重新匹配或重新着色：

```text
order_num
comment
group
```

执行 Apply Order 后，真正的 Pattern list order 改变，才触发 Presentation Revision。

---

# 10. Matcher 兼容层

```rust
pub trait AnalyseMatcherBackend {
    fn compile(
        &self,
        pattern: &AnalysePattern,
    ) -> Result<Box<dyn CompiledAnalyseMatcher>, AnalysePatternError>;
}

pub trait CompiledAnalyseMatcher: Send + Sync {
    fn find_all(
        &self,
        text: &str,
        line_index: &LineIndex,
        cancel: &CancellationToken,
    ) -> Result<Vec<RawMatch>, AnalysePatternError>;
}
```

路由：

```text
Normal          -> Literal Matcher
Escaped         -> Analyse Extended Translator -> Literal Matcher
Regex           -> Regex Functional Backend
Regex Multiline -> Regex Functional Backend + dot matches newline
```

现有 `SearchMatcher` 可以共享低层辅助函数，但 Analyse 不共享 Find UI State、Search Report 或生命周期。

---

# 11. Extended Search 兼容规则

必须独立实现：

```rust
pub fn translate_analyse_extended(input: &str) -> Result<String, ExtendedError>;
```

| 输入 | 语义 |
|---|---|
| `\r` | CR |
| `\n` | LF |
| `\0` | NUL |
| `\t` | TAB |
| `\\` | Backslash |
| `\b01000001` | 8 位二进制值 |
| `\o101` | 3 位八进制值 |
| `\d065` | 3 位十进制值 |
| `\x41` | 2 位十六进制值 |
| `\u0041` | 4 位十六进制 Unicode code unit |

非法或长度不足的数值转义按普通文本回退，不能静默丢字符。

必须覆盖：

- 尾部单个反斜杠；
- 未知转义；
- 数字不足；
- 超出进制范围；
- 大小写十六进制；
- `\0`；
- `\u` 代理项；
- CRLF 跨行；
- 连续混合转义。

现有 Notra `translate_extended()` 行为不能被修改，以免回归 Find。

---

# 12. Regex 功能策略

Rust `regex` 不支持 Look-around 和 Backreference，不能独立覆盖 AnalysePlugin 的 Regex 功能。

例如：

```regex
foo(?=bar)
(foo).*\1
(?<=ERROR )wlan0
```

功能后端要求：

- Windows、macOS、Linux 使用同一语义后端；
- Regex 和 Regex Multiline 仅在 Dot/Newline Flag 上产生预期差异；
- 支持 Look-around、Backreference、POSIX Class 和 `\R`；
- 明确定义 Zero-length Match 的推进规则；
- 返回清晰的 Pattern 级错误；
- 支持取消；
- 编译结果可缓存。

开发实现采用 `fancy-regex` 0.14.x 作为跨平台功能后端：

- Look-around、Backreference、POSIX Class 由 `fancy-regex` 提供；
- `\R` 由 Analyse Adapter 转换为通用 Unicode 换行集合；
- Regex 默认启用行锚点语义，Regex Multiline 额外启用 Dot Matches Newline；
- Fancy Pattern 使用 Backtrack Limit 避免灾难性回溯无限占用；
- Matcher Trait 保留，后续发现阻断性兼容差异时可以替换后端而不影响 Engine/UI。

Windows Golden 在 Release 阶段对上述实现做差异测试，但不要求二进制行为 100% 一致。后端版本、许可证和功能差异必须记录到 `REFERENCE.md`。

---

# 13. Match、LineIndex 与 Unicode 坐标

```rust
pub struct RawMatch {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub line_spans: Vec<LineSpan>,
}

pub struct LineSpan {
    pub line: usize,
    pub start_byte_in_line: usize,
    pub end_byte_in_line: usize,
}
```

跨行 Match 必须把覆盖的每一行加入 Result。

每次 Analyse Run 只建立一次 LineIndex，所有 Pattern 共享。

内部统一使用 UTF-8 Byte Offset；发送给 Monaco 时显式转换为 UTF-16 Column：

```rust
pub struct MonacoRangeDto {
    pub start_line: u32,
    pub start_column_utf16: u32,
    pub end_line: u32,
    pub end_column_utf16: u32,
}
```

不能把以下坐标混为一谈：

```text
UTF-8 byte offset
Unicode scalar index
UTF-16 code unit index
```

强制测试：中文、繁体、日文、韩文、Emoji、组合字符、CRLF、CR、LF、Mixed EOL。

---

# 14. Result 聚合模型

```rust
pub struct PatternResult {
    pub pattern_id: PatternId,
    pub matches: Vec<RawMatch>,
    pub error: Option<AnalysePatternError>,
    pub search_revision: u64,
}

pub struct LinePatternMatch {
    pub pattern_id: PatternId,
    pub spans: Vec<LineSpan>,
}

pub struct MergedLineResult {
    pub source_line: usize,
    pub text: String,
    pub matches: Vec<LinePatternMatch>,
    pub styled_segments: Vec<StyledSegment>,
}

pub struct AnalyseResult {
    pub run_id: u64,
    pub document_id: u64,
    pub document_revision: u64,
    pub pattern_revision: u64,
    pub lines: Vec<MergedLineResult>,
    pub total_matches: usize,
    pub pattern_errors: Vec<AnalysePatternError>,
}
```

聚合使用：

```text
PatternResult[]
    -> BTreeMap<source_line, line matches>
    -> MergedLineResult[]
```

强制语义：

- Result 按源行升序；
- 同一源行只出现一次；
- 同一行保留全部 Matching Pattern；
- Matching Pattern 顺序与当前 Pattern List 一致；
- 多行 Match 覆盖的每一行都进入 Result；
- Disabled Pattern 不产生 Result；
- 单 Pattern Regex 错误不清空其他 Pattern 的有效结果。

---

# 15. Style Resolver

最终样式必须在 Rust 数据层计算，不能依赖 Monaco Decoration 的偶然覆盖顺序。

规则：

1. 每行先应用 Default Style。
2. Pattern 按列表从上到下处理。
3. `selection = text` 只作用于 Match Span。
4. `selection = line` 作用于整条源文本，不包括生成的行号前缀。
5. 后处理的 Pattern 覆盖相交区域的已有样式。
6. 结果输出为无重叠 Segment。

```rust
pub struct StyledSegment {
    pub start_byte_in_line: usize,
    pub end_byte_in_line: usize,
    pub pattern_id: Option<PatternId>,
    pub style: EffectiveStyle,
}
```

Hide 规则：

```text
Logical Result Text 永远保留完整文本
View 通过 Style 隐藏
Plain Copy / RTF / Save 仍读取完整文本
```

Compatibility Style Limit：

```text
Pattern index 0...246 -> 独立样式
Pattern index >= 247 -> Default Style
```

Enhanced Mode 后续可以取消该限制，但不能改变 Compatibility Mode。

---

# 16. Result Monaco

Result 使用第二个只读 Monaco，放在主编辑器下方的可调高度 Pane 中。

Pattern Configuration 放入现有右侧工具体系，新增：

```ts
type RightTool = "search" | "outline" | "analyse";
```

Result Monaco 配置：

```ts
monaco.editor.create(container, {
  readOnly: true,
  minimap: { enabled: false },
  lineNumbers: "off",
  wordWrap: settings.resultWordWrap ? "on" : "off",
  largeFileOptimizations: true,
});
```

维护：

```ts
interface ResultLineMapping {
  resultLine: number;
  sourceDocumentId: number;
  sourceLine: number;
  matchingPatternIds: number[];
}
```

源行号必须显示源文件行号，而不是 Result Model 行号。兼容输出使用文本前缀：

```text
  15: ERROR wifi timeout
 104: WARN driver recovery
```

切换 Show Line Numbers 时只重建 Result Model 和 Mapping，不重新匹配。

双击 Result：

```text
resultLine
  -> sourceLine mapping
  -> 激活源文档
  -> revealLineInCenterIfOutsideViewport
  -> setPosition
  -> 根据设置决定焦点归属
```

---

# 17. Source Bookmark

r54 在源编辑器使用 Bookmark Marker，不应用 Pattern 颜色。

Notra 必须维护独立集合：

```ts
userBookmarkDecorations
analyseBookmarkDecorations
```

Analyse Bookmark 生命周期：

- Run 完成后替换为当前 Result 行集合；
- Clear Analyse 时只清除 Analyse Bookmark；
- Pattern Disable/删除后随 Result 更新；
- 文档切换时按 Document ID 恢复；
- 永远不删除用户书签。

参考插件可能共用宿主 Bookmark Marker。Notra 的隔离策略属于安全修正，不改变 Analyse 结果语义。

---

# 18. Pattern Configuration UI

配置区必须包含：

```text
Search Text
Search Type
Case
Whole Word
Do Search

Selection
Hide
FG / BG
Bold / Italic / Underline

Comment
Group
Order
```

Pattern Table 最低列集合：

```text
Enabled | Order | Search Text | Type | Group | Comment | FG | BG | Hits | Status
```

交互规则：

### Add

- 无选中行：追加到末尾；
- 有选中行：插入到选中行之后。

### Update

- Draft 覆盖选中 Pattern；
- 只重算 Search Fingerprint 变化的 Pattern。

### Delete

- 删除 Pattern 和对应 PatternResult；
- 重新 Merge/Style，不重跑无关 Pattern。

### Up / Down / Sort

- 不重新匹配；
- 必须重新计算 Pattern Priority 和 Style；
- Apply Order 只在实际列表顺序改变后生效。

### Double Click Row

- 切换 `enabled`；
- Disable 删除该 Pattern 的有效 Result；
- Enable 只执行该 Pattern。

### Keyboard

- Pattern Table 上下方向键改变选中行时，Draft 同步显示该 Pattern；
- `Esc` 放弃未提交 Draft，并恢复当前选中 Pattern 的字段。

### Enter State Machine

```text
Draft unchanged -> Search

Draft changed -> EnterAction
  - Just Search
  - Update Line
  - Add Line
```

必须写纯状态机测试，不能把规则散落到 DOM Event Handler。

---

# 19. XML Profile 兼容

基本格式：

```xml
<AnalyseDoc
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:noNamespaceSchemaLocation="./AnalyseDoc.xsd">
  <SearchText
    orderNum="10"
    doSearch="true"
    searchType="rgx_multiline"
    matchCase="false"
    wholeWord="false"
    select="line"
    hide="false"
    bold="false"
    italic="false"
    underlined="false"
    color="red"
    bgColor="#FFFFFF"
    comment=""
    group="network">
    ERROR.*timeout
  </SearchText>
</AnalyseDoc>
```

必须支持属性：

```text
hits
orderNum
doSearch
searchType
matchCase
wholeWord
select
hide
bold
italic
underlined
color
bgColor
comment
group
```

Runtime/XSD 差异：

- XSD 没有 `doSearch`，Runtime 会读写；
- XSD 没有 `rgx_multiline`，Runtime 接受；
- Runtime Writer 可以写 `hits`；
- Runtime Reader 不使用 `hits` 作为搜索输入；
- Runtime 保留 SearchText 中有意义的空格和 Tab；
- 未知属性被忽略；
- 缺失属性使用默认 Pattern 值。

禁止严格 XSD Validation 作为 Import Gate。

Writer：

- UTF-8；
- 根节点为 `AnalyseDoc`；
- 保持当前 Pattern 顺序；
- 只写与默认值不同的可选属性；
- 支持预定义颜色和 `#RRGGBB`；
- `rgx_multiline` 使用 Runtime 接受值；
- 可选写入 Hits；
- Notra 导出文件必须能被固定参考插件加载。

Load Mode：

```rust
pub enum ProfileLoadMode {
    Replace,
    Append,
    Prepend,
}
```

Prepend 和 Append 必须保持导入文件内部顺序。

---

# 20. Auto Update、Cancel 与陈旧结果保护

Auto Update 监听当前 Monaco Model：

```text
onDidChangeContent
  -> debounce 300 ms
  -> cancel previous run
  -> run current document revision
```

后端状态：

```rust
pub struct AnalyseRunContext {
    pub run_id: u64,
    pub document_id: u64,
    pub document_revision: u64,
    pub pattern_revision: u64,
    pub cancel: Arc<AtomicBool>,
}
```

Frontend 接受结果前必须校验：

```text
run_id
document_id
document_revision
pattern_revision
```

任一不一致，丢弃返回结果。

取消检查至少发生在：

- Pattern 之间；
- 长 Literal 扫描的分块边界；
- Regex Backend 可中断边界；
- Merge/Style 大结果循环；
- RTF 大结果序列化。

Progress Event：

```ts
interface AnalyseProgress {
  runId: number;
  currentPattern: number;
  totalPatterns: number;
  matches: number;
}
```

---

# 21. Synchronize Scrolling

Source -> Result：

```text
source first visible line
  -> binary search nearest matched source line
  -> result line
  -> result reveal near top
```

Result -> Source：

```text
result first visible line
  -> ResultLineMapping.sourceLine
  -> source reveal near top
```

必须使用双向保护：

```ts
let syncingFromSource = false;
let syncingFromResult = false;
```

文档 ID 不一致时禁止同步。

---

# 22. Result Context 功能

菜单：

```text
Copy
Copy as Rich Text
Select All
────────────────
Find in Analyse Result...
────────────────
Bind Result to File...
Unbind Result File
Save Result As...
Save Result as RTF...
────────────────
Word Wrap
Show Source Line Numbers
────────────────
Matching Patterns >
Analyse Settings...
```

Matching Patterns：

- 使用当前 Result 行的 Pattern ID；
- 按 Pattern List 顺序；
- 最后一项是最终样式优先级最高的 Pattern；
- 菜单显示 Order Number 时与 r54 行为一致。

Result Find 使用独立状态，不复用主编辑器 Find State。它支持 Normal、Escaped、Regex、Case、Whole Word，并可以从当前 Pattern 快速填入查询。Regex/Extended 行为必须复用 Analyse 兼容层，不能改用 JavaScript `RegExp` 形成第三套语义。

---

# 23. Copy、RTF 与文件输出

## 23.1 Plain Text

读取 Result Logical Model，包含视觉上 Hide 的文本。

## 23.2 RTF

Rust Serializer 输入：

```text
Result Text
+ StyledSegment[]
+ Result Font
```

输出必须包含：

```text
font table
color table
foreground
background highlight
bold
italic
underline
Unicode escaping
line ending
```

## 23.3 Cross-platform Rich Clipboard

Notra 发布三个平台，富文本复制按能力协商：

```text
text/rtf + text/html + text/plain
    -> text/html + text/plain
    -> text/plain
```

RTF 和 HTML 必须由同一个 Rust Logical Result 生成，不能让三个平台使用不同的内容语义。Windows WebView 支持 `text/rtf` 时优先同时写入 RTF；macOS/Linux 或其他不接受 RTF 的 WebView 使用 HTML 富文本。

Release 测试必须验证至少一个富文本 MIME 能被目标平台的常见应用粘贴；只有 WebView Clipboard 在目标平台实测无法完成富文本复制时，才增加该平台的 Native Adapter。

## 23.4 Bound Result File

绑定后，每次有效 Result 更新都原子覆盖目标文件。

无效、取消或陈旧 Run 不得覆盖文件。

用户显式 Unbind 后停止写入。

---

# 24. Persistence

Profile 本身继续使用 XML 文件，不把完整 Pattern Profile 复制进数据库。

当前工作 Pattern List 使用 Notra 管理的内部 XML 自动保存，并在下次启动恢复；用户显式加载和保存的 Profile 仍是独立文件。内部工作 Profile 必须原子写入，不能依赖 `lastProfile` 指向的外部文件仍然存在。

SQLite Session 只保存轻量状态：

```text
panelVisible
resultVisible
resultPaneHeight
defaultPattern
useBookmark
autoUpdate
syncScrolling
dblClickJumpsToEditor
enterAction
resultWordWrap
resultLineNumbers
resultFontName
resultFontSize
maxRecentProfiles
recentProfiles
searchHistory
commentHistory
groupHistory
customColors
patternTableColumnWidths
patternTableColumnOrder
lastProfile
boundResultFile
```

第一版在现有 SessionSnapshot 中增加可选 `analyse` 字段，复用当前 SQLite JSON 存储。

不持久化：

- Match；
- Merged Result；
- Styled Segment；
- Cancellation Token；
- Run ID。

如果以后 Analyse 状态需要独立迁移，再升级数据库 Schema；Compatibility 1.0 不提前增加新表。

---

# 25. Command System

扩展现有 `CommandCategory`：

```text
Analyse
```

注册：

```text
analyse.togglePanel
analyse.addSelectionAsPattern
analyse.run
analyse.cancel
analyse.clear
analyse.openProfile
analyse.saveProfile
analyse.options
analyse.focusResult
```

默认快捷键进入现有 Keybinding System，不在 DOM Event 中硬编码。

`Add Selection as Pattern`：

- 获取主 Monaco 当前选择；
- 空选择时命令不可用；
- 使用 Default Pattern 的非文本属性；
- 插入到当前 Pattern 后；
- 不自动扩大到 Workspace。

---

# 26. 大文件策略

必须分开：

```text
Editable Limit
Analyse Limit
```

执行路由：

```text
普通或 Dirty 文档
  -> Monaco text snapshot
  -> analyse_text

大文件、未修改、存在本地路径
  -> analyse_path
  -> Rust 直接读取/映射
  -> Compact Result DTO
```

第一阶段先保证 20 MB 内活动文档正确，不提前引入复杂 mmap 管线。

性能阶段按顺序优化：

1. LineIndex 每个 Run 只建立一次；
2. Pattern Compile Cache；
3. 只重算 Search Fingerprint 改变的 Pattern；
4. Merge/Style 只处理受影响行；
5. 大 Result 分批传输；
6. Benchmark 证明有收益后启用 mmap；
7. Literal 多 Pattern Automaton 最后考虑。

UI 线程不得执行 Regex 扫描、XML 大文件解析或 RTF 大结果序列化。

---

# 27. Golden Compatibility Harness

在实现 UI 前建立：

```text
tests/analyse-compat/
    REFERENCE.md
    profiles/
    fixtures/
    expected/
```

`REFERENCE.md` 必须记录：

```text
AnalysePlugin source revision
AnalysePlugin DLL hash
Notepad++ version
Notepad++ architecture
Windows version
Regex runtime information
Golden 生成步骤
```

Expected 不能只保存截图：

```json
{
  "sourceLines": [12, 38],
  "totalMatches": 3,
  "lines": [
    {
      "line": 12,
      "matchingPatterns": [1, 4],
      "segments": [
        {
          "start": 0,
          "end": 5,
          "pattern": 4,
          "hidden": false,
          "foreground": "#FF0000",
          "background": "#FFFFFF"
        }
      ]
    }
  ]
}
```

Golden Matrix 至少覆盖：

### Search

```text
Normal / Escaped / Regex / Regex Multiline
Case Sensitive / Insensitive
Whole / Partial
Empty Pattern
Invalid Regex
Zero Length
Multiple Match Same Line
Multi-line Match
```

### Extended

```text
\r \n \0 \t \\
\b \o \d \x \u
invalid / truncated / mixed
```

### Regex

```text
anchors
capture
backreference
lookahead
lookbehind
greedy/lazy
POSIX class
\R
dot newline
Unicode
zero length
CRLF boundary
```

### Style

```text
Text / Line
FG / BG
Bold / Italic / Underline
Hide
Overlap
Nested
Pattern 247 / 248
```

### XML

```text
missing defaults
doSearch
rgx_multiline
named color
#RRGGBB
whitespace
unknown attributes
Replace / Append / Prepend
Save with Hits
```

---

# 28. 实施路线

## Phase 0 — Reference Baseline

交付：

- 固定 AnalysePlugin r54 DLL；
- 固定 Notepad++ 版本；
- 记录二进制 Hash；
- 建立 Release Golden Corpus 与采集工具；
- 确认跨平台 Regex 功能后端；
- 更新 `REFERENCE.md`。

完成标准：核心功能范围有源码证据，Release 对照工具准备完成；Windows 实际采集可以延后。

## Phase 1 — Domain、Extended、Profile

交付：

- `notra-core::analyse` 模块；
- Pattern/Profile/Result Model；
- Extended Translator；
- XML Import/Export；
- Search/Presentation Revision；
- Rust Unit/Golden Tests。

完成标准：Profile 可在 Rust 测试中正确 Round-trip，Extended Corpus 全通过。

## Phase 2 — Regex、Merge、Style

交付：

- Regex Functional Backend；
- 单次 LineIndex；
- PatternResult；
- Merge By Line；
- Style Resolver；
- Cancellation；
- Pattern 级错误。

完成标准：

```text
text + profile -> deterministic AnalyseResult
```

核心功能测试 100% 通过；Windows Binary Golden 不阻塞进入 Phase 3。

## Phase 3 — Daily-use UI

交付：

- Analyse 模块化前端；
- Pattern Panel；
- Add/Update/Delete/Move/Sort；
- Result Monaco；
- Double Click Navigation；
- Analyse Bookmark；
- Line Number、Word Wrap、Font；
- Command Registry 集成。

完成标准：不依赖 Debug JSON 即可完成日常 Analyse 工作流。

## Phase 4 — Full Workflow Compatibility

交付：

- Load/Append/Prepend/Save；
- Drag-and-drop Profile Load；
- Recent Profiles；
- Auto Update；
- Progress/Cancel；
- Stale Protection；
- Scroll Sync；
- Result Find；
- Matching Patterns；
- Plain/RTF Copy；
- Text/RTF Save；
- Bound Result File；
- Settings Persistence。

完成标准：AnalysePlugin 用户可见核心工作流完整。

## Phase 5 — Cross-platform / Large-file Hardening

交付：

- 三平台 Regex 一致性；
- 三平台 Rich Clipboard；
- 100 MB / 500 MB / 1 GB Benchmark；
- `analyse_path`；
- 大 Result 分批传输；
- mmap 或其他经 Benchmark 证明有效的优化；
- 回归、性能和发布测试。

完成标准：三平台功能、性能和 Release 兼容性风险均达到发布要求。

---

# 29. 最终发布门槛

## Search

```text
Normal 功能测试                 100%
Escaped 功能测试                100%
Regex 核心功能测试              100%
Regex Multiline 核心功能测试    100%
Windows Golden 差异             已执行、已分级、无功能阻断项
```

## Result

```text
源行聚合可用且顺序稳定
Matching Pattern 顺序稳定
Style Winner 与 Hide 规则稳定
行号显示正确
```

## Profile

```text
AnalysePlugin XML -> Notra
Notra XML -> 固定 AnalysePlugin
Replace / Append / Prepend
Save with Hits
```

## Interaction

```text
Add / Update / Delete / Clear
Up / Down / Sort / Apply Order
Double Click Toggle
Enter Action
Group Actions
Add Selection as Pattern
```

## Behaviour

```text
Bookmark
Auto Update
Cancel
Stale Result Protection
Bidirectional Scroll Sync
```

## Result Workflow

```text
Double Click
Find
Plain Copy
RTF Copy
Text Save
RTF Save
Bound File
Word Wrap
Line Numbers
Font
Matching Patterns
```

## Platform

```text
Windows x64
macOS arm64
Linux x64
```

三平台必须使用同一 Search/Profile/Result 实现。Windows Golden 不要求所有边缘行为完全一致，但发现的差异必须有明确分级、测试或发布说明。

---

# 30. 风险与控制

| 风险 | 等级 | 控制措施 |
|---|---|---|
| Regex 与 Notepad++ 边缘行为不同 | 高 | Release Binary Golden + 差异分级 |
| GPL 代码污染 MIT 工程 | 极高 | 只读外部参考、独立实现、Diff 审核 |
| Extended `\u`/非法回退不一致 | 中 | 单元测试 + Release Golden |
| Pattern 重叠样式不确定 | 高 | Rust Segment Resolver |
| Hide 导致 Copy 丢文本 | 高 | Logical Model 与 View 分离 |
| UTF-8/UTF-16 偏移 | 高 | 显式 Range DTO + Unicode Matrix |
| Auto Update Race | 高 | Run ID + 四重 Revision 校验 + Cancel |
| 三平台 Regex 不一致 | 高 | 单一 Functional Backend |
| 三平台 Rich Clipboard 差异 | 高 | RTF/HTML 能力协商 + 平台测试，失败平台再补 Native Adapter |
| 大 Result 阻塞 UI | 高 | Rust 后台 + 分批 DTO |
| Analyse 删除用户 Bookmark | 中 | 独立 Decoration Collection |
| `main.ts` 继续膨胀 | 高 | Analyse 前端模块边界 |

---

# 31. 正式技术决策

### ADR-001

Analyse 是独立子系统，不扩展现有 Find 状态机。

### ADR-002

Compatibility 1.0 只分析当前活动文档。

### ADR-003

匹配、Merge、Style、XML、RTF 的唯一实现位于 Rust。

### ADR-004

Normal、Escaped、Regex、Regex Multiline 通过统一 Matcher Backend 接口执行。

### ADR-005

Analyse Escaped 使用独立 Translator，并支持 r54 Runtime 的 `\uHHHH`。

### ADR-006

Regex 采用可覆盖 Look-around、Backreference、POSIX Class、`\R` 和 Dot Matches Newline 的跨平台功能后端。Windows Binary Golden 是 Release 差异测试，不是 Backend 选型或 UI 开发的前置门槛。

### ADR-007

PatternResult 与 MergedLineResult 分离，支持单 Pattern 增量重算。

### ADR-008

Search Revision、Presentation Revision、Metadata Revision 分离。

### ADR-009

最终 Style 在 Rust 数据层 Resolve，后置 Pattern 获胜。

### ADR-010

Compatibility Style Limit 使用 r54 实际值 247。

### ADR-011

Result 使用第二个只读 Monaco；源编辑器 Compatibility 标记只使用 Analyse Bookmark。

### ADR-012

Analyse Bookmark 与 User Bookmark 隔离。

### ADR-013

Profile 使用宽松 Runtime-compatible Parser，不使用严格 XSD Gate。

### ADR-014

轻量 Analyse Settings 复用当前 SQLite Session Snapshot；不持久化 Result。

### ADR-015

大文件优化由 Benchmark 驱动。`mmap` 只有在不可变快照或跨平台文件稳定性契约成立后才能进入生产路径；多 Pattern Automaton 最后考虑。

### ADR-016

GPL 参考源码保持在 Notra 仓库之外，不复制、不翻译、不提交。

---

# 32. 当前实施状态

| 模块 | 状态 |
|---|---|
| 两仓库代码基线核实 | 已完成 |
| r54 源码事实校正 | 已完成 |
| Reference DLL / Notepad++ Oracle 固定 | Release 测试准备已完成：r54 x64/x86 DLL 与 Notepad++ 8.4.8 x64 已固定并记录 SHA-256，Manifest 与 Windows 准备脚本已完成；Windows 实际执行延后 |
| Golden Corpus | 已建立目录、Fixture、结构化 Contract 与生成规范；Windows Golden 输出作为 Release 测试证据后补 |
| Analyse Rust Domain | Phase 1 已完成：Pattern、Result、Style DTO 与 Revision 分类 |
| Extended Functional Support | Phase 1 实现与单测已完成；二进制边缘差异移至 Release 测试 |
| Regex Functional Backend | 已完成：`fancy-regex`、Look-around、Backreference、POSIX Class、`\R`、Dot Matches Newline、Zero-length、错误隔离与取消边界 |
| Merge / Style Engine | Phase 2 核心已完成：Mixed-EOL LineIndex、Normal/Escaped/Regex、Compile Cache、按 Search Revision 增量 PatternResult、Merge、Style、247 Limit、取消与 Pattern 错误隔离 |
| Pattern UI | 功能主链已完成：Add/Update/Delete/Clear/Move/Sort、双击切换、全部与 Group 启停、Add Selection、Enter Action、字段编辑、Hits/错误状态 |
| XML Profile | Phase 4 工作流已接入：Load/Replace/Append/Prepend/Save、拖放、Recent 与 Session 持久化；固定插件反向加载待 Release 验证 |
| Result Monaco | Phase 3 主切片已完成：独立只读 Model、样式、源行号、换行和字号 |
| Bookmark / Navigation | Phase 3 主切片已完成：Analyse Bookmark 独立 Collection、Result 双击回源 |
| Auto Update / Scroll Sync | Phase 4 已完成：300 ms Debounce、真实后端 CancellationToken、四重 Stale Gate、双向滚动同步 |
| RTF / Clipboard / File Binding | 功能主链已完成：Plain Copy、Rust RTF/HTML Rich Copy、Text/RTF Save、Bound Result File；三平台实际粘贴留 Release 验证，失败平台再补 Native Adapter |
| Settings / Persistence | 已完成：Auto Update、Line Number、Word Wrap、Font、Scroll Sync、Enter Action、Bound File、Recent Profile 与当前 Working Pattern List 复用 SQLite Session Snapshot |
| Large File Optimization | Phase 5 大文件主链已完成：安全 `analyse_path`、2 GiB 上限、读取前后尺寸校验、2000 行结果分批和 100/500/1024 MiB Benchmark；mmap 因外部截断风险暂保持 Benchmark-only |
| Release Compatibility Test | 准备完成、实际三平台与 Windows Oracle 执行待后续 Release |

## 32.1 2026-08-15 首个实施切片

已落地：

- 新增 `notra-core::analyse`，保持与现有 Find 状态机隔离；
- 新增 Pattern/Profile/Result 领域模型和 Search/Presentation/Metadata Revision 分类；
- 新增独立 Extended Translator，覆盖简单转义、二/八/十/十六进制、`\uHHHH`、非法回退和 UTF-16 代理项；
- 新增 AnalysePlugin Runtime-compatible XML Profile Parser/Writer；
- 新增 Replace、Append、Prepend Load Mode；
- 新增 `tests/analyse-compat` Golden Harness、参考哈希和 Phase 1 Fixture；
- `notra-core` 单元/集成/Doc Tests 全部通过，Clippy `-D warnings` 通过。

移至 Release 阶段的兼容证据：

- 固定 Windows 版本中的 r54 + Notepad++ 8.4.8 实际输出；
- Regex Golden 差异报告；
- Notra 导出 XML 在固定参考插件中的反向加载；
- `\u` 代理项、非法数值转义的二进制 Oracle 校准。

## 32.2 2026-08-15 Phase 2 Core

已落地：

- 新增 `AnalyseMatcherBackend`、`CompiledAnalyseMatcher` 与 `AnalyseMatcherRouter`；
- Regex / Regex Multiline 可通过 Matcher Router 注入或禁用 Backend；Backend 不可用时返回 Pattern 级错误，不影响其他 Pattern；
- 新增每次 Run 只构建一次的 `LineIndex`，覆盖 CRLF、CR、LF 和 Mixed EOL；
- 新增 UTF-8 Byte Offset 到 Monaco UTF-16 Column 的显式转换；
- 新增 64 KiB 分块 Literal 扫描、跨块 Match、全局非重叠语义和取消检查；
- 新增 `AnalyseEngine`、按源行 `BTreeMap` Merge、Matching Pattern 顺序和单 Pattern 错误隔离；
- 新增按 Search Fingerprint 的 Compile Cache，并按当前 Pattern 集合修剪；
- 新增按 Document Revision 和 Pattern Search Revision 复用 `PatternResult` 的增量执行路径；排序、样式和元数据变化不重新扫描；
- 新增确定性 Style Resolver，覆盖 Text、Line、Hide、FG/BG、Bold/Italic/Underline、后置 Pattern 获胜及第 248 个 Pattern Default Style；
- 新增 Phase 2 结构化 Expected Contract；该文件明确标记为 `provisional_not_binary_verified`，不作为二进制兼容证据；
- 本切片完成时 `notra-core` 59 个单元/集成测试通过；Regex Functional Backend 接入后的最新结果见 32.5。

Phase 2 的 Windows Oracle、Extended、Whole Word 和 Unicode Case Golden 已调整为 Release 验证项，不再阻塞 Phase 3。

## 32.3 2026-08-15 Windows Oracle 准备工具

已新增：

- 机器可读 `oracle-manifest.json`，固定官方 URL、Archive SHA-256、DLL/EXE SHA-256 和架构；
- `prepare-oracle.ps1`，在 Windows x64 下载并校验两个官方包、安装插件、拒绝复用陈旧 Runtime；
- 自动生成不含用户名和机器名的 `oracle-environment.json`，记录 Windows 版本、Build、架构和二进制哈希；
- Windows Golden 采集步骤与只允许使用仓库 Fixture 的隐私边界。

当前 macOS 主机没有 PowerShell 或 Windows Runtime，因此已完成 JSON 解析、下载文件哈希和静态检查，但 PowerShell 脚本尚未在 Windows 实际执行。

## 32.4 2026-08-15 Windows 环境状态

当前主机已核实：

- 没有 Wine、CrossOver、Whisky；
- 没有 UTM、Parallels Desktop、VMware Fusion；
- 没有 QEMU、VirtualBox 或对应 CLI；
- Docker CLI 存在但 daemon 未运行，且 macOS Docker 不能提供固定 Windows Desktop Oracle；
- 没有 PowerShell，无法在本机执行 Windows 准备脚本。

这不再构成开发阻塞。获得 Windows 环境后执行以下 Release 验证：

```text
校验 Environment/Artifact Hash
    -> 导入 Golden Expected
    -> 对 Notra Functional Regex Backend 跑同一 Corpus
    -> 生成差异报告并按功能影响分级
    -> 修复功能阻断差异
    -> 记录允许保留的边缘行为差异
```

## 32.5 2026-08-15 功能一致口径与 Regex 解锁

产品目标已明确为“功能保持一致”，不要求所有边缘行为与固定二进制完全一致。已据此解除 Phase 2 的 Windows 前置 Gate：

- 默认接入纯 Rust `fancy-regex` 0.14.0 Functional Backend；
- 支持行锚点、Look-around、Backreference、POSIX Class、`\R`、Regex Multiline 和 Zero-length Match；
- 保留 `AnalyseMatcherBackend` 接口，后续 Release 测试发现阻断性差异时可局部替换；
- Invalid Regex、Backtrack Limit 和取消继续保持 Pattern 级隔离；
- `notra-core` 单元、集成和 Doc Test 共 63 个通过。

Phase 2 已达到功能开发完成标准，下一步进入 Phase 3 Daily-use UI。

## 32.6 2026-08-15 Phase 3 Daily-use UI 主切片

已落地：

- 新增 `AnalyseService` 与 `run_analyse` Tauri Command，将活动文档、Pattern 和 Revision 映射到唯一 Rust Engine；
- 新增独立 `analysePanel.ts`，没有继续把 Pattern 状态和 Result 状态堆入现有 Find 状态机；
- Pattern Panel 已支持 Add、Update、Delete、Move、Order Sort、启用切换、类型、Case、Whole Word、Text/Line、Hide、样式、颜色、Comment 和 Group；
- Pattern 列表已显示 Hits 与 Pattern 级错误，单个错误不会丢弃其他有效结果；
- 新增第二个只读 Monaco Result Model，支持源行号开关、自动换行、字号和 Rust Resolve 后的 Segment Style；
- Result 行保存显式 Source Document/Line Mapping，双击可返回源行；
- Analyse Bookmark 使用独立 Decoration Collection，Run/Clear/文档切换不会删除用户书签；
- 已接入右侧工具栏与会话恢复；完整 Analyse Command 集成见 32.9；
- 前端通过 Run ID、Document ID、Model Version 和 Pattern Revision 拒绝陈旧结果；
- 普通浏览器环境不再因 Tauri Window API 停在启动画面，便于后续 UI 自动回归；Tauri Runtime 下的窗口与拖放行为保持不变。

本切片验证：

- `notra-core`：59 个单元测试、1 个 Engine 集成测试、3 个 Profile 集成测试及 Doc Test 全部通过，共 63 个；
- `notra-app --no-default-features`：8 个测试全部通过，包含 Tauri Request 校验和 Regex Result DTO 映射；
- Frontend：`tsc && vite build` 通过；
- 本地浏览器：页面有有效内容、无 Vite Error Overlay、重新加载后无新增 Console Error；Analyse 打开、Pattern 添加/排序、自动换行和字号交互通过；
- 当前工具链没有安装 `rustfmt`，因此本切片未执行 `cargo fmt --check`，不把环境缺失误报为代码验证成功。

Phase 3 的日常主链已经可用，随后进入 Phase 4 补齐 Profile UI、Recent、Auto Update/Cancel、Scroll Sync、Result Find、Copy/RTF/Save/Bind 和设置持久化；Windows 功能一致性对照仍在 Release 阶段执行。

## 32.7 2026-08-15 Phase 4 Full Workflow 主链

已落地：

- XML Profile 已通过 Rust Command 接入 Replace、Append、Prepend、Save，保留 Import 顺序和稳定 Pattern ID；
- Analyse 面板支持 Profile 文件选择、右侧面板拖放加载、最近 Profile 和 SQLite Session 恢复；
- Auto Update 使用 300 ms Debounce；每个 Run 注册独立 `CancellationToken`，取消按钮、模式修改和后续 Run 可以终止旧任务；
- Run ID、Document ID、Model Version 和 Pattern Revision 四重校验继续作为最终 Stale Result Gate；
- Source 与 Result 已接入可选双向滚动同步，双击 Result 仍执行显式回源和聚焦；
- Result Find 支持 Normal、Escaped、Regex、Regex Multiline、Case 和 Whole Word，并调用同一个 Rust Functional Matcher，不新增 JavaScript Regex 语义；
- 当前 Result 行显示去重后的 Matching Patterns；
- Result 支持 Plain Copy、Rich Copy、Text Save、RTF Save、Bind/Unbind File；绑定文件在每次有效 Result 更新后写入；
- 新增 Rust RTF Serializer，覆盖颜色表、Bold、Italic、Underline、Hidden、Unicode 和源行号；不要求与参考插件 RTF 字节完全一致；
- Rich Clipboard 的初始 RTF-only 回退已在 32.10 升级为 RTF/HTML/Plain 能力协商；
- Auto Update、Line Number、Word Wrap、Result Font、Scroll Sync、Bound Result Path 和 Recent Profiles 已纳入现有 SQLite Session Snapshot。

最新验证：

- `notra-core`：60 个单元测试、4 个集成测试及 Doc Test 全部通过，共 64 个；
- `notra-app --no-default-features`：11 个测试全部通过，覆盖 Run DTO、Profile Command、Result Find 和 RTF Command；
- Frontend：`tsc && vite build` 通过；
- 浏览器可见验收：Phase 4 控件全部渲染，页面无 Error Overlay、无新增 Console Error；自动换行与字号交互通过；
- `git diff --check` 与新增源码行尾空白检查通过；
- 当前 Rust 工具链仍未安装 `rustfmt`，因此没有伪报 `cargo fmt --check` 成功。

Phase 4 的桌面功能主链已经完成。后续 Phase 5 聚焦三平台 Release 验证、大文件路径、性能基准、Native Rich Clipboard 兜底和 Windows 功能差异分级，而不是追逐非功能性的逐行为一致。

## 32.8 2026-08-15 Phase 5 Large-file Hardening 主切片

已落地：

- 新增 `run_analyse_path` Tauri Command；大文件、未修改且存在本地路径时不再把全文通过 IPC 发送给 Rust；普通或 Dirty 文档继续使用文本快照，保证编辑内容优先；
- 路径执行复用唯一 `AnalyseEngine`，并验证普通文件、前置期望尺寸、2 GiB 安全上限和读取后尺寸，文件在读取期间变化时拒绝返回可能陈旧的结果；
- 超过 2000 行的结果由后端保存并按最多 2000 行逐批传给前端，前端每批检查 Run/Document/Revision Stale Gate，并在过期或异常时主动释放结果 Token；
- 新增 Release Benchmark Example，可生成确定性的 100/500/1024 MiB Fixture，并以同一 Normal + Lookbehind Regex Pattern 对比 owned 与 mmap；
- 新增机器可读 Benchmark 记录和复现说明，放在 `tests/analyse-compat/benchmarks/`。

当前 macOS arm64 Release Benchmark：

| Size | owned load / analyse / total | mmap load / analyse / total | Result |
|---:|---:|---:|---|
| 100 MiB | 24 / 61 / 85 ms | 6 / 57 / 64 ms | 100 matches / 100 lines，一致 |
| 500 MiB | 84 / 242 / 327 ms | 32 / 230 / 263 ms | 500 matches / 500 lines，一致 |
| 1024 MiB | 319 / 744 / 1063 ms | 72 / 500 / 576 ms | 1024 matches / 1024 lines，一致 |

决策：mmap 在 1 GiB 上端到端约快 46%，收益明确；但普通桌面文件可能被其他进程截断，直接映射可能触发进程级故障。当前生产路径继续采用安全 owned 读取，mmap 保持 Benchmark-only，待建立不可变快照或跨平台稳定性契约后再启用。这不会阻断功能一致性目标：`analyse_path` 已消除大文本 IPC 成本，大结果也已分批传输。

Phase 5 剩余项仅作为后续 Release 验证：三平台 Regex、Rich Clipboard、Windows Oracle 功能差异分级和发布包回归；它们不阻塞继续开发，也不要求非功能性的逐行为一致。

## 32.9 2026-08-15 Interaction / Command 功能补齐

按最终交互门槛反查实现后，补齐了不应推迟到 Release 的日常功能：

- Pattern 操作新增 Clear All、Enable/Disable All 和按当前选中 Pattern Group 的 Enable/Disable；双击行继续切换单 Pattern Enabled；
- 新增 Add Selection as Pattern，读取主 Monaco 非空选区，继承当前 Draft 的非文本属性，并插入到当前 Pattern 后；空选区给出明确提示；
- Enter Action 支持 Just Search、Update Line、Add Line；无选中 Pattern 时稳定走 Add，Draft 未变化时稳定走 Search；`Esc` 放弃未提交 Draft；
- Enter 决策拆为无 DOM 的纯状态机 `analyseState.ts`，由 `npm run test:analyse` 覆盖全部分支；
- Command System 增加独立 `Analyse` 分类，并注册 `analyse.togglePanel`、`analyse.addSelectionAsPattern`、`analyse.run`、`analyse.cancel`、`analyse.clear`、`analyse.openProfile`、`analyse.saveProfile`、`analyse.options` 和 `analyse.focusResult`；
- 当前 Working Pattern List 与 Enter Action 已纳入现有 SQLite Session Snapshot，重启不再丢失未显式保存到外部 XML 的工作配置。

本切片验证：

- `notra-app --no-default-features`：15 个测试通过；`notra-core` 单元、集成和 Doc Test 共 65 个通过；
- `npm run test:analyse` 与 `npm run test:keybindings` 通过；
- Frontend `tsc && vite build` 通过；仅保留仓库既有的 Vite 动态导入和大 Chunk Warning；
- 浏览器可见验收通过：新增控件完整渲染，Pattern Add、Group Disable/Enable、Enter 模式、空选区提示和 Clear All 生效，无错误覆盖层或已捕获 Console Error；
- `git diff --check` 通过；当前 Rust Toolchain 未安装 `clippy` 和 `rustfmt` 组件，因此没有伪报对应检查成功。

## 32.10 2026-08-15 Rich Clipboard 功能降级链

为满足“富文本功能一致、MIME 行为不必完全一致”的目标，已完成：

- 新增 Rust `write_result_html`，与 RTF Serializer 读取同一 `MergedLineResult` 和 `StyledSegment`；支持行号、前景/背景色、Bold、Italic、Underline、Hide、Unicode 和 HTML 转义；
- 新增 `serialize_analyse_html` Tauri Command，并复用 RTF Command 的 DTO 到 Logical Result 转换，避免两套样式语义；
- “复制富文本”根据 WebView 能力同时写入 `text/rtf`、`text/html` 和 `text/plain`；不支持 RTF 时仍可用 HTML 保留可见样式，只有所有富文本 MIME 都不可用时才退回纯文本；
- RTF 文件保存仍使用 RTF Serializer，不受 Clipboard MIME 降级影响。

最新回归：`notra-core` 61 个单元测试 + 4 个集成测试共 65 个通过，`notra-app` 15 个测试通过，Frontend 生产构建与 `git diff --check` 通过。三平台向 Word、TextEdit/Pages、LibreOffice 等目标应用的实际粘贴效果留到 Release Matrix 验证；失败的平台再局部增加 Native Adapter。

---

# 33. 结论

Notra 已经具备 Monaco、多文档、搜索、书签、命令、SQLite、Tauri 后台任务和三平台发布基础，因此 Analyse 的难点不在搭建编辑器外壳。

真正的核心是：

1. 覆盖 AnalysePlugin 的用户可见核心功能和工作流；
2. 提供跨平台可用、稳定且可替换的 Extended 和 Regex；
3. 正确合并跨行、多 Pattern、重复源行结果；
4. 确定性处理 Pattern 顺序、Text/Line、Hide 和 247 Style Limit；
5. 在 Auto Update、大文件和三平台环境下保证并发与性能正确。

实施遵循：

```text
Reference Feature
    -> Independent Design
    -> Implementation
    -> Functional Test
    -> Release Compatibility Compare
```

第一目标是建立可以被自动验证、长期维护、不会污染 MIT 边界的 Analyse Functional Core，并完整交付用户日常工作流。
