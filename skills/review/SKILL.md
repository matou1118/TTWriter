---
name: review
description: 审阅与协作 WordCraft 文档——加批注、开修订跟踪、接受/拒绝修改、两版对比、限制编辑。Use when the user wants to review a document (批注/审阅/修订/留痕), compare two versions (对比/比较两个版本/找出改动), accept or reject changes (接受修订/拒绝修订), restrict editing (限制编辑/只读/保护文档), or handle a review round with multiple reviewers (会签/多人审阅). This is the REVIEW layer: it manages comments, tracked changes and document protection on text that already exists. For producing the document use the wordcraft skill; for legal or official-document layout use legal-docs or gongwen.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary on PATH. No network access needed.
metadata:
  author: 蜻蜓文档 TTWriter
  version: "1.0"
---

# 文档审阅协作

在**已存在**的文档上做审阅：批注、修订、对比、保护。四个能力互相独立，按需组合。

## 工作流

```
① 选模式 → ② 批注/修订 → ③ 处理对方改动 → ④ 定稿保护
```

| 场景 | 用什么 |
|---|---|
| 我要给对方提意见，不改原文 | 批注（comment） |
| 我直接改，但对方能看到改了哪里 | 修订跟踪（track changes） |
| 两份文件找出差异 | 对比（compare） |
| 定稿后不许再改 | 限制编辑（restrict） |

## ① 批注：提意见不改字

批注挂在**选区**上。先选文字再批注：

```json
{ "command": "select.text", "params": { "text": "本季度" } }
{ "command": "review.newComment", "params": { "text": "这里要写明是哪个季度" } }
```

- 批注里**只写意见**，不写替代文字——替代文字属于修订
- 回复别人的批注：`review.reply`（先 `review.nextComment` 定位到它）
- 已处理的批注 `review.resolveComment` 归档，不要删（留痕）
- 一轮审阅结束用 `review.comments` 列出全部，逐条核对无遗漏

## ② 修订：改了留痕

```json
{ "command": "review.trackChanges", "params": { "value": true } }
```

开了之后的一切编辑（插入、删除、改格式）都记为修订。**审阅者视角**：

```json
{ "command": "review.markup", "params": {} }
```

可选显示：`simple`（接受后的样子）、`all`（带修订标记）、`original`（改前样子）。

**写修订的人注意**：改完自己通读一遍 `review.changes`（审阅窗格），删掉手误——修订会被对方逐条看到，错字越多越失礼。

## ③ 处理别人的修订

```json
{ "command": "review.nextChange", "params": {} }     // 逐条走
{ "command": "review.accept", "params": {} }          // 接受当前
{ "command": "review.reject", "params": {} }          // 拒绝当前
{ "command": "review.acceptAll", "params": {} }       // 全部接受（确认后）
```

**逐条处理优先**。全部接受只在该作者之前已被信任时用。对格式修订（formatting 类）看清楚再定——格式修订在 markup 视图里不明显。

## ④ 对比：两版找差异

不改任何一份原件，产出一份带修订标记的第三份：

```json
{ "command": "review.compare", "params": {
    "path": "合同-v2.docx",
    "original": "合同-v1.docx",
    "author": "对比",
    "level": "word",
    "showIn": "new"
} }
```

- `level`: `word`（默认，快）或 `character`（细，慢）
- `showIn: "new"` 产出到新文档；`"original"`/`"revised"` 在原文上标记
- 对比结果的修订可以正常 accept/reject——**用法务审合同的流程**：v1 定稿 → 收到 v2 → compare 出差异 → 逐条 accept（同意）/reject（不同意）→ 存回 v3
- `caseChanges`/`whiteSpace`/`formatting` 控制是否把大小写/空白/格式差异算作改动，默认格式算、大小写空白不算（噪音太多）

## ⑤ 限制编辑：定稿保护

```json
{ "command": "review.restrict", "params": { "mode": "readOnly", "password": "…" } }
```

| mode | 效果 |
|---|---|
| `readOnly` | 只读，谁都不能改 |
| `comments` | 只能加批注 |
| `trackedChanges` | 能改，但全部强制留痕 |
| `forms` | 只能填表单域 |

解除：`review.restrict {"stop": true}`（带密码时须 `password`）。

## 命令速查

| 动作 | 命令 |
|---|---|
| 新批注 / 回复 / 解决 / 删除 | `review.newComment` / `reply` / `resolveComment` / `deleteComment` |
| 下一条批注 / 上一条 | `review.nextComment` / `previousComment` |
| 列出全部批注 | `review.comments` |
| 开关修订 / 显示模式 | `review.trackChanges` / `review.markup` |
| 逐条接受 / 拒绝 / 全部 | `review.accept` / `reject` / `acceptAll` / `rejectAll` |
| 对比 / 合并 | `review.compare` / `combine` |
| 限制编辑 | `review.restrict` |
| 审阅窗格 / 字数 | `review.changes` / `review.wordCount` |

## 完整示例：一轮合同审阅

```json
{
  "file": "contract-v2.docx",
  "commands": [
    { "command": "review.compare", "params": {
        "path": "contract-v2.docx", "original": "contract-v1.docx",
        "author": "律师甲", "level": "word", "showIn": "new" } },
    { "command": "select.text", "params": { "text": "违约金" } },
    { "command": "review.newComment", "params": { "text": "违约金上限建议改为合同总额的 20%" } },
    { "command": "review.trackChanges", "params": { "value": true } },
    { "command": "select.text", "params": { "text": "30 日" } },
    { "command": "text.insert", "params": { "text": "（自然日）" } },
    { "command": "review.trackChanges", "params": { "value": false } },
    { "command": "review.restrict", "params": { "mode": "comments", "password": "final-round" } }
  ],
  "save": ["contract-v3-tracked.docx"]
}
```

## 铁律

1. **批注提意见，修订动文字**——两者别混
2. **对比不改原件**——compare 永远产出第三份
3. **接受/拒绝前看 markup**——别在 simple 视图里处理修订
4. **密码丢了只能手工去除保护**（OOXML 层面），设密码前想清楚
5. **审阅完 `review.changes` + `review.comments` 双查**——确认没有漏网的修订和批注再定稿
