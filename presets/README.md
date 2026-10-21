# 中文预设（presets）

中文文档的默认设置。**每一个预设都是一段 WordCraft 命令序列**，
用 `wordcraft-cli run --script` 应用，或作为 `styles.modify` 的参数。

这些预设对应**中文版专属**的部分——上游没有、也不该有（见
[`../OPEN-SOURCE-STRATEGY.md`](../OPEN-SOURCE-STRATEGY.md)）。

---

## `zh-general.json` — 通用中文正文

适合：报告、总结、说明书、方案。

```json
{
  "commands": [
    { "command": "layout.margins", "params": { "top": 70.9, "bottom": 70.9, "left": 85.0, "right": 56.7 } },
    { "command": "styles.modify", "params": { "style": "Normal",
      "chr": { "font": "Times New Roman", "fontEa": "宋体", "size": 12 },
      "para": { "lineSpacing": { "rule": "multiple", "value": 1.5 }, "indentFirstChars": 2 } } },
    { "command": "styles.modify", "params": { "style": "Title",
      "chr": { "fontEa": "黑体", "size": 18, "bold": false, "color": "000000" },
      "para": { "align": "center", "lineSpacing": { "rule": "multiple", "value": 1.5 }, "indentFirstChars": 0 } } },
    { "command": "styles.modify", "params": { "style": "Heading1",
      "chr": { "fontEa": "黑体", "size": 16, "bold": false, "color": "000000" },
      "para": { "lineSpacing": { "rule": "multiple", "value": 1.5 }, "indentFirstChars": 0 } } },
    { "command": "styles.modify", "params": { "style": "Heading2",
      "chr": { "fontEa": "黑体", "size": 14, "bold": false, "color": "000000" },
      "para": { "lineSpacing": { "rule": "multiple", "value": 1.5 }, "indentFirstChars": 2 } } },
    { "command": "styles.modify", "params": { "style": "Heading3",
      "chr": { "fontEa": "宋体", "size": 12, "bold": true, "color": "000000" },
      "para": { "lineSpacing": { "rule": "multiple", "value": 1.5 }, "indentFirstChars": 2 } } },
    { "command": "styles.modify", "params": { "style": "Caption",
      "chr": { "fontEa": "宋体", "size": 10.5, "bold": false, "color": "000000" },
      "para": { "align": "center", "lineSpacing": { "rule": "multiple", "value": 1.0 }, "indentFirstChars": 0 } } }
  ]
}
```

**纸张不用设** —— 中文系统按地区自动 A4（上游的 `region` 机制）。

---

## `zh-gongwen.json` — 公文

GB/T 9704—2012。完整说明见 [`../skills/gongwen/`](../skills/gongwen/)。

| 项 | 值 |
|---|---|
| 版心 | 156 × 225 mm（A4） |
| 边距 | 上 37 / 下 35 / 左 28 / 右 26 mm |
| 正文 | 仿宋_GB2312 3 号，固定行距 28 磅，首行缩进 2 字符 |
| 标题 | 方正小标宋简体 2 号居中（无此字体则黑体） |
| 一级「一、」 | 黑体 3 号顶格 |
| 二级「（一）」 | 楷体_GB2312 3 号顶格 |
| 颜色 | 全黑（公文不用彩色标题） |

套版式：`python skills/gongwen/scripts/format_gongwen.py content.docx -o gongwen.docx --pdf`

---

## `zh-thesis.json` — 论文

GB/T 7713 / 7714。完整说明见 [`../skills/thesis/`](../skills/thesis/)。

| 项 | 值 |
|---|---|
| 边距 | 上 25 / 下 25 / 左 30（装订）/ 右 20 mm |
| 正文 | 宋体小四，固定行距 20 磅，首行缩进 2 字符 |
| 题目 | 黑体三号居中 |
| 章标题 | 黑体三号居中；节标题黑体四号左空二字 |
| 参考文献 | 宋体五号，悬挂缩进（`indentLeftChars: 2` + `indentFirstChars: -2`） |
| 引用样式 | `GB/T 7714`，标题传 `"参考文献"` |

套版式：`python skills/thesis/scripts/format_thesis.py content.docx -o thesis.docx --pdf --citation-style "GB/T 7714"`

---

## `zh-legal.json` — 法务文书

见 [`../skills/legal-docs/`](../skills/legal-docs/)。

---

## 术语表（`terminology.json`）

同一名词全稿统一。内容层（写作 skill）负责**查找与替换**，本表只提供数据。

```json
{
  "terms": [
    { "preferred": "信息与网络安全", "variants": ["信息安全", "网络安全"], "note": "以正式文件名称为准" }
  ]
}
```

---

## 应用预设

```bash
wordcraft-cli run --file doc.docx --script presets/zh-general.json --save out.docx
```

⚠️ `save` 的相对路径**相对脚本所在目录**解析，所以从仓库根跑要写
`--save out.docx`（脚本在 `presets/`，输出会落到 `presets/out.docx`）——
用**绝对路径**最保险。

---

## 为什么预设是 JSON 而不是配置项

上游的偏好设置（`file.options`）只覆盖**纸张、单位、数字分隔符、拼写选项**，
**没有字体/字号/行距/缩进**。所以中文版要么：

1. **推上游**（推荐）：把"默认样式跟随地区/语言"做成 `file.options` 的一个新项
   —— 形状和已有的 `region` 完全一致，上游难拒
2. **留在本层**：用预设脚本在需要时套用（当前做法）

一旦上游接受第 1 条，这些预设就退化成"一张地区 → 默认样式的表"。
