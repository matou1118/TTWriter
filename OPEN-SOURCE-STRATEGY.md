# 开源策略：与上游的关系

> 决策依据均来自上游仓库的原文与 GitHub API 实际查询（2026-10-21）。

---

## 一、直接回答

**要开源。但不是以"WordCraft 的一个分叉"的形式开源，而是以"中文层"的形式。**

同时：**绝大部分代码改动要推回上游，不留在自己的仓库里。**

原因不是情怀，是三个硬事实。

---

## 二、三个硬事实

### 事实 1：上游迭代速度 = 任何代码分叉的维护成本

| 事实 | 数据 |
|---|---|
| 上游发布频率 | v0.5.0(10-11) → v0.6.1(10-12) → v0.7.0(10-14) → v0.8.0(10-18) → **v0.9.0(10-21)** |
| 一次发布间隔的改动量 | v0.8.0→v0.9.0 = **278 commit / 267 文件 / 586 hunk** |
| 活跃度 | 15k star、4k fork、**1460 open issues** |

本仓库 rebase 到 v0.9.0 时，**我改的 5 个文件全部被上游动过**，其中 3 个是同一区域
（`props.rs` 新增了 `east_asian_layout`、`ligatures` 两个字段）。

如果中文版长期携带自己的 Rust 改动（中文校对、简繁表、拼音表、中文默认样式），
就是**每周一次的 rebase 冲突**。这不是能忽略的成本。

### 事实 2：上游明确接收语言类贡献 —— 所以"通用"的中文能力**应该**推上去

刚查的上游 PR（近 40 个）：

| PR | 标题 | 状态 |
|---|---|---|
| #3392 | **Add Korean support** | open |
| #3380 | **Traditional Chinese interface: draw hanzi to the Taiwan standard with an installed TC font** | open（作者 TobyWu666） |
| #3420 | Options: list the languages alphabetically by their own name | open |
| #3378 | M25: add a configurable default document zoom | open |

**Korean 支持和繁体中文字形都在上游的 PR 队列里。** 说明上游欢迎语言工作。

这带来一个**关键的战略推论**：

> 如果中文校对、简繁转换、拼音排序、中文字数这些**通用能力**都推回上游，
> 那么本仓库就只剩**中文专有的层**——**冲突面趋近于零**。

### 事实 3：商标条款对"分叉发布"有明确要求

`docs/brand/LICENSE-brand.txt` 原文：

> The ArtCraft name, wordmark and mark … are trademarks and are NOT open source.
>
> If you distribute or publish a modified version of WordCraft, or any work derived from it,
> you **must remove the ArtCraft Marks** from it (or replace them with your own) and
> **must not call it ArtCraft or present it as an ArtCraft product**.
> **You may say, in plain text, that your work is based on WordCraft by the ArtCraft team.**
>
> **A fork kept only to propose changes back to this repository may keep the marks unchanged.**

也就是说：

| 形态 | 能否保留 ArtCraft 标识 | 能否用 "WordCraft" 做产品名 |
|---|---|---|
| **只用来提 PR 的 fork** | ✅ 可以 | ✅ 可以（提议性质） |
| **对外发布的修改版** | ❌ **必须移除或替换** | ⚠️ 不能暗示是 ArtCraft 产品；营销/名称/域名/图标/应用商店里不得出现 ArtCraft |

> 另注：仓库里有 `docs/brand/artcraft-logo*.svg`、`artcraft-mark*.svg` 和
> `assets/wordcraft-1024.png` 等带品牌标识的图标，**对外发布前都要换掉**。

---

## 三、结论：两仓库结构

```
┌──────────────────────────────────────────────────────────┐
│  github.com/<你>/wordcraft            ← fork，只用来提 PR  │
│  （可以保留 ArtCraft 标识，因为只用于向上游提议）              │
│    feature/w-lang-eastasia     → PR                                  │
│    feature/cjk-word-count      → PR                                  │
│    feature/zh-hant-convert     → PR                                  │
│    feature/pinyin-sort         → PR                                  │
│    feature/regional-defaults   → PR                                  │
└──────────────────────────────────────────────────────────┘
                            │ 合并后
                            ▼
┌──────────────────────────────────────────────────────────┐
│  github.com/<你>/wordcraft-zh         ← 公开的中文层，无上游源码 │
│    skills/{wordcraft,gongwen,thesis,legal-docs}                │
│    presets/     中文默认样式与模板                                 │
│    build/       拉上游 + 打补丁 + 编译 + 出包                       │
│    README       "基于 WordCraft（ArtCraft 团队）的中文增强层"          │
└──────────────────────────────────────────────────────────┘
```

### 为什么公开仓库不复制上游源码

1. **零冲突** —— 没有需要 rebase 的代码，只在构建时拉指定 tag
2. **零商标风险** —— 公开仓库里没有 ArtCraft 标识，也不用改图标
3. **不重复上游** —— 上游源码在它自己的仓库里有唯一真相源
4. **skill 独立可用** —— 别人用别的排版工具也能用这些 skill

### 公开仓库该有什么

| 内容 | 许可 | 说明 |
|---|---|---|
| `skills/` | Apache-2.0 | 核心资产，与 WordCraft 解耦 |
| `presets/` | Apache-2.0 | 中文默认样式、模板、术语表 |
| `build/` | Apache-2.0 | 拉上游 → 打补丁 → 编译 → 打包 |
| `patches/` | Apache-2.0 | 上游不收的补丁（如有），以 patch 形式打 |

---

## 四、哪些改动推上游，哪些留在中文层

| 改动 | 通用性 | 归属 | 理由 |
|---|---|---|---|
| 中文字数按字计 | 通用（中日韩） | **上游 PR** | 已实现，补丁已备好 |
| `w:lang/@w:eastAsia` | 通用 | **上游 PR** | 通用缺陷，上游未修 |
| 简繁转换 | 通用（繁中用户也要） | **上游 PR** | 上游已有 `zh-hant` 界面却不能转内容，这个不一致本身就值得提 |
| 拼音排序 | 通用 | **上游 PR** | 通用排序能力 |
| **默认样式跟随地区/语言** | 通用 | **上游 PR** | 见下 ⭐ |
| `wordcraft` skill | 通用 | **上游 PR** | 直接回应上游 issue **#1302** |
| `gongwen` / `thesis` / `legal-docs` skill | 中文专有 | **中文层** | 国标是地区性的，上游不会做 |
| 中文校对规则与词典 | 中文专有 | **中文层** | 词典有授权问题，只能自己维护 |
| 中文默认字体（仿宋/黑体/楷体） | 中文专有 | **中文层** | |

### ⭐ 最值得推的一个新发现：让默认样式跟随地区

**上游已经有这个机制，只是没覆盖到样式。**

`file.options` 的 `region` 参数实测：

> `"region"?: string (a locale tag like pl-PL, saved: **the paper File › New starts on**,
> the default units and the decimal separator; auto: the system's)`

**纸张已经是按地区自动的**（中文系统自动 A4）。但**字体、字号、行距、首行缩进**仍硬编码在
`StyleSheet::builtin()`（`lang:"en-US"` + Aptos + 1.15 倍行距）。

把这个扩展成"**默认样式跟随地区/语言**"是**通用特性**（日本、韩国、越南同样需要），
形状和已有的 `region` 设置完全一致 —— **上游很难拒绝**。

一旦它被接受，中文版就只剩**"zh-CN 这个地区的默认样式是什么"**这一张配置表，
**一行 Rust 代码都不用留在分叉里**。

---

## 五、开源 vs 不开源

| | 公开 | 不公开 |
|---|---|---|
| 上游 PR 的可信度 | ✅ 有公开记录，合并率高 | ❌ 无法证明你做了什么 |
| skill 的复用 | ✅ 别人（别的排版工具）能用 | ❌ 浪费 |
| 你的中文校对词典/规则 | ⚠️ 公开=可被抄，但也能被改进 | ✅ 独占 |
| 署名与信誉 | ✅ 累积 | ❌ 无 |
| 商标义务 | 公开=必须移除标识 | 同样适用（"distribute or publish"含发布二进制） |

**建议公开**，理由：

1. **上游合并你的 PR 需要能看到你的工作**。这是事实 2 的前提。
2. **skill 才是长期资产**，而 skill 与 WordCraft 解耦，公开才能扩散。
3. **不公开并不能保护词典**——真正的保护是**不用有版权的词典**
   （现代汉语词典有版权；CC-CEDICT 是 CC BY-SA，可用但有署名/相同方式共享义务）。
4. 闭源中文版在法律上和公开版**义务相同**（"distribute or publish" 覆盖分发二进制）。

---

## 六、发布前的检查清单

### 商标与品牌
- [ ] 移除 `docs/brand/` 全部 ArtCraft 标识（logo / mark / wordmark）
- [ ] 替换 `assets/wordcraft-1024.png`、`.icns`、`.ico`、`.svg` 等带标识的图标（**自己原创**，不能改色描摹）
- [ ] 不用 "ArtCraft" 做名称、logo、域名、社交账号、应用商店条目的任何部分
- [ ] 不用 "WordCraft" 做产品名 / 暗示是 ArtCraft 产品（可以平文本写"基于 WordCraft（ArtCraft 团队）"）
- [ ] `docs/brand/LICENSE-brand.txt` 里关于"必须移除"的条款照做
- [ ] 保留 `NOTICE`、`LICENSE-MIT`、`LICENSE-APACHE`（上游要求）
- [ ] `ATTRIBUTION.md` 补齐自己新增的资产行（上游 `cargo xtask assets` 会校验）

### 许可与来源
- [ ] 每个新数据/资产记录：作者、来源、许可
- [ ] 简繁表：OpenCC（Apache-2.0）；拼音表：mozillazg/pinyin-data（MIT）—— 记入 `ATTRIBUTION.md`
- [ ] 确认没有引入任何 Microsoft / Adobe / WPS 的资产或模板

### 工程
- [ ] `cargo xtask ci` 全绿（fmt / clippy -D warnings / 测试 / 资产 / 分层 / wasm）
- [ ] 构建脚本固定上游 tag（不要跟 main）
- [ ] 公开仓库不含上游源码副本
- [ ] 第一个上游 PR 里把自己的条目加进 `contributors/people.toml`（按 AGENTS.md：只加自己的，在自己的 PR 里提交）

---

## 七、下一步

1. **决定公开仓库形态**：按第三节的两仓库结构（推荐），还是单仓库分叉
2. **提第一个上游 PR**：`w:lang/@w:eastAsia`（补丁已备好，在 `target-gnu/pr-patches/`）
   —— 顺带验证"公开 + 与上游联动"这条路走得通
3. **在 PR 里加自己的 `contributors/people.toml` 条目**
4. **再提**：`wordcraft` skill（回应 #1302）、中文字数、简繁转换、拼音排序、
   默认样式跟随地区
5. **公开仓库**：`wordcraft-zh`，只放 skills + presets + build

---

## 附：本次核实的出处

| 结论 | 出处 |
|---|---|
| 上游发布节奏 | GitHub API `/releases`（v0.5.0→v0.9.0 日期） |
| v0.8.0→v0.9.0 的改动量 | GitHub API `/compare/v0.8.0...v0.9.0`（278 commits / 267 files / 586 hunks） |
| 上游接收语言 PR | GitHub API `/pulls`，近 40 个中 #3392(Korean)、#3380(TC hanzi) |
| 商标条款 | `docs/brand/LICENSE-brand.txt` 全文 |
| 纸张按地区自动 | `crates/geom/src/region.rs`（LETTER 列表不含 CN）+ `file.options` 的 `region` 参数说明 |
| 默认样式仍硬编码 | `crates/doc/src/styles.rs` 的 `StyleSheet::builtin()` |
| 贡献者署名规则 | `contributors/people.toml` 头部注释 + `AGENTS.md` 的 Contributor credits 段 |
| 资产授权校验 | `AGENTS.md` + `cargo xtask assets` |
| 上游 issue #1302 | GitHub API `/issues/1302` |
