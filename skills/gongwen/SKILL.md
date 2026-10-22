---
name: gongwen
description: 按《党政机关公文格式》GB/T 9704—2012 排版中文公文（通知、请示、报告、函、纪要、批复、意见等），输出符合国标版式的 .docx 与 PDF。Use when the user wants a 公文/红头文件/通知/请示/报告/函/会议纪要/批复/意见 laid out to the Chinese national standard, mentions GB/T 9704、版心、公文版式、红头、版记、份号、密级, or asks for a 机关/国企 document to be formatted to standard. This skill is the LAYOUT layer only - it formats content that is already final and never writes, invents or edits the text. For drafting the content use a writing skill; for WordCraft's general command set use the wordcraft skill.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary on PATH. No network access needed.
metadata:
  author: 蜻蜓文档 TTWriter
  version: "1.0"
  standard: GB/T 9704-2012
---

# 公文排版（GB/T 9704—2012）

把**已定稿**的公文内容排成符合国标的 `.docx` / PDF。

## 铁律

1. **不改内容。** 这层只管字体、字号、段落、页码、标题层级、表格、版式。
   内容需要改 → 交回写作环节，不要在这里动。
2. **不补事实。** 缺文号、日期、主送单位就留 `[待补充]`，不要"合理推测"。
3. **不生成正文。** 没有正文就问，不要拿模板填空。
4. **排完必须看。** 渲染成 PNG 并**真的打开看**——国标问题（版心、页码位置、字号）
   在结构上看不出来。

## 工作流

```
① 确认内容已定稿  →  ② 设版心  →  ③ 套样式  →  ④ 加页码/版记  →  ⑤ 版面检查
```

### ① 确认内容已定稿

先问清楚（或从用户材料里读出）这几项，缺了就标 `[待补充]`：

| 项 | 说明 |
|---|---|
| 文种 | 通知 / 请示 / 报告 / 函 / 纪要 / 批复 / 意见 / 决定 / 公告 / 通报 / 命令 / 议案 |
| 发文机关标志（红头） | 机关全称 + "文件"；上行文（请示、报告）不冠 |
| 发文字号 | 如「XX〔2026〕15 号」；无则标 `[待补充]` |
| 密级和保密期限 | 公开 / 内部 / 秘密★1年 / 机密★5年 / 绝密★10年 |
| 紧急程度 | 特急 / 加急；一般不标 |
| 份号 | 份号（6 位 3 位阿拉伯数字），印发时使用 |
| 主送机关 | 请示、报告、函必须有 |
| 正文 | 已定稿 |
| 附件说明 | 「附件：1.XXX」 |
| 发文机关署名、成文日期、印章 | 成文日期用阿拉伯数字，印章覆盖成文日期 |
| 附注 | （承办单位、联系人、联系电话） |
| 版记 | 抄送、印发机关和印发日期 |

### ② 设版心

**A4（210×297mm）**，版心 **156×225mm**：

| 边 | 标准 | 毫米 | 磅 |
|---|---|---|---|
| 上白边 | 37mm±1 | 37 | 104.9 |
| 下白边 | 35mm±1 | 35 | 99.2 |
| 左白边（订口） | 28mm±1 | 28 | 79.4 |
| 右白边 | 26mm±1 | 26 | 73.7 |

用 `layout.margins` 设边距（磅），纸宽高由 A4 减去四边得出，
**不要**去改 `layout.size`（纸还是 A4，版心靠边距实现）。

```json
{ "command": "layout.margins", "params": { "top": 104.9, "bottom": 99.2, "left": 79.4, "right": 73.7 } }
```

### ③ 套样式

| 要素 | 字体 | 字号 | 行距 | 其他 |
|---|---|---|---|---|
| 正文 | 仿宋 | 3 号（16pt） | 固定值 28-30 磅 | 每自然段左空二字 |
| 标题 | 方正小标宋简体 | 2 号（22pt） | — | 居中，回行不断尾 |
| 一级标题「一、」 | 黑体 | 3 号（16pt） | 同正文 | 顶格 |
| 二级标题「（一）」 | 楷体 | 3 号（16pt） | 同正文 | 顶格 |
| 三级标题「1.」 | 仿宋 加粗 | 3 号（16pt） | 同正文 | 顶格 |
| 四级标题「（1）」 | 仿宋 | 3 号（16pt） | 同正文 | 顶格 |
| 主送机关 | 仿宋 | 3 号（16pt） | 同正文 | 顶格，标题下空一行 |
| 页码 | 宋体 | 4 号（14pt） | — | 单页右空一字，双页左空一字，用一字线「—」 |

**行距固定值 28 磅 → 每面 22 行**。这是国标里"每面 22 行每行 28 字"的来源：
版心高 225mm ÷ 22 行 ≈ 28 磅。

```json
{ "command": "para.set", "params": { "props": { "lineSpacing": { "rule": "exactly", "value": 28 } } } }
```

**正文首行缩进 2 字符**（`indentFirstChars: 2`，不是磅），**标题和层级标题顶格不缩进**。

改样式比逐段设置好：

```json
{ "command": "styles.modify", "params": { "style": "Normal",
  "chr": { "fontEa": "仿宋", "size": 16 },
  "para": { "lineSpacing": { "rule": "exactly", "value": 28 }, "indentFirstChars": 2 } } }
```

> ⚠️ **字体名要装得上**：`fontEa` 设的是字体名，优先用机器上真装的字体。
> 仿宋 / 黑体 / 楷体 Windows 一般自带（仿宋是随 Office 装的）；本发行版又内置了开源的
> **朱雀仿宋**与**霞鹜文楷 GB** 兜底，所以这三样缺了也照常排、不掉黑体。
> **方正小标宋简体不是**系统字体，需自行安装（不装则由程序兜底为宋体）。
> `font` 设拉丁字体、`fontEa` 设中文字体，两个都要设。

### ④ 页码、版记

**页码**：4 号半角宋体，单页码居右空一字，双页码居左空一字。

```json
{ "command": "insert.footer", "params": { "preset": "blankThree" } },
{ "command": "layout.differentFirstPage", "params": {} }
```

页码位置（单/双页分置）用页脚里的制表位或表格实现；简单场景用
`insert.pageNumber {"position": "bottom", "align": "right"}`。

> 单双页分置需要**奇偶页不同的页脚**。先确认 WordCraft 当前的奇偶页支持情况再承诺。

**版记**：置于公文最后一面，版记中的分隔线与版记中的文字等宽。
用表格或段落下框线实现，**不要**用图片。

### ⑤ 版面检查（必做）

先自动体检（快、免人眼），再人眼看 PNG（终审）：

```bash
python scripts/check_gongwen.py gongwen.docx        # 版心/行距/字号/字体/标题/页码 自动核对
```

```bash
wordcraft-cli render gongwen.docx page1.png --page 1
```

**打开 PNG 逐项核对**：

- [ ] 版心位置正确（左右上下留白）
- [ ] 正文 3 号仿宋、行距固定值 28 磅、每段左空二字
- [ ] 标题 2 号小标宋、居中、**没有回行断尾**（标题不该断到第二行）
- [ ] 层级标题「一、」「（一）」「1.」「（1）」层级清晰、顶格
- [ ] 页码位置和字号对
- [ ] 每页行数 ≈ 22 行、每行 ≈ 28 字
- [ ] 没有孤行（单字成行）
- [ ] 表格不超出版心、不跨页断行失控
- [ ] 附件说明、成文日期、印章位置对

行数/字数可以算：

```bash
wordcraft-cli info gongwen.docx
```

## 各文种的要素结构

不同文种要素不同，**不要套错**：

| 文种 | 必有要素 | 注意 |
|---|---|---|
| 通知 | 发文机关标志、发文字号、标题、主送机关、正文、发文机关署名、成文日期 | 向下级机关行文 |
| 请示 | **上行文**：不冠发文机关标志标志、不加签发人、一文一事 | 只主送一个上级机关 |
| 报告 | **上行文**：不冠标志、不加签发人 | 不得夹带请示事项 |
| 函 | 发文机关标志、标题、主送机关、正文、署名、日期 | 可平行/上下行文 |
| 纪要 | 标题、出席/列席/缺席、议定事项、发文机关署名、日期 | 议定事项用「一、」分层 |
| 批复 | 发文机关标志、发文字号、标题、主送机关、正文、署名、日期 | 只答一件请示 |
| 意见 | 发文机关标志、发文字号、标题、正文、署名、日期 | 分条列项 |
| 决定 | 发文机关标志、发文字号、标题、正文、署名、日期 | 事项用「一、」 |
| 通报 | 同通知 | 表彰/批评/情况通报 |
| 命令（令） | 发文机关标志、发文字号、标题、正文、署名、日期 | 限定受文范围 |
| 公告 | 发文机关标志、标题、正文、署名、日期 | 向国内外宣布重要事项 |

**上行文**（请示、报告）：不冠发文机关标志，版记不设签发人。

## 完整示例

```json
{
  "file": "content.docx",
  "commands": [
    { "command": "layout.margins", "params": { "top": 104.9, "bottom": 99.2, "left": 79.4, "right": 73.7 } },
    { "command": "styles.modify", "params": { "style": "Normal",
      "chr": { "font": "Times New Roman", "fontEa": "仿宋", "size": 16 },
      "para": { "lineSpacing": { "rule": "exactly", "value": 28 }, "indentFirstChars": 2 } } },
    { "command": "styles.modify", "params": { "style": "Title",
      "chr": { "fontEa": "黑体", "size": 22, "bold": false },
      "para": { "align": "center", "lineSpacing": { "rule": "exactly", "value": 28 }, "indentFirstChars": 0 } } },
    { "command": "styles.modify", "params": { "style": "Heading1",
      "chr": { "fontEa": "黑体", "size": 16 },
      "para": { "lineSpacing": { "rule": "exactly", "value": 28 }, "indentFirstChars": 0 } } },
    { "command": "styles.modify", "params": { "style": "Heading2",
      "chr": { "fontEa": "楷体", "size": 16 },
      "para": { "lineSpacing": { "rule": "exactly", "value": 28 }, "indentFirstChars": 0 } } }
  ],
  "save": ["gongwen.docx", "gongwen.pdf"]
}
```

## 常见问题

| 现象 | 原因 | 处理 |
|---|---|---|
| 标题断成两行 | 标题太长或字号过大 | 缩小字号到 2 号仍放不下时，按标准"回行时词意完整、排列对称"手工断行 |
| 每页不是 22 行 | 版心高或行距不对 | 行距必须是**固定值** 28 磅，不能用 1.5 倍 |
| 正文没缩进 2 字 | 用的是 `indentFirst`（磅） | 改用 `indentFirstChars`（字符） |
| 中文字体没生效 | 机器没装该字体 | `fontEa` 只对**已安装**的字体有效；查字体列表确认 |
| 改了样式但没变化 | 段落有直接格式 | `format.clear` 清掉直接格式 |
| 页码位置不对 | 单双页未分置 | 需要奇偶页不同页脚，先确认支持情况 |

## 参考

- [references/gbt9704.md](references/gbt9704.md) —— 国标要素与数值速查
- [references/wenzhong.md](references/wenzhong.md) —— 12 个文种的写法与格式要求
- 排版命令的完整说明见 wordcraft skill 的 [references/commands.md](../wordcraft/references/commands.md)

## 标准来源

《党政机关公文格式》GB/T 9704—2012（现行）。
标准全文可在国家标准化管理委员会公开系统查阅
（<https://openstd.samr.gov.cn/bzgk/gb/newGbInfo?hcno=F3CC9BEF482524C895FDA7A08BB4A70E>）。
**按标准实现是合法的；不要抄 WPS 或 Word 的模板文件**——那是别人的著作权内容。
