---
name: thesis
description: 按 GB/T 7713（论文编写规则）排版中文学位论文，参考文献用 GB/T 7714 著录，并生成符合规范的封面、页眉、目录、图表编号、页码。Use when the user wants a 学位论文/毕业论文/期刊论文/科技报告 laid out to the Chinese national standard, mentions GB/T 7713、GB/T 7714、论文排版、参考文献格式、论文封面, or asks for a Chinese academic paper to be formatted with a table of contents, figure/table captions and a reference list. This skill is the LAYOUT layer only - it formats content that is already final and never writes or edits the text. For drafting the paper use a writing skill; for WordCraft's general command set use the wordcraft skill.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary on PATH. No network access needed.
metadata:
  author: WordCraft Chinese edition
  version: "1.0"
  standard: GB/T 7713-2006, GB/T 7714-2015
---

# 论文排版（GB/T 7713 / GB/T 7714）

把**已定稿**的中文学术论文排成符合国标的 `.docx` / PDF。

## 铁律

1. **不改内容。** 只管字体、字号、段落、页码、标题层级、图表编号、参考文献格式。
   内容要改 → 交回写作环节。
2. **不补文献。** 参考文献条目必须由作者提供；缺项标 `[待补参考文献]`。
   **不要凭印象编造作者、刊名、年份、卷期页码。**
3. **不编造引用。** 引用与参考文献表必须一一对应。
4. **排完必须看。** 渲染成 PNG 并**真的打开看**——目录页码、图表编号、
   参考文献悬挂缩进这类问题在结构上看不出来。

## 工作流

```
① 内容定稿  →  ② 页面与样式  →  ③ 封面/目录  →  ④ 页眉页码  →  ⑤ 图表与引用  →  ⑥ 版面检查
```

### ① 内容定稿

先确认这几项，缺了标 `[待补充]`：

| 项 | 说明 |
|---|---|
| 题目 | 论文题目（封面与页眉都要用） |
| 作者、单位、学号/专业、导师、日期 | 封面要素 |
| 摘要 + 关键词 | 中文摘要，关键词 3–8 个 |
| 目录 | 一级、二级标题（章 → 节） |
| 正文 | 章节结构完整 |
| 图表 | 每个图有题注、每个表有题注；图题在图下、表题在表上 |
| 参考文献 | 按 GB/T 7714 的条目类型准备 |
| 致谢、附录、攻读学位期间成果 | 可选 |

### ② 页面与样式

**A4（210×297mm）**，页边距（常见约定，按学校要求调整）：

| 边 | 常用值 | 毫米 | 磅 |
|---|---|---|---|
| 上 | 25 | 25 | 70.9 |
| 下 | 25 | 25 | 70.9 |
| 左（装订） | 30 | 30 | 85.0 |
| 右 | 20 | 20 | 56.7 |

**样式**（正文中文宋体、标题黑体是通行做法，各校要求不同，以用户/学校模板为准）：

| 要素 | 字体 | 字号 | 行距 | 其他 |
|---|---|---|---|---|
| 题目 | 黑体 | 二号/三号 | 固定值 22–24 磅 | 居中 |
| 一级标题（章） | 黑体 | 三号 | 固定值 22 磅 | 居中，段前段后各半行 |
| 二级标题（节） | 黑体 | 四号 | 同上 | 左空二字 |
| 三级标题 | 黑体 | 小四 | 同上 | 左空二字 |
| 正文 | 宋体 | 小四（12pt） | 固定值 20 磅 | **首行缩进 2 字符** |
| 图题 | 宋体 | 五号（10.5pt） | 单倍 | 图下方居中 |
| 表题 | 宋体 | 五号 | 单倍 | 表上方居中 |
| 参考文献 | 宋体 | 五号 | 单倍 | **悬挂缩进**（左空 2 字符，右空 0） |
| 页眉 | 宋体 | 小五 | — | 论文题目 |
| 页码 | 宋体 | 小五 | — | 居中，正文部分罗马数字或阿拉伯数字 |

```json
{ "command": "layout.margins", "params": { "top": 70.9, "bottom": 70.9, "left": 85.0, "right": 56.7 } },
{ "command": "styles.modify", "params": { "style": "Normal",
  "chr": { "font": "Times New Roman", "fontEa": "宋体", "size": 12 },
  "para": { "lineSpacing": { "rule": "exactly", "value": 20 }, "indentFirstChars": 2 } } }
```

**参考文献的悬挂缩进**（GB/T 7714 的著录格式）：

```json
{ "command": "para.set", "params": { "props": { "indentLeftChars": 2, "indentFirstChars": -2 } } }
```

`indentFirstChars: -2` = 负值 = 悬挂缩进，与 `indentLeftChars: 2` 配对。

### ③ 封面与目录

**封面**用 `insert.coverPage`：

```json
{ "command": "insert.coverPage", "params": {
  "title": "基于……的……研究",
  "subtitle": "——以……为例",
  "author": "张三" } }
```

封面后**分节符**，让封面/摘要/目录/正文各有自己的页眉页码。

**目录**：

```json
{ "command": "references.toc", "params": {
  "levels": 3, "title": "目 录", "pageNumbers": true,
  "rightAlign": true, "tabLeader": "dot", "hyperlinks": true } }
```

目录是**域**——内容改了要 `references.updateFields` 刷新。

### ④ 页眉页码

```json
{ "command": "insert.header", "params": { "text": "基于……的……研究" } },
{ "command": "insert.footer", "params": { "preset": "blankThree" } },
{ "command": "insert.pageNumber", "params": { "position": "bottom", "align": "center" } }
```

正文页码通常从正文首页重新起算（`layout.pageNumberFormat`），前置部分用罗马数字。

### ⑤ 图表与引用

**题注**：

```json
{ "command": "references.caption", "params": {
  "label": "Figure", "text": "研究框架", "position": "below" } }
```

`label` 用 `Figure`/`Table`/`Equation` 或任意标签；`chapter` + `separator` 可做
"图 1-1" 这种按章编号。

**引用（GB/T 7714）**：

```json
{ "command": "references.citation", "params": {
  "source": { "author": "张三", "title": "共享空间研究", "year": "2021",
              "journal": "建筑学报", "volume": "3", "issue": "2", "pages": "45-52" } } },
{ "command": "references.citationStyle", "params": { "style": "GB/T 7714" } },
{ "command": "references.bibliography", "params": { "title": "参考文献" } }
```

GB/T 7714 是**顺序编码制**：按正文首次引用顺序编号，参考文献表用 `[1] [2]`。
`references.citationStyle` 的可选值（实测）：

```
APA | Chicago | GB/T 7714 | GOST (name sort) | GOST (title sort) | Harvard (Anglia)
| IEEE | ISO 690 (author-date) | ISO 690 (numerical) | MLA | SIST02 | Turabian
```

其他标准也可以：`GB/T 774`、`GB/T 7714`、`ISO 690`（数字顺序制/著者-出版年制）。

> ⚠️ **参考文献是硬伤，而且上游的 GB/T 7714 实现有两处与标准不符**（实测，已知缺口）：
>
> 1. **期号（issue）被丢弃。** 上游 GB/T 7714 的条目模板是
>    `"{journal}, {year}⟦, {volume}⟧⟦: {pages}⟧"` —— **没有 `{issue}` 槽位**。
>    标准要求 `刊名, 年, 卷(期): 页码.`，所以实际输出是
>    `建筑学报, 2021, 3: 45-52.` 而**少了 `(2)`**。**要自己补期号。**
> 2. **标题恒为英文 "References"。** `bib_title` 硬编码在样式表里。
>    `references.bibliography` 接受 `title` 参数，**传 `"参考文献"`** 覆盖。
>
> 除此之外，类型码（`[J]` `[M]` `[D]` `[C]` `[EB/OL]`）、标点、责任者"等/et al."规则
> 都要**逐条人工核对**——引用格式错了在评审里会被直接指出。

### ⑥ 版面检查（必做）

```bash
wordcraft-cli render thesis.docx page1.png --page 1
```

**打开 PNG 逐项核对**：

- [ ] 封面要素完整、无页眉页码
- [ ] 目录层级正确、页码对齐、有点线前导符
- [ ] 正文首行缩进 2 字符
- [ ] 图题在图下、表题在表上、编号连续
- [ ] 参考文献悬挂缩进正确、编号与正文引用一致
- [ ] 页眉是论文题目、页码位置和编号方式正确
- [ ] 没有孤行、标题不跨页断行
- [ ] 图表没有超出版心

## 图表与编号的常见问题

| 现象 | 原因 | 处理 |
|---|---|---|
| 目录是空的 | 标题没样式也没 `outlineLevel` | 用 Heading 样式，或给自定义样式设 `outlineLevel` |
| 目录页码是旧的 | 目录是域 | `references.updateFields` |
| 图编号从 0 开始 | 插入顺序 | 题注在插入图表**之后**建 |
| 参考文献编号乱 | 引用顺序变了 | 重新 `references.citationStyle` + `references.bibliography` |
| 跨页表格断行难看 | — | 允许跨页但要设 `table.repeatHeader` 重复标题行 |

## 参考文献著录（GB/T 7714 速查）

| 类型 | 标识 | 格式 |
|---|---|---|
| 专著 | `[M]` | 主要责任者. 题名: 其他责任者. 版本项. 出版地: 出版者, 出版年: 引文页码〔文献类型标识〕. |
| 期刊文章 | `[J]` | 主要责任者. 题名〔文献类型标识〕. 刊名, 年, 卷(期): 页码. |
| 学位论文 | `[D]` | 主要责任者. 题名〔文献类型标识〕. 保存地: 保存者, 年份. |
| 会议论文 | `[C]` | 主要责任者. 题名〔文献类型标识〕//论文集主要责任者. 论文集名. 出版地: 出版者, 年: 页码. |
| 专利 | `[P]` | 专利申请者. 专利题名: 专利号〔文献类型标识〕. 专利公布日期. |
| 电子资源 | `[EB/OL]` | 主要责任者. 题名〔文献类型标识〕/主要责任者. 题名. (更新或修改日期)〔引用日期〕. 获取和访问路径. |
| 标准 | `[S]` | 主要责任者. 标准编号 标准名称〔文献类型标识〕. 出版地: 出版者, 出版年. |
| 报纸 | `[N]` | 主要责任者. 题名〔文献类型标识〕. 报纸名, 年-月-日(版次). |

著录用**顺序编码制**（正文 `[1]` 式标注）或**著者-出版年制**（正文 `作者(年)` 式）。

## 参考

- [references/gbt7713.md](references/gbt7713.md) —— 论文版式要素与数值
- [references/gbt7714.md](references/gbt7714.md) —— 参考文献著录规则速查
- 排版命令的完整说明见 wordcraft skill 的 [references/commands.md](../wordcraft/references/commands.md)

## 标准来源

- 《信息与文献 论文编写规则》GB/T 7713—2006
- 《信息与文献 参考文献著录规则》GB/T 7714—2015
- 《信息与文献 学术论文编写规则》GB/T 7713.2—2022（较新，部分高校已采用）
