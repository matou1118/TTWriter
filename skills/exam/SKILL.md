---
name: exam
description: 排版中文试卷——8 开双栏、密封线、答题卡留位、数学公式、题目编号——输出符合考场印制的 .docx 与 PDF。Use when the user wants a 试卷/考卷/试题册/模拟卷/练习卷 laid out for printing (双栏/密封线/装订线/8开/答题卡/题号/选择题选项排列), mentions 密封线、装订线内不准答题、A3 双栏试卷、答题卡, or asks for exam paper formatting with equations, choice options and blank answer space. This is the LAYOUT layer only - it formats exam content that is already final and never writes, invents or edits questions. For authoring the questions use a writing skill; for WordCraft's general command set use the wordcraft skill.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary on PATH. No network access needed.
metadata:
  author: 蜻蜓文档 TTWriter
  version: "1.0"
---

# 试卷排版

把**已定稿**的题目内容排成可进考场的试卷。中国考场标准形态：A3（8 开）双栏 + 密封线装订边 + 客观题涂卡区 + 主观题答题留白。

## 工作流

```
① 题目定稿 → ② 页面与分栏 → ③ 密封线区 → ④ 题号与选项 → ⑤ 公式 → ⑥ 印前检查
```

## ① 题目定稿

先确认（缺了标 `[待补充]`，不要自己编题）：

| 项 | 说明 |
|---|---|
| 卷头 | 学年学期、学校、科目、考试时长、满分 |
| 大题 | 一、选择题；二、填空题；三、解答题……各题数量与分值 |
| 题目 | 定稿文本（含公式） |
| 选项 | 每题 A/B/C/D 四项（或三项） |
| 答题留白 | 填空横线、解答题空白高度（按分值估算：每 5 分约 8-10 行） |
| 座位号栏 | 密封线内：姓名、班级、考号、座位号 |

## ② 页面与分栏

**A3 横向双栏**是考场标准（一张对折两张 A4）。屏幕编辑用 A4 双栏预览亦可。

```json
{ "command": "layout.size", "params": { "name": "A3" } }
{ "command": "layout.orientation", "params": { "value": "landscape" } }
{ "command": "layout.margins", "params": { "top": 28.4, "bottom": 28.4, "left": 85, "right": 42.5, "gutter": 28.4 } }
{ "command": "layout.columns", "params": { "count": 2, "space": 42.5 } }
```

| 项 | 取值 | 说明 |
|---|---|---|
| 纸 | A3 横向 420×297mm | 对折后两张 A4 |
| 边距 | 上下 10mm | |
| 左（装订）边 | 30mm | 密封线区 |
| 右边 | 15mm | |
| 栏间距 | 15mm | `gutter` 与 `space` 都给，装订边在栏间也留 |

**A4 版**（平时练习卷）：A4 纵向、单栏或双栏，边距正常，无密封线。

## ③ 密封线区

装订边（左边 30mm）放：竖排「密封线」三个字 + 姓名/班级/考号填写框 + 「密」「封」「线」间隔字样。

**实现**：装订边文字用页眉左侧竖排文本框，或正文首列前的无边框表格。**不要**用图片。竖排（section 级，密封线区常用 tbRl）：

```json
{ "command": "layout.textDirection", "params": { "value": "down" } }
```

（文本框内竖排用 `shape.textDirection {"dir": "vert"}`；表格单元格用 `table.textDirection`。）

卷头（第 1 栏顶部）：

```
━━━━━━━━━━━━━━━━━━━━
 2026—2027 学年第一学期期末考试
　　　数学试卷（满分 150 分，时长 120 分钟）
　　　姓名：______ 班级：______ 考号：______
━━━━━━━━━━━━━━━━━━━━
```

姓名等信息在**密封线内**（装订后看不见）——卷头再放一份是为了发卷核对，两处都要。

## ④ 题号与选项

**大题题头**（黑体、顶格）：

```
一、选择题（本大题共 8 小题，每小题 5 分，共 40 分）
```

**小题编号**（宋体，缩进一致）：

```json
{ "command": "para.set", "params": { "props": { "indentFirstChars": 2 } } }
```

**选择题选项排列**——短选项一行四项、长选项一行两项：

```
1. 下列函数中是增函数的为（　　）
   A. y=-x　　B. y=1/x　　C. y=x³　　D. y=√x
```

选项间用**全角空格**（`　`）分隔，不用制表位（跨栏更稳）。

**填空题**留横线：`答：______`（下划线用字符，不用下划线格式——留白更直观）。

**解答题**分值写在题尾（「（本题满分 12 分）」），题后按分值留白（插入空段落数 = 分值/5 × 8 行，向上取整）。

## ⑤ 公式

公式是试卷的重头。行内公式（Linear/LaTeX 输入）：

```json
{ "command": "equation.set", "params": { "latex": "y = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}" } }
```

- **行内**（嵌在题干里）：光标在插入点直接 `equation.set`
- **独立成行**（居中大公式）：先 `text.newParagraph`，段落居中，再插公式
- **公式编号**（解答题的化简过程）：`equation.number {"value": true}`
- 已有公式编辑：`equation.edit`
- 输入格式切换：`equation.inputFormat`（linear/latex）

**公式转纯文本校对**：`wordcraft-cli text 卷.docx` 导出纯文本，公式成线性串，通读查错别符号。

## ⑥ 印前检查（必做）

```bash
wordcraft-cli render 试卷.docx page1.png --page 1
wordcraft-cli render 试卷.docx page2.png --page 2
```

**打开 PNG 逐项核对**：

- [ ] 密封线在装订边内、竖排文字不断裂
- [ ] 双栏平衡：最后一页左右栏不悬殊（`text.columnBreak` 手动平衡）
- [ ] 大题题头没排在栏底（孤行）：`para.keepNext`
- [ ] 题号连续、分值加总 = 卷头满分
- [ ] 选项排列整齐、无跨行断组
- [ ] 公式字号与正文一致、没有显示成方框（缺字体）
- [ ] 解答题留白够写（估字数）
- [ ] 页码在页脚（第 1 页不显页码是惯例：`layout.differentFirstPage`）

```bash
wordcraft-cli info 试卷.docx     # 页数、字数——印量与纸张核对
```

## 各卷型的版式差异

| 卷型 | 纸/栏 | 特有要素 |
|---|---|---|
| 考试正卷 | A3 横双栏 | 密封线、考号栏、答题卡说明 |
| 平时练习 | A4 纵单栏 | 无密封线，题量小 |
| 答题卡专项 | A4 纵单栏 | 涂卡网格（表格画，不用图片） |
| 理科综合 | A3 横双栏 | 三科分区题头、共用卷头 |

## 完整示例

```json
{
  "file": "数学卷内容.docx",
  "commands": [
    { "command": "layout.size", "params": { "name": "A3" } },
    { "command": "layout.orientation", "params": { "value": "landscape" } },
    { "command": "layout.margins", "params": { "top": 28.4, "bottom": 28.4, "left": 85, "right": 42.5, "gutter": 28.4 } },
    { "command": "layout.columns", "params": { "count": 2, "space": 42.5 } },
    { "command": "styles.modify", "params": { "style": "Normal",
      "chr": { "fontEa": "宋体", "size": 10.5 },
      "para": { "indentFirstChars": 2, "lineSpacing": { "rule": "exactly", "value": 16 } } } },
    { "command": "styles.modify", "params": { "style": "Heading1",
      "chr": { "fontEa": "黑体", "size": 10.5 },
      "para": { "indentFirstChars": 0, "keepNext": true } } },
    { "command": "layout.differentFirstPage", "params": {} }
  ],
  "save": ["数学试卷.docx", "数学试卷.pdf"]
}
```

## 铁律

1. **不编题。** 题目内容定稿才排版；缺题目问，不拿模板填
2. **分值加总必须等于满分**——这是数学题也是纪律题
3. **密封线内不能有题目内容**——装订后看不见的字不要排
4. **公式排完必须看**——PNG 里公式断行、缺字体在结构上看不出来
5. **留白宁可多两行不可少一行**——写不下的考场事故比纸贵
