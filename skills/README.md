# Agent Skills

Skills for driving WordCraft from an AI agent, and for the Chinese document
formats that WordCraft does not ship.

| Skill | What it is | License |
|---|---|---|
| [`wordcraft/`](wordcraft/SKILL.md) | The core skill: build, edit, format, verify and export `.docx` with WordCraft. Includes Chinese-document conventions. | Apache-2.0 |
| [`gongwen/`](gongwen/SKILL.md) | 公文排版 per GB/T 9704—2012: 版心, 仿宋/黑体/楷体 styles, fixed 28pt line spacing, page numbers, and a script that applies the whole layout to a finished document. | Apache-2.0 |

## The architecture: content layer vs layout layer

The vertical skills split cleanly, and the split is what keeps this small:

```
内容层（别人已经做好了，不要重做）      排版层（这里做）
────────────────────────────────      ──────────────────
cn-gov-soe-writing-router  ──┐        wordcraft   核心命令与验证
  公文/汇报/纪要/研究/审核     ├──>      gongwen     公文版式 GB/T 9704
WorkMind 法务子系统          ─┘        legal-docs  法务文书版式
  咨询/起诉状/合同审查/意见书            thesis      论文版式 GB/T 7713
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

# user-global
mkdir -p ~/.agents/skills && cp -r wordcraft gongwen ~/.agents/skills/
```

OpenCode also reads `~/.config/opencode/skill/`. Check your agent's documentation; the
discovery path differs between them.

## Using it

The skill needs the `wordcraft-cli` binary on `PATH` (or the WordCraft app for the
screenshot tools). Nothing else — it runs entirely locally.

```bash
wordcraft-cli --help
```

`gongwen` also ships a script:

```bash
python skills/gongwen/scripts/format_gongwen.py content.docx -o gongwen.docx --pdf
```

## Validating

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

- [`../SKILLS-PLAN.md`](../SKILLS-PLAN.md) — the survey behind these skills: what already
  exists on GitHub (and what is licensed to reuse), what WorkMind provides, and what
  deliberately is not being built.

