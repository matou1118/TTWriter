# 中文排版规范 / Chinese document conventions

WordCraft's Chinese support is real: the interface is fully translated, East Asian typography
(禁则, 中西文间距, 文档网格, 竖排, 中文数字) is implemented, and A4 follows the system locale.
This file is about the **conventions** — the parts that are Chinese-specific and that Word
itself does not do for you.

---

## 默认值：先确认再改

| 项 | WordCraft 的行为 | 要不要动 |
|---|---|---|
| 纸张 | 按系统地区（CLDR）：中文系统自动 **A4** | **不要动** |
| 边距 | 1 英寸（72pt） | 公文/论文要改，见下 |
| 正文字体 | Aptos（拉丁） | 中文文档要设 `fontEa` |
| 校对语言 | 跟随系统 | 视需求 |
| 首行缩进 | 0 | 中文正文一般设 2 字符 |
| 行距 | 1.15 倍 | 中文正文一般 1.5 倍或固定值 |

**不要**用 `layout.size` 强行设 A4 —— 已经是了，改了反而破坏地区跟随。

---

## 中文字体

Windows 自带的中文正文/标题字体（`fontEa` 用）：

| 字体 | 用途 | 备注 |
|---|---|---|
| 宋体 SimSun | 正式正文 | 最通用 |
| 等线 DengXian | 较新正文的默认 | Win10+ |
| 仿宋 FangSong | 公文正文（GB/T 9704 要求） | |
| 楷体 KaiTi | 公文正文/批注 | |
| 黑体 SimHei | 公文标题、一级标题 | |
| 微软雅黑 Microsoft YaHei | 屏幕阅读、标题 | |

```json
{"command": "format.set", "params": {"props": {"fontEa": "仿宋", "size": 16}}}
```

`font` 设拉丁字体，`fontEa` 设中文字体。中文文档通常两个都要。

**字体名用中文还是英文？** 两种都能查到——上游的字体库按 `name` 表存了本地化名
（SimSun 的中文名是「宋体」），可以在界面语言的字体下拉框里直接看到。

---

## 首行缩进 2 字符

中文排版的核心习惯。**用字符单位，不是磅**：

```json
{"command": "para.set", "params": {"props": {"indentFirstChars": 2}}}
```

`indentFirstChars: 2` = 缩进 2 个字符（随字号变化），`indentFirst: 24` = 固定 24 磅。
Word 的规则是 `indentFirstChars` 为 0 时会取消 `indentFirst`。

行距同理，中文正文常用：

```json
{"command": "para.set", "params": {"props": {"lineSpacing": {"rule": "multiple", "value": 1.5}}}}
```

---

## 公文（GB/T 9704—2012《党政机关公文格式》）

⚠️ **这是公开的国家标准，按标准实现是合法的。但不要抄 WPS 或 Word 的模板文件**——
那是别人的著作权内容。按标准自己写。

主要格式要求：

| 项 | 要求 | WordCraft 怎么设 |
|---|---|---|
| 版心 | 156 mm × 225 mm | `layout.margins` 配合 `layout.size`（A4 210×297） |
| 上白边 | 37 mm | `layout.margins {"top": 104.9}` (37mm ≈ 104.9pt) |
| 下白边 | 35 mm | `layout.margins {"bottom": 99.2}` |
| 左白边 | 28 mm | `layout.margins {"left": 79.4}` |
| 右白边 | 26 mm | `layout.margins {"right": 73.7}` |
| 标题 | 2 号小标宋（方正小标宋简体） | `format.set {"fontEa": "方正小标宋简体", "size": 22}` |
| 正文 | 3 号仿宋_GB2312 | `format.set {"fontEa": "仿宋", "size": 16}` |
| 行距 | 固定值 28-30 磅 | `para.set {"props": {"lineSpacing": {"rule": "exactly", "value": 28}}}` |
| 每页行数 | 每面 22 行，每行 28 字 | 由版心 + 行距推出 |
| 页码 | 4 号半角宋体阿拉伯数字，单页右空一字，双页左空一字 | `insert.pageNumber` + 页眉/页脚 |
| 一级标题 | 3 号黑体 | `styles.modify {"style": "Heading1", "chr": {"fontEa": "黑体", "size": 16}}` |
| 二级标题 | 3 号楷体 | `styles.modify {"style": "Heading2", ...}` |
| 层级 | 一、（一）1.（1） | `list.*` / `numbering` |
| 版记 | 文种、份号、密级、紧急程度、签发人、印发机关和印发日期 | 表格 + 段落 |
| 主题词 | 正文下空一行 | 段落样式 |

建模板的骨架：

```json
{
  "template": "blank",
  "commands": [
    { "command": "format.set", "params": {"props": {"fontEa": "仿宋", "size": 16}} },
    { "command": "para.set", "params": {"props": {"lineSpacing": {"rule": "exactly", "value": 28}} } },
    { "command": "text.insert", "params": {"text": "关于××××的通知" } },
    "caret.docEnd",
    { "command": "text.newParagraph" },
    { "command": "text.insert", "params": {"text": "各有关单位：" } }
  ],
  "save": ["gongwen.docx"]
}
```

标题（发文机关标志）通常是一个**红色宋体大字**加一条红色分隔线——用
`insert.horizontalLine` + 段落下框线做，不要用图片。

---

## 论文（GB/T 7713 / 7714）

**引用格式上游已经实现了**：

```json
{"command": "references.citationStyle", "params": {"style": "GB/T 7714"}}
```

其他可选：`APA`、`MLA`、`Chicago`、`IEEE`、`Harvard (Anglia)`、`ISO 690 (numerical)`、
`ISO 690 (author-date)`、`GOST`、`SIST02`、`Turabian`。

著录格式：先正文引用，再生成参考文献表：

```json
{"command": "references.citation", "params": {"source": {"author": "张三", "title": "共享空间研究", "year": "2021"}}},
{"command": "references.citationStyle", "params": {"style": "GB/T 7714"}},
{"command": "references.bibliography", "params": {}}
```

输出形如 `[1] 张三. 共享空间研究[M]. 2021.`

论文排版：

| 项 | 做法 |
|---|---|
| 题目 | `Title` 样式，`fontEa` 黑体，2 号，居中 |
| 一级标题 | `Heading1`，黑体，3 号 |
| 二级标题 | `Heading2`，黑体，4 号 |
| 正文 | `Normal`，宋体，小四（12pt），行距 1.5 倍，**首行缩进 2 字符** |
| 目录 | `references.toc {"levels": 3, "tabLeader": "dot", "hyperlinks": true}` |
| 页眉 | 论文题目；`insert.header` |
| 页码 | `insert.pageNumber {"align": "center"}` |
| 图表题注 | `references.caption`，格式「图 1」「表 2」 |

---

## 中文数字

上游实现了完整的中文数字体系（`crates/doc/src/cjk_numbers.rs`），**在编号里可用**：

| OOXML 名 | 例子 | 用途 |
|---|---|---|
| `chineseCounting` | 一、二、十二 | 章节序号（公文一级标题） |
| `chineseCountingThousand` | 一千二百三十四 | 财务大写 |
| `chineseLegalSimplified` | 壹拾、贰佰 | **金额大写（简体）** |
| `ideographLegalTraditional` | 壹拾、貳佰 | 金额大写（繁体） |
| `ideographDigital` | 〇一二三 | 年份 |
| `ideographZodiac` | 甲子、乙丑 | 干支编号 |
| `taiwaneseCounting` | 壹貳參（国语数字） | 台湾用法 |
| `koreanCounting` | 一、二 | 韩文 |

```json
{"command": "list.define", "params": {"levels": [
  { "format": "chineseCounting", "text": "一、%1" },
  { "format": "chineseCounting", "text": "（%2）%1" }
]}}
```

`%1` 是第 1 级的编号，`%2` 是第 2 级，依此类推。`text` 是编号模板，可以带标点。

> ⚠️ **命令文档不全**：`wordcraft-cli commands --json` 里 `list.define` 的 `format`
> 参数说明只列了 `decimal|lowerLetter|upperLetter|lowerRoman|upperRoman|decimalZero|ordinal|bullet|none`，
> **没列中文格式**。但实际接受上面这些 OOXML 名（`NumFormat::from_ooxml` 认全部）。
> 传错名字**不会报错**，会静默退回 `decimal`——所以**一定要 render 出来看**。

> ⚠️ **`text` 命令不可信**：`wordcraft-cli text` 提取的纯文本里，中文编号会显示成
> "1." 而不是"一、"（提取器用的是简化路径）。**但渲染是对的**——
> `wordcraft-cli render` 出来的页面上确实是「一、」「二、」。
> 列表编号**一律以渲染结果为准**。

---

## 排版细节：上游已经做好了

这些**不用**你操心，已经实现：

- **禁则处理**（行首不出现 `。，）」`，行尾不出现 `（「`）
- **中西文自动间距**（`autoSpaceDe` / `autoSpaceDn`）
- **标点压缩**（`overflowPunct`、`topLinePunct`）
- **文档网格**（`layout.documentGrid`）
- **竖排**（`layout.textDirection`）
- **合并字符 / 带圈字符**（`format.enclose`）
- **拼音指南**（`format.phonetic`，ruby 文字）
- **CJK 断词**（不按拉丁规则断）

**改这些之前先查文档**——你以为缺的东西可能已经在了。

---

## WordCraft 做不到的（中文相关）

诚实地说清楚，别承诺做不到的事：

- **没有中文拼写/语法检查**。`ProofLang` 没有中文变体，中文文本**不会有下划线**。
  中文稿要自己校对，或用别的工具。
- **没有拼音排序**。表格按姓名排序会按 Unicode 码位排，不是拼音。
- **没有简繁转换**。`zh-hant` 只是界面语言，**内容不会自动转**。
- **没有中英混排的智能断字**（西文单词中间断行可开 `wordWrap`，但没有中文分词）。

---

## 核对

中文文档尤其要**看渲染结果**，因为排版问题（文字溢出单元格、标题断页、字号不一致）
在结构上看不出来：

```bash
wordcraft-cli render gongwen.docx page1.png --page 1
```

然后**打开 PNG 看**。
