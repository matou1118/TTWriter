---
name: mailmerge
description: 批量文书套打——用 Excel/CSV 名单 + 一份模板生成 N 份个性化文档（工资条、录取通知、证书、邀请函、批量公文、信封、标签）。Use when the user wants to 批量生成/套打/群发文书 from a 名单/花名册/数据表 (Excel 名单、CSV、通讯录), mentions 邮件合并/合并域/MERGEFIELD, or asks for one document per recipient (每人生成一份/按名单各出一版). This is the MERGE layer: it fills an existing template's placeholders from a table of recipients. For drafting the template use a writing skill; for WordCraft's general command set use the wordcraft skill.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary on PATH. No network access needed.
metadata:
  author: 蜻蜓文档 TTWriter
  version: "1.0"
---

# 批量文书套打（邮件合并）

一份模板 + 一张名单 = N 份文书。名单可以是 `.xlsx/.xlsm/.xls/.ods/.csv/.tsv/.txt`，或直接给行数据。

## 工作流

```
① 备模板 → ② 接名单 → ③ 插合并域 → ④ 预览核对 → ⑤ 合并输出
```

## ① 备模板

模板就是普通文档，**占位处留合并域**，不是留空或写「XXX」。先想清楚每份文书里什么是变的（姓名、金额、日期……），什么是死的。

模板排版（字体、页边距、落款位置）在这一步做完——**合并不改版式**，模板什么样,输出就什么样。

## ② 接名单

```json
{ "command": "mailings.recipients", "params": { "path": "名单.xlsx", "headers": true } }
```

- `headers: true`（默认）：第一行是字段名（姓名、部门、工资……）
- 多 sheet 工作簿会返回 `{"sheets": [...], "chooseSheet": true}`，再带 `"sheet": 1`（或名字）重发
- 编码可疑的 CSV：`"encoding": "gbk"`（或 `big5`、`shift_jis`……），默认自动探测
- 名单先人工抽查三五行：**错名单 = N 份错文书**

## ③ 插合并域

光标放到占位处，插域：

```json
{ "command": "mailings.insertField", "params": { "field": "姓名" } }
```

逐个占位处重复。常用域的替代方案：

- **地址块**：`mailings.addressBlock`（地址多字段拼一块）
- **称呼行**：`mailings.greetingLine`（「尊敬的 X 先生/女士」的自动称呼）

**域两旁的固定文字（如「先生/女士」）别做成域**——名单里性别字段解决不了的事，写在模板里。

## ④ 预览核对

```json
{ "command": "mailings.preview", "params": {} }        // 切换预览：域显示成第一条记录的值
{ "command": "mailings.findRecipient", "params": { "…": "张三" } }   // 跳到指定记录
{ "command": "mailings.checkErrors", "params": {} }     // 模拟合并，报错不产出
```

**预览时核对**：域位置对不对、有没有漏插的域、日期数字格式对不对。`checkErrors` 通过再合并——50 人的名单合并到一半报错最浪费。

筛选与排序（在合并前做，只影响本次输出）：

```json
{ "command": "mailings.filterRecipients", "params": {
    "rules": [ { "field": "部门", "op": "equal", "value": "销售部" } ],
    "sort": [ { "field": "工资", "descending": true } ] } }
```

ops: `equal/notEqual/less/greater/lessOrEqual/greaterOrEqual/blank/notBlank/contains/notContains`；多条规则用 `"join": "and|or"` 连接。

## ⑤ 合并输出

```json
{ "command": "mailings.finish", "params": { "path": "工资条-全部.docx" } }
{ "command": "mailings.finish", "params": { "path": "前10.docx", "from": 1, "to": 10 } }
```

- 输出是**一个多页文档**，每条记录一页（或一段，取决于模板分页）——要每人单独一个文件：合并后逐页拆，或按 `from/to` 分批合并保存
- 信封 / 标签：`mailings.envelopes` / `mailings.labels`（专门的版式向导）

## 命令速查

| 动作 | 命令 |
|---|---|
| 开始（选类型） | `mailings.start`（letters/emails/envelopes/labels/directory） |
| 接名单 / 编辑名单 | `mailings.recipients` / `editRecipients` |
| 筛选 / 排序 | `mailings.filterRecipients`（一次给 rules+sort） |
| 插域 / 地址块 / 称呼行 | `mailings.insertField` / `addressBlock` / `greetingLine` |
| 预览 / 找记录 / 查错 | `mailings.preview` / `findRecipient` / `checkErrors` |
| 下一条 / 上一条 | `mailings.next` / `previous` |
| 合并输出 | `mailings.finish`（`path` + 可选 `from`/`to`） |
| 高亮域 | `mailings.highlightFields`（核对时开） |

## 完整示例：销售部工资条

```json
{
  "file": "工资条模板.docx",
  "commands": [
    { "command": "mailings.recipients", "params": { "path": "名单.xlsx" } },
    { "command": "mailings.filterRecipients", "params": {
        "rules": [ { "field": "部门", "op": "equal", "value": "销售部" } ],
        "sort": [ { "field": "工号" } ] } },
    { "command": "caret.docStart", "params": {} },
    { "command": "mailings.insertField", "params": { "field": "姓名" } },
    { "command": "text.insert", "params": { "text": "　" } },
    { "command": "mailings.insertField", "params": { "field": "工号" } },
    { "command": "mailings.checkErrors", "params": {} },
    { "command": "mailings.finish", "params": { "path": "工资条-销售部.docx" } }
  ]
}
```

## 铁律

1. **名单先核对再合并**——错名单批量产出错文书，返工成本 = 人数
2. **`checkErrors` 通过才 `finish`**——别让 50 页合并到第 30 页才报错
3. **占位用合并域，不用「XXX」**——后者合并后原样留在文里
4. **模板版式先定型**——合并只填字不改版式
5. **敏感名单（工资、身份证）本地处理**——本流程全程离线，别把名单贴进任何在线工具
