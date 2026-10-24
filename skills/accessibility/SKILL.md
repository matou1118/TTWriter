---
name: accessibility
description: 无障碍发布检查——扫描文档的替代文字、语言、标题结构、表格表头、对比度问题并逐项修复，符合 GB/T 37668 与 WCAG 对文档的要求。Use when the user wants 无障碍检查/可访问性/a11y/accessibility check on a .docx, mentions 无障碍标识、适老化、信息无障碍、GB/T 37668、WCAG、screen reader/读屏, or is publishing 政府公开附件/面向公众的文档 that must pass an accessibility review. This skill CHECKS and FIXES accessibility issues on an existing document. For producing the document use the wordcraft skill.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary on PATH. No network access needed.
metadata:
  author: 蜻蜓文档 TTWriter
  version: "1.0"
  standard: GB/T 37668-2019, WCAG 2.1
---

# 无障碍发布检查

对**已存在**的文档跑无障碍体检，把发现的问题逐项修掉。面向：政府网站附件（GB/T 37668 是国标要求）、面向公众的表单与通知、适老化改造项目。

## 工作流

```
① 全文扫描 → ② 逐类修复 → ③ 复扫清零 → ④ 留档
```

## ① 全文扫描

```json
{ "command": "file.accessibility", "params": {} }
```

返回 issues 列表（每条：类别、位置、说明）。配套的读屏视角核对：

```json
{ "command": "document.inspect", "params": { "text": true } }
```

典型 issue 类别与修复方法（**每条 issue 都有对应命令**——这是本 skill 的核心价值：检查和修复用同一套命令体系）：

## ② 逐类修复

### 图片缺替代文字（最高频）

```json
{ "command": "picture.altText", "params": { "text": "2026 年第三季度销售额柱状图，总额 420 万元" } }
```

替代文字写法：
- **描述内容和用途**，不是描述视觉外观（「柱状图，销售额 420 万」对，「蓝色矩形图」错）
- **装饰性图片**标 `decorative: true`（读屏跳过，不报缺）
- 复杂图表（流程图、多数据系列图）：短 alt + 正文文字说明配合

### 表格缺表头

读屏靠表头单元格定位。修复：首行设为表头行（表格属性 `headerRow`）+ 表头单元格文字非空。

### 标题层级跳跃

Heading1 直接跳 Heading3 = 读屏用户迷路。修复：逐级排（1→2→3），用 `styles.apply` 重设段落样式。**先 `document.inspect` 看全文结构**再动手。

### 文档语言未设

读屏的发音引擎靠语言标记。中文文档：

```json
{ "command": "review.language", "params": { "lang": "zh-CN" } }
```

混排文档按段落设（英文引文段落设 en-US）。

### 对比度不足

浅灰字浅灰底过不了 WCAG AA（4.5:1）。修复：`format.color` 换深色（正文用默认黑；辅助文字至少 #595959）。

### 空链接 / 裸链接文字

「点这里」和裸 URL 都不行。修复：链接文字说明去处（「国家标准化管理委员会官网」）。

## ③ 复扫清零

修完再跑一次 `file.accessibility`——**直到 issues 为空**。半修的文档和无障碍标一样误导。

## ④ 留档

报告存档（发布方常要证明检查过）：

```bash
wordcraft-cli run --file 文档.docx --cmd 'file.accessibility' --save 无障碍-已修.docx
```

## 无障碍 vs 体检脚本

`file.accessibility` 查的是**文档语义层**（alt、结构、语言）；gongwen/thesis 的 `check_*.py` 查的是**版式数值层**（边距、行距、编号）。发布重要文档两层都跑。

## 铁律

1. **修完必须复扫**——issues 清零才算完
2. **替代文字写用途不写外观**——读屏用户要的是信息等价
3. **不要为了清零把图片全标 decorative**——那是把问题藏起来
4. **标题结构先看全文再改**——局部改跳级会引入新跳级
5. **发布侧要求（GB/T 37668）是硬指标**——政府网站附件过不了检查会被退回
