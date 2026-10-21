# WordCraft 中文版 — 二次开发方案

基于上游 `storytold/wordcraft` v0.8.0（MIT OR Apache-2.0）的简体中文增强版。
上游活跃度极高（日均多次提交），本仓库以**薄分叉 + 上游优先**策略维护。

---

## 一、两个参考源的对比结论

### 1. 上游 `storytold/wordcraft`（官方源）

| 项 | 事实 |
|---|---|
| 语言/架构 | 纯 Rust，无 Electron/Tauri，egui + wgpu，V8 分层 L0–L6 |
| 协议 | MIT OR Apache-2.0（保留双许可声明） |
| 品牌 | ArtCraft 商标**不可**随分叉分发，fork 必须移除 `docs/brand/` 标识 |
| 最新版本 | v0.8.0（2026-10-18），star 15k，fork 4k，open issues 1460 |
| 迭代速度 | v0.5.0(10-11) → v0.8.0(10-18)，一周三版 |
| **中文界面** | **官方已内置 `zh-hans`（简体中文）2203 条 + `zh-hant`（繁體中文）2203 条** |
| 覆盖率 | zh-hans 键集与 de/ja **完全一致**（实测 0 缺失）；ru 反而多 367 条 |
| 质量门 | `cargo xtask ci`：fmt + clippy + ~800 测试 + 资产授权 + 分层 + wasm |

**关键判断：上游已把"汉化"这件事做完了，而且做得比 cikw 版好。**
`crates/ui-egui/src/i18n/` 下 23 个语言目录，zh-hans 是**参考键集基准**（其他语言对齐它），
德语/日语/繁中条目数与它一字不差，测试 `i18n/tests.rs` 强制这一约束。

### 2. cikw.top「WordCraft v0.5.0 绿色便携汉化版」

| 项 | 事实 |
|---|---|
| 本质 | 上游 v0.5.0 的**重打包**，非独立分叉、无源码 |
| 版本落后 | v0.5.0，比上游 v0.8.0 **落后 3 个版本**（一周内的差距） |
| 体积 | 标称 68.78 MB；上游 v0.5.0 官方 portable zip 为 85.12 MB → 非官方重打包 |
| 中文能力 | v0.5.0 时上游 zh-hans 仅 1305 条（现已 2203 条），无 cikw 独有内容 |
| 声明的改动 | 更新日志第 5 条称"完善简体中文界面语言包适配"——但**上游 v0.5.0 release notes 里没有这条**，是站长改写的营销文案 |
| 风险 | 闭源重打包，无 checksum 溯源，二进制来源不可验证 |

**结论：cikw 版没有可复用的技术资产。** 它唯一"多做"的事是**把默认语言从 `auto` 改成中文**——
而这正是中文版真正该做的事，上游反而没做。

#### 1.1 cikw 版落后于上游的**中文专属**改动（关键）

cikw 站长的更新日志第 5 条写"完善简体中文界面语言包适配，**优化段落首行缩进等中文排版习惯的默认预设**"。
但实测上游版本线：

| 能力 | v0.5.0（cikw 打包的版本） | v0.6.1 起 | 上游 issue |
|---|---|---|---|
| 字符单位缩进 `w:firstLineChars` / `w:leftChars` | ❌ **无** | ✅ 有 | #550（2026-10-12 关闭，晚于 v0.5.0 发布 10-11） |
| zh-hans 条目数 | 1305 | 2203 | — |
| 禁则（kinsoku） | ✅ | ✅ | — |

即：**cikw 号称补上的"首行缩进"，恰恰是上游在 v0.5.0 之后才加、而它没有跟进的功能。**
它宣传的"中文排版习惯默认预设"，用的其实是 2026-10-11 之前的能力。
这坐实了"该做的是**追平上游 + 补上游没有的**，而不是照抄 cikw"。

#### 1.2 为什么不能用 cikw 版当基础

| 风险 | 说明 |
|---|---|
| 无源码 | 只有网盘里的绿色包，无法审阅、无法修 bug、无法跟进上游 |
| 版本倒退 | v0.5.0 落后 3 个版本，缺 #550（字符缩进）、#2397（中西文间距）等中文能力 |
| 不可溯源 | 68.78 MB vs 上游官方 portable zip 85.12 MB / MSI 71.02 MB —— **非官方重打包**，无 checksum 可校验 |
| 商标 | 重打包未移除 ArtCraft 标识（`docs/brand/LICENSE-brand.txt` 明令 fork 必须移除） |
| 不可持续 | 上游一周三版，闭源包永远追不上 |

### 3. 真正有价值的差距（实测，非推测）

这是本项目的存在理由。按"用户痛点 × 实现成本"排序：

| # | 差距 | 上游现状（实测） | 用户影响 | 成本 |
|---|---|---|---|---|
| **G1** | **新文档默认英语** | `crates/doc/src/styles.rs:475` 硬编码 `lang: "en-US"`、`BODY_FONT = "Aptos"`（:448）；`SectionProps::default()` 为 US Letter 612×792pt | 中文用户新建文档，字体/校对语言/纸张全是美式，必须手动改 | **低** |
| **G2** | **中文字数统计错误** | `crates/doc/src/lib.rs:1386` `count_words()` 按空白切词；中文无空格 → 整段算 1 词。**上游无对应 issue** | "字数"是中文排版核心指标，当前完全不可用 | **低** || **G3** | **无中文校对** | `crates/proof/src/lang.rs` 的 `ProofLang` 枚举无 `Zh`；`from_tag("zh-CN")` → `Other` | 中文文档无拼写/语法下划线 | **高** |
| **G4** | **无拼音排序** | `crates/doc/src/collate.rs` 仅拉丁字母表，无 pinyin。**上游无对应 issue** | 表格按姓名排序错乱（按 Unicode 码位） | **中** |
| **G5** | **无中文模板** | `crates/engine/src/sample.rs` 仅 `sample_document/letter/resume/report`；`cmd/file.rs:181` 分发 | 缺公文、简历、合同、论文格式 | **中** |
| **G6** | **中文排版内核已就绪** | 禁则 `layout/src/kinsoku.rs`、中西文间距、文档网格、竖排、中文数字 `doc/src/cjk_numbers.rs` **均已实现** | 已基本可用 | — |

> G6 说明上游的**排版内核**中文支持其实相当完整——这是它比 cikw 版更值得做中文版的原因，
> 也是我们**应该做薄分叉而不是重写**的依据。

### 3.1 关键实现钩子（已定位到具体行）

| 需求 | 落点 | 改动量 |
|---|---|---|
| ~~新文档用中文校对~~ | ~~`i18n/mod.rs:59` `editing_lang`~~ | ❌ **被上游守卫测试挡住**（见下） |
| ↳ 正确做法 | `crates/proof/src/lang.rs` 新增 `ProofLang::Zh` → `from_tag("zh-CN")` 返回它 → 再给 `zh-hans` 设 `editing_lang: Some("zh-CN")` | 中等，**与 G3 合并做** |
| ↳ 守卫测试 | `crates/ui-egui/src/i18n/tests.rs:1115` `the_editing_languages_are_the_ones_wordcraft_can_proof` 断言 `editing_lang` 只给可校对语言 | 必须同步更新 |
| 东亚字体 | `doc/src/props.rs:363` `font_ea` 字段已存在；`docx/src/write/props.rs:34` 已写 `w:eastAsia` | 只填值 |
| 页面 A4 | `doc/src/section.rs` `SectionProps::default()` 现为 US Letter 612×792 | 小改 |
| 中文字数 | `doc/src/lib.rs:1386` `count_words()` | 小改 + 回归测试 |
| 中文模板 | `engine/src/sample.rs` 加 `pub fn`，`cmd/file.rs:181` 加分支，`ui-egui/src/backstage.rs:272` 加列表项 | 纯增量 |

> **⚠️ G1 的一行方案不成立（实测推翻）**：
> `i18n/mod.rs` 的注释写明 `editing_lang` *"only when `wordcraft-proof` can proof the language"*，
> 且 `i18n/tests.rs:1115` 有守卫测试强制这一点（当前只允许 pt-br/pl/cs/tr/it）。
> 中文当前**无法校对**（`ProofLang` 无 `Zh`），所以**不能**直接给 `zh-hans` 设 `editing_lang`——
> 否则新文档会声称自己是 `zh-CN`，却没有对应校对能力。
>
> **正确路径**：G1 的语言部分**并入 G3**（先做 `ProofLang::Zh`），G1 只保留纯样式部分
> （东亚字体、A4、首行缩进），这些与校对能力无关，可以独立先做。

> **`ProofLang::Other` 是安全兜底**：中文当前映射到 `Other`，意味着不做拼写下划线、也不套用拉丁断词，
> **不会产生误报**。等 G3 落地再引入真正的 `Zh` 变体。

### 3.2 界面翻译覆盖率（实测）

`crates/ui-egui/src/i18n/` 下 23 个语言目录：

| 语言 | 条目数 | 键集 |
|---|---|---|
| zh-hans | **2203** | 基准（与 de/ja/zh-hant 完全一致，0 缺失） |
| zh-hant | 2203 | 与 zh-hans 一致 |
| de / ja | 2203 | 与 zh-hans 一致 |
| ru | 2571 | **比 zh-hans 多 367 条** |

进一步统计 `ui-egui` 内 830 个 `tl!()` 字面量键：**zh-hans 缺 30 个**（800/830 ≈ 96%），
缺失项集中在**恢复/回收站对话框**（`Recovery copy`、`Restore`、`Discard…`、`No recovery copies.` 等），
属新功能未跟译。**与上游自述的 "~95% of visible" 吻合。**

### 3.3 已实测发现的上游缺口（适合提 PR，勿在分叉里改）

#### (a) zh-hans 缺 18 个内置样式名 —— 但**不能**在分叉里直接补

界面截图实测：`zh-hans` 下**样式库显示英文** `Normal` / `No Spacing` / `Subtitle`
（`标题 1`/`标题 2`/`标题` 是翻译好的，因为它们恰好在词条里）。

实测 18 个内置样式名在 zh-hans **全部缺失**：
`Normal`、`No Spacing`、`Subtitle`、`Quote`、`Intense Quote`、`List Paragraph`、
`Table Grid`、`Default Paragraph Font`、`Emphasis`、`Strong`、`Hyperlink`、
`Footnote Text`、`Endnote Text`、`TOC Heading`、`Normal Indent`、`Normal Table`、
`Intense Emphasis`、`Intense Reference`。

各语言目录现状（18 个键全覆盖情况）：

| 已覆盖 18/18 | 缺失 0/18 |
|---|---|
| bg, cs, es, hr, hu, it, ko, pl, ru, sv, tr, vi | de, fr, ja, pt-br, sr, sr-latn, uk, zh-hant |

> **⚠️ 不能只改 zh-hans（已实测失败）**：上游把 `zh-hans` 当作**参考键集基准**，
> `crates/ui-egui/src/i18n/tests.rs` 有 5 条守卫测试要求
> `de` / `pt-br` / `uk` / `sr` / `sr-latn` 的键集与 `zh-hans` **完全相等**，
> `et` / `es` / `tr` / `it` 必须是它的**超集**。
> 单独给 zh-hans 加词条 → 5 条测试直接挂（`is_subset` 失败）。
>
> **正确做法**：这是上游该修的（要么给所有 22 个目录补齐这 18 个键，
> 要么把 `zh-hans` 从"基准"降级为普通语言）。**在本分叉里不要动**——
> 改动会与上游持续冲突，且要维护 22 份翻译。
> 建议向上游提 issue/PR。

#### (b) zh-hans 缺 30 个 `tl!()` 字面量键（上游新功能未跟译）

集中在**恢复/回收站**对话框（`Recovery copy`、`Restore`、`Discard…`、
`No recovery copies.`、`Continue without restoring`）与**交叉引用**（`Reference type:`、
`For which item:`、`Insert reference to:`）。实测这些键在 v0.7.0 的 zh-hans 里**也不存在**
→ 是上游新增字符串后未跟译，非本分叉引入。

#### (c) ⭐ `w:lang/@w:eastAsia` 被静默丢弃 —— **最高价值的可提 PR 项**（已实测复现）

**这是本次调研最有价值的发现，且上游 v0.9.0（今天发布）仍未修。**

Word 为中文文档写的是三元组：
```xml
<w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/>
```
WordCraft 的模型**只有两个槽位**：
- `CharProps.lang` → `w:lang/@w:val`
- `CharProps.lang_bidi` → `w:lang/@w:bidi`
- `w:lang/@w:eastAsia` → **无对应字段，读取时丢弃，写回时丢失**

**实测复现**（本仓库 `target-gnu/make-lang-ea-docx.py`）：

1. 构造含 `<w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/>` 的最小 docx
2. `wordcraft-cli inspect` → 读出的 run 只有 `{"lang":"en-US","langBidi":"ar-SA"}`，**`eastAsia` 消失**
3. `wordcraft-cli convert in.docx out.docx` 往返后 `document.xml` 里是
   `<w:lang w:val="en-US" w:bidi="ar-SA"/>` → **`w:eastAsia="zh-CN"` 彻底丢失**

**为什么这很要紧（中文场景）**：
- `w:eastAsia` 是 Word 判定"这段是东亚文本"、用哪个校对器/字体/换行规则的依据
- 丢了它，**用 Word 打开 WordCraft 保存的中文文档，中文段落会按西文规则校对**
  （拼写检查在中文段落上误报、字体回退到西文字体、Word 的中文版式选项失效）
- 属于上游 issue **#1034** 描述的同一类问题（"Saving a .docx silently drops settings
  the model doesn't represent"），但**目前没有独立的 issue 提到 eastAsia**

**修复成本极低**（照抄 `lang_bidi` 的现成模式，4 处）：
| 文件 | 改动 |
|---|---|
| `crates/doc/src/props.rs` | 加 `pub lang_ea: Option<String>`（`lang_bidi` 旁边），并加进 `overlay_fields!` / `drop_same_fields!` 列表 |
| `crates/docx/src/read/props.rs:227` | `"w:lang"` 分支加读 `w:eastAsia` |
| `crates/docx/src/write/props.rs:109` | `w:lang` 写出加 `w:eastAsia` 属性 |
| `crates/docx/tests/east_asian_language.rs` | 新增往返测试（照 `bidi.rs` 写法） |

> **✅ 已实现并验证（commit `8c3be92`）**：
> - `wordcraft-cli inspect` 现在报 `"langEa": "zh-CN"`（修复前该字段不存在）
> - `wordcraft-cli convert` 往返后 `document.xml` 里是
>   `<w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/>`（修复前 `w:eastAsia` 丢失）
> - `cargo test --release --workspace`：**除 2 个预先存在的失败外全绿**
>   （那 2 个是 `wordcraft-render` 的彩色 emoji 测试，本机 `C:\Windows\Fonts` **没有装彩色 emoji 字体**，
>   已用 `git stash` 在**纯上游代码**上复现确认与本次改动无关）

> **上游 v0.9.0（2026-10-21 发布）仍未修**——已核对 v0.9.0 的 `props.rs` / `read/props.rs` /
> `write/props.rs`，`lang_ea` 仍不存在。**这是当前最高价值的可提 PR 项。**

**为什么归 A 类（可 PR 上游）**：这是**通用缺陷**，不只是中文问题（日文/韩文同理），
上游没有理由拒绝；修好对本分叉也是必须的前提（否则中文文档存出去就是坏的）。

> **顺带**：`w:lang/@w:eastAsia` 修好后，`ProofLang` 也可以从 `w:lang` 的三个属性里
> 正确解析出东亚语言 → 这是 **G3 中文校对** 的前置条件（见 3.4）。

### 3.4 G3 的正确切法（基于 (c) 的发现）

原本以为 G3 要"从零做中文校对"，但 (c) 揭示了更准的路径：
**先修 `w:lang/@w:eastAsia` 的模型缺口**，再让 `ProofLang` 认识中文。

```
1. 加 CharProps.lang_ea + 读/写 w:lang@w:eastAsia      ← (c)，A 类，可 PR 上游
2. ProofLang 加 Zh 变体，from_tag 认 zh/zh-CN            ← A 类
3. proof_runs 从 run 的 lang_ea 取中文校对语言            ← A 类
4. LANGUAGES 里 zh-hans 的 editing_lang = Some("zh-CN")  ← B 类（此时才合法）
5. 中文词典 + 分词 + 语法规则                            ← A 类，最重
```

第 1 步是**所有后续步骤的前提**：没有它，WordCraft 根本不知道一段文本是中文。

### 4. 上游已有但未合并的中文相关 issue（可摘取）

- #3191 泰中缩进体系 · #1888 中文输入法文本重叠 · #2491 目录域中文乱码
- #3384 自动拼音指南 · #1245 WASM 内嵌简体中文字体 · #3225 中文窗口标题字体混排
- #2061/#1969 macOS 原生 PingFang SC

---

## 二、技术方案

### 架构决策：薄分叉，不重写

| 方案 | 结论 |
|---|---|
| 闭源重打包（cikw 路线） | ❌ 落后版本、无源码、无法跟进上游、无法修内核 bug |
| 完整重写中文版 | ❌ 重复实现 egui/wgpu/OOXML 排版内核，数千人月，纯亏损 |
| **薄分叉 + 上游优先** | ✅ **采用**。内核改动尽量以 PR 推回上游，自己只保留"中文默认值"层 |

**核心原则：所有改动分两类**

- **A 类（上游可接受）**：G2 字数统计、G4 拼音排序、G3 中文校对 → 写成通用能力，PR 回上游。
  好处是既修了中文版，又拿到上游信誉与持续同步。
- **B 类（中文版专属）**：G1 默认值、G5 中文模板 → 留在本仓库，通过配置/补丁层实现，
  与上游冲突面最小。

### B 类实现方式（关键：不污染内核）

新增一个 `crates/ui-egui/src/zh/` 特性层，或更轻量：直接改 `LANGUAGES` 表 + 新增
`zh` 偏好文件。上游 i18n 设计已经预留了钩子：

```rust
// crates/ui-egui/src/i18n/mod.rs
LangInfo {
    code: "zh-hans",
    name: "简体中文",
    source: include_str!("zh-hans.tsv"),
    prefer_hans: true,
    editing_lang: None,        // ← G1 钩子：改为 Some("zh-CN")
    catalog: OnceLock::new(),
}
```

`editing_lang` 的语义正是"新空白文档用它书写"——上游目前只给 `pl` 用了它。
中文版把 `zh-hans` 的 `editing_lang` 设为 `Some("zh-CN")`，
再让 `crates/doc/src/styles.rs` 的默认值按界面语言取，即为 G1 的最小实现。

### G2 中文字数统计（最小正确实现）

现有：
```rust
pub fn count_words(s: &str) -> usize {
    s.split(|c: char| c.is_whitespace() || c == para::OBJ)
        .filter(|w| w.chars().any(char::is_alphanumeric)).count()
}
```
中文无空格，整段只算 1 词。修正为**中文字符逐字计数 + 西文按词计数**（Word 中文版的实际口径）：

```
字数 = CJK 字符数 + 西文单词数
```
落点在 `crates/doc/src/lib.rs`，UI 侧 `review.wordCount` 的 `words` 字段复用，
并在 `zh-hans.tsv` 增加"字数"标签（现有目录里只有"Word Count 单词统计"/"字符数"，需补"字数"）。

### G3 中文校对（成本最高，需分阶段）

上游 `ProofLang` 机制清晰：枚举 + `from_tag` + 词典 + 语法规则 + 分词规则。
中文需要**分词**才能查错，纯 Hunspell 方案不适用（上游多语言走 Hunspell 是因为西文有词边界）。

推荐路径（按成本递增）：
1. **P0 形态学检查**：全角/半角混用、直角引号、中英文空格、重复标点、常见误用词
   （"的地得"、"在再"、"做作"）——规则表驱动，无需分词，收益立竿见影
2. **P1 词典法**：接入分词后的词表比对（可用开源 CC-CEDICT / 词频词典）
3. **P2 语法规则**：针对中文的句式规则（主语缺失、动宾搭配）

> 建议**先做 P0**。中文校对是唯一能让用户"感觉像中文 Word"的功能，但也是最容易做砸的，
> 宁可 P0 做扎实再往上走。

### G4 拼音排序

`crates/doc/src/collate.rs` 现有结构是"每语言一张字母表 → 主权重 + 次权重"。
中文需扩展为**汉字 → 拼音 → 权重**：
- 轻量方案：内置 GB2312 常用字拼音映射表（~6763 字），生成排序键
- 完整方案：完整 Unicode CJK 拼音表

建议：常用字表先行，表数据放 `crates/doc/src/pinyin.rs`（生成代码，非运行时依赖）。

### G5 中文模板

上游模板在 `crates/engine/src/sample.rs` + `crates/engine/src/io.rs` 注册。
新增：公文（GB/T 9704 格式）、中文简历、合同、论文（含目录/页眉/参考文献）。
每个模板 ≈ 数十行 `Paragraph`/`Table` 构造代码，**纯增量、零侵入**。

### 打包与分发

- 上游 `packaging/windows/package.ps1` + `wordcraft.wxs`（WiX）已就绪
- 中文版需：独立产品名（**不可**用 ArtCraft/WordCraft 商标）、独立图标
  （`crates/ui-egui/src/icons.rs` 需**原创**绘制，AGENTS.md 明令禁止临摹 MS 图标）
- 保留 `NOTICE` / `ATTRIBUTION.md`，MIT+Apache 双许可声明照抄
- 去掉 `docs/brand/` 全部 ArtCraft 标识

---

## 四、同步策略（上游一周三版，这是最大的持续成本）

**现状**：本仓库已 **rebase 到 v0.9.0**（2026-10-21 发布）。
v0.8.0 → v0.9.0 之间上游有 **278 个 commit**、267 个文件、586 个 hunk。

**rebase 记录（已完成）**：
- 拉取上游：`git fetch --tags upstream`（走 `gh-proxy.com` 代理，已配置全局 `insteadOf`）
- 用 `git format-patch` 生成 **2 个独立补丁**，并验证它们在 **v0.9.0 和 main 上都干净应用**
  - `target-gnu/pr-patches/0001-fix-doc-count-East-Asian-characters-as-words-G2.patch`
  - `target-gnu/pr-patches/0002-fix-docx-keep-a-run-s-East-Asian-language-w-lang-w-e.patch`
- PR 描述已写好：`target-gnu/pr-patches/PR-1-eastasia-language.md` /
  `PR-2-chinese-word-count.md`
- 本仓库现在的代码差异只有 **152 行**（7 个文件），全部可逐行解释
- v0.9.0 上验证通过：CLI 报 `words: 22`；往返后 `w:eastAsia="zh-CN"` 保留

> **⚠️ 踩过的坑（下次 rebase 务必照做）**：
> v0.9.0 给 `CharProps` **新增了 `east_asian_layout` 和 `ligatures` 两个字段**。
> 如果用 `git checkout <旧commit> -- <file>` 整文件回贴，
> 会把上游这 278 个 commit 里同文件的其他改动**一起覆盖掉**
> （实测 `props.rs` 少 34 行、`lib.rs` 少 63 行、`read/props.rs` 少 37 行……）。
> **正确做法**：`git checkout v0.9.0 -- <file>` 恢复基线后，**逐处手工打补丁**，
> 再用 `git diff v0.9.0 --stat` 核对行数是否合理（本次：152 行）。

**建议的节奏**：
1. **先提 PR，再 rebase**（顺序很重要）
   - PR #1：`w:lang/@w:eastAsia`（通用缺陷，上游没理由拒绝）
   - PR #2：中文字数统计（通用改进）
   - 上游合并后，本分叉**删掉这两个补丁** → 分叉只剩"中文默认值"，冲突面趋近于零
2. **不要**在分叉里长期维护这两个补丁
3. rebase 频率：**每周一次**（对齐上游发布节奏），而不是攒着

---

## 五、目标与里程碑

### M0 — 地基（已完成）
- [x] 拉取上游源码（GitHub 直连被墙，走 codeload tarball / gh-proxy 通道）
- [x] 建立 git 仓库，打基线 tag；`upstream` remote 已接好
- [x] `.gitattributes` 固定 LF，避免 Windows 换行污染 diff

### M1 — 可编译可运行（已完成）
- [x] 装 Rust 工具链（rustc 1.99.1）
- [x] **网络打通**：见下"环境配置"
- [x] `cargo build --release -p wordcraft-cli` 跑通
- [x] `cargo build --release -p wordcraft` 跑通（GUI，77.9 MB）
- [x] `cargo test --release --workspace`（除 2 个**预先存在**的彩色 emoji 失败——本机无彩色 emoji 字体）
- [x] 中文界面截图验证（`target-gnu/ui-zh-hans.png`）
- [x] **rebase 到 v0.9.0**（见第四节）
- [ ] `cargo xtask ci` 全绿（fmt/clippy/800 测试/分层/**wasm**）——wasm 部分还没验证，提 PR 前需补

#### 环境配置（本机实测，已落盘）

| 障碍 | 解决方案 | 状态 |
|---|---|---|
| `github.com:443` 直连被墙（git 协议） | `git config --global url."https://gh-proxy.com/https://github.com/".insteadOf "https://github.com/"` | ✅ |
| `crates.io` 索引极慢（~10-20s/crate） | `~/.cargo/config.toml` 换 `rsproxy.cn` 稀疏索引（实测 221ms） | ✅ |
| `craft-libs` 两个 git 依赖取不到 | 走上面的 git 代理；两处 `rev` 已验证可拉 | ✅ |
| **MSVC 链接失败 `LNK1104: msvcrt.lib`** | 本机 VS2022 在 `D:\AzerothCore\env\VS2022`，**缺 `lib` 目录**（无 `libcmt.lib`）；Windows SDK `um\x64` 也缺 `msvcrt.lib`；**全盘 C:\ 搜索无 `msvcrt.lib`** | ❌ 不可用 |
| GNU 工具链缺 `dlltool.exe` | `--profile minimal` 不带 `rust-mingw`；需 `--component rust-mingw` | ✅ 已装 |
| `dlltool` 报 `CreateProcess` | GNU `dlltool` 生成导入库时要调汇编器 `as.exe`，Rust 的 `rust-mingw` **不含** `as` → `choco install mingw -y` | ✅ 已装 |
| Rust 工具链缺失 | `rustup` 已装 stable-msvc **与** stable-gnu | ✅ |

> **MSVC 确认不可用**：`msvcrt.lib` 在整个 C 盘都不存在，Windows SDK 组件装漏了。
> 这是**环境问题，不是项目问题**。**当前用 GNU 工具链**（自带 mingw，不依赖 VS）。
> 正式发布若要用 MSVC（体积/性能/签名更顺），需先在 VS Installer 里补装
> "MSVC v143 生成工具" + Windows SDK 10.0.26100 的 C 库组件。

**本机构建命令**（已封装为 [`build.ps1`](build.ps1)，直接 `powershell -File build.ps1` 即可）：

```powershell
powershell -ExecutionPolicy Bypass -File build.ps1 build --release -p wordcraft-cli
```

### M2 — 中文默认层（B 类，风险最低，收益最快）
- [ ] 默认样式中文化：东亚字体（宋体/等线）落位到 `font_ea`
- [ ] 页面默认 A4（595×842pt）+ 中式页边距
- [ ] 首行缩进 2 字符（`indent_first_chars = 2.0`）
- [ ] `zh-hans` 的 `editing_lang` → `Some("zh-CN")` — **必须等 M4**（上游守卫测试只允许可校对语言）

### M3 — 内核能力（A 类，可 PR 上游）
- [x] **G2 中文字数统计** — 已实现并验证，补丁已备好
- [x] **`w:lang/@w:eastAsia` 不再丢失** — 已实现并验证，补丁已备好
- [x] **rebase 到 v0.9.0**，2 个补丁在 v0.9.0 和 main 上都干净应用
- [ ] **提交 2 个 PR 给上游**（描述已写好，见 `target-gnu/pr-patches/`）
- [ ] G4 拼音排序

> **G2 已完成（实测）**：`crates/doc/src/lib.rs::count_words()` 改为"汉字逐字计 + 西文按词计"。
> 验证（`wordcraft-cli run --cmd review.wordCount`）：
> `这是一段中文文本，用于验证字数统计功能是否正确。` → 修复前 `words: 1`，修复后 `words: 22`
> （24 字符含 2 个标点，标点不计词），与 Word 的口径一致。**v0.9.0 上复验通过。**
> 回归测试：`crates/doc/src/tests.rs::word_count_counts_east_asian_characters_as_words`
> （同时覆盖日语假名、韩文音节、混合中英文）。

> **`w:lang/@w:eastAsia` 已完成（实测）**：`CharProps.lang_ea` + 读 + 写 + 往返测试。
> v0.9.0 上复验：`convert` 往返后 `document.xml` 里
> `<w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/>`，**属性完整保留**。

### M4 — 中文校对（G3）
- [ ] P0 形态学/常见误用规则
- [ ] P1 词典 + 分词
- [ ] P2 语法规则

### M5 — 模板与打包
- [ ] G5 中文模板集
- [ ] 独立品牌 + 图标 + Windows 安装包

---

## 六、风险

| 风险 | 缓解 |
|---|---|
| **上游迭代极快**（一周三版，v0.9.0 已领先 287 commit） | 薄分叉 + A/B 分类；**先提 PR 再 rebase**（见第四节）；`upstream-v0.8.0` tag 记基线；每周 rebase 一次 |
| **GitHub 直连被墙** | 已验证 codeload tarball 通道可用；`gh-proxy.com` 可用；gitee 可作镜像 |
| **商标/品牌** | ArtCraft 标识不可随分叉分发；中文版需自有名称与**原创**图标 |
| **AGENTS.md 硬约束** | 禁 MS/Adobe 图标资产、禁 `unwrap/panic`（`clippy::unwrap_used` deny）、禁手写 JS/TS、clean-room 不得抄 MS 措辞 |
| **中文校对质量** | 分阶段，规则表驱动 + 回归测试，不追求一步到位 |
| **本机无 Rust** | rustup 已下载安装中；VS2022 + Win SDK 10.0.26100 就绪 |
| **磁盘** | 剩余 ~85 GB，够用；`cargo` target 目录建议单独放 |

---

## 七、立即可执行的下一步

```powershell
cd D:\App\WordCraft
powershell -ExecutionPolicy Bypass -File test-all.ps1     # 完整测试：编译 → 测试 → 质量门 → skill 校验 → 排版冒烟
powershell -ExecutionPolicy Bypass -File build.ps1 build --release -p wordcraft-cli
```

### 建议顺序

1. **提 PR #1：`w:lang/@w:eastAsia`**（已实现，commit `8c3be92`）
   - 补丁与 PR 描述在 [`upstream-prs/`](upstream-prs/)
   - 上游 v0.9.0 仍未修，通用缺陷，不限中文
   - 附上复现步骤（`inspect` / `convert` 往返对比）和回归测试
2. **提 PR #2：中文字数统计**（已实现，commit `f19dbf7`）
3. **rebase 到上游最新**（v0.9.0 之后可能又有新版本）
4. 然后才做 **G1 的纯样式部分**（东亚字体 + A4 + 首行缩进 2 字符）——
   注意 `editing_lang` 那部分**必须等 G3**（上游守卫测试只允许可校对语言）
5. **G4 拼音排序**、**简繁转换**（无上游 issue，纯增量）
6. **G5 中文模板**

> M1 全量门 `cargo xtask ci` 的 **wasm 部分**尚未验证——提 PR 前需补（需装 trunk）。
