# Agent Skills

Skills for driving WordCraft from an AI agent, and for the Chinese document
formats that WordCraft does not ship.

| Skill | What it is | License |
|---|---|---|
| [`wordcraft/`](wordcraft/SKILL.md) | The core skill: build, edit, format, verify and export `.docx` with WordCraft. Includes Chinese-document conventions. | Apache-2.0 |
| [`gongwen/`](gongwen/SKILL.md) | 公文排版 per GB/T 9704—2012: 版心, 仿宋/黑体/楷体 styles, fixed 28pt line spacing, page numbers, and a script that applies the whole layout to a finished document. `scripts/check_gongwen.py` audits the numbers automatically. | Apache-2.0 |
| [`thesis/`](thesis/SKILL.md) | 论文排版 per GB/T 7713 / 7714: 封面, 页眉, 目录, 图表编号, 参考文献悬挂缩进. `scripts/check_thesis.py` audits figure/table numbering and citation↔bibliography correspondence. | Apache-2.0 |
| [`legal-docs/`](legal-docs/SKILL.md) | 法务文书排版: 起诉状, 律师函, 法律意见书, 合同审查报告; 脱敏版标注. Bridges to WorkMind's legal content layer. | Apache-2.0 |
| [`review/`](review/SKILL.md) | 审阅协作: 批注, 修订跟踪, 接受/拒绝, 两版对比 (`review.compare`), 限制编辑 — the document lifecycle after it exists. | Apache-2.0 |
| [`mailmerge/`](mailmerge/SKILL.md) | 批量套打: Excel/CSV 名单 + 模板 → N 份文书 (工资条, 录取通知, 证书, 信封, 标签). | Apache-2.0 |
| [`exam/`](exam/SKILL.md) | 试卷排版: A3 双栏, 密封线, 答题留白, 数学公式 (`equation.set` LaTeX), 题号选项排列. | Apache-2.0 |
| [`accessibility/`](accessibility/SKILL.md) | 无障碍发布检查 per GB/T 37668 / WCAG: alt text, heading structure, table headers, language, contrast — every issue has a fix command. | Apache-2.0 |

## The architecture: content layer vs layout layer

The vertical skills split cleanly, and the split is what keeps this small:

```
内容层（别人已经做好了，不要重做）      排版层（这里做）
────────────────────────────────      ──────────────────
cn-gov-soe-writing-router  ──┐        wordcraft   核心命令与验证
  公文/汇报/纪要/研究/审核     ├──>      gongwen     公文版式 GB/T 9704
WorkMind 法务子系统          │        thesis      论文版式 GB/T 7713
  咨询/起诉状/合同审查/意见书   │        legal-docs  法务文书版式
                              │        presets     中文默认样式
```

- **内容层不碰排版，排版层不碰事实。**
- A layout skill only accepts content that is **already final**. It never writes,
  invents or edits text. Missing facts stay `[待补充]`.
- Each layer works on its own: you can use `gongwen` without any writing skill.

## What this is for

WordCraft already exposes 633 commands over the CLI, an MCP server and a JSON control channel.
A skill tells an agent **which sequence of commands to run** for a document task, and — more
importantly — **how to check its own work** before reporting success.

The core skill's rule: *plan → build → verify → save*. Verification is two checks, because they
catch different failures:

- `document.inspect` catches structural mistakes (wrong style, text in the wrong paragraph).
- `render` to PNG catches visual mistakes (text overflowing a cell, a heading broken across
  pages) — and you have to **look at the PNG**.

## Installing

Copy the skill directory into wherever your agent looks for skills:

```bash
# project-local (OpenCode, Claude Code, Codex, Cursor…)
cp -r wordcraft .opencode/skills/wordcraft
cp -r gongwen  .opencode/skills/gongwen
cp -r thesis   .opencode/skills/thesis
cp -r legal-docs .opencode/skills/legal-docs

# user-global
mkdir -p ~/.agents/skills && cp -r wordcraft gongwen thesis legal-docs ~/.agents/skills/
```

OpenCode also reads `~/.config/opencode/skill/`. Check your agent's documentation; the
discovery path differs between them.

## Using it

The skill needs the `wordcraft-cli` binary on `PATH` (or the WordCraft app for the
screenshot tools). Nothing else — it runs entirely locally.

```bash
wordcraft-cli --help
```

The vertical skills ship a script that applies the whole layout to a finished document:

```bash
python skills/gongwen/scripts/format_gongwen.py content.docx -o gongwen.docx --pdf
python skills/thesis/scripts/format_thesis.py content.docx -o thesis.docx --pdf --citation-style "GB/T 7714"
python skills/legal-docs/scripts/format_legal.py complaint.json -o out.docx --masked
```

## Verifying the skills

`wordcraft/evidence.md` is the **verification ledger**: every factual claim in the core skill,
how to re-check it, and the upstream commit it was checked against. Upstream releases three
times a week, so the claims go stale — that file is where you update them.

`skill-up` validates skills and runs evaluations against them:

```bash
skill-up validate ./wordcraft
```

Note: `skill-up` currently ships for macOS and Linux only.

The spec is at <https://agentskills.io/specification>. A skill is a directory with a
`SKILL.md` carrying YAML frontmatter (`name`, `description`, and optionally `license`,
`compatibility`, `metadata`, `allowed-tools`), plus optional `scripts/`, `references/` and
`assets/`.

## Adding a skill

Follow the layout of `wordcraft/`:

- `SKILL.md` — frontmatter plus the instructions, **under 500 lines**
- `references/` — detail loaded only when the task needs it
- `scripts/` — runnable helpers

Two rules learned the hard way:

1. **Verify every command name against the binary.** `wordcraft-cli commands` is
   authoritative. A wrong command name in a skill teaches the agent to fail.
2. **Verify every parameter name against `wordcraft-cli commands --json`.** A misspelled field
   in a props object is *ignored, not refused* — it fails silently, and the agent will report
   success on a document that was never changed.

## Related work

- [`../PLAN.md`](../PLAN.md) — 与上游/cikw 的对比、能力差距、技术方案、里程碑
- [`../ASSESSMENT.md`](../ASSESSMENT.md) — 优化与垂直拓展评估（P0–P3），含实测依据
- [`../SKILLS-PLAN.md`](../SKILLS-PLAN.md) — 垂直行业 skill 体系：GitHub 调研、什么复用、什么不做
- [`../OPEN-SOURCE-STRATEGY.md`](../OPEN-SOURCE-STRATEGY.md) — 与上游的关系：哪些推上游、哪些留本仓库、商标义务



