---
name: wordcraft
description: Create, read, edit, format, verify and export Word documents (.docx) with WordCraft, an open-source Rust word processor. Use this skill whenever the user wants a .docx/.dotx produced, modified, inspected or converted - a report, memo, letter, resume, contract, thesis, official document (公文), academic paper, or any document with headings, tables, page numbers, headers/footers or references. Triggers include "Word document", "word doc", ".docx", "make a report", "write a letter", "fill in a template", "convert to PDF", "check the document", and Chinese requests such as 写一个文档 / 生成公文 / 排版论文. Prefer this over unzipping OOXML by hand: WordCraft is a real word processor with 600+ typed commands, so a document is built by applying commands, not by editing XML.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary (or the WordCraft app for the screenshot tools). No network access needed; everything runs locally and offline.
metadata:
  author: WordCraft Chinese edition
  version: "1.0"
---

# WordCraft documents

WordCraft is a clean-room, open-source word processor written in Rust. It reads and writes
`.docx` (OOXML) and also `.odt`, `.rtf`, `.html`, `.md`, `.tex`, `.txt`; it exports PDF with real
selectable text. It has **600+ commands**, and every one of them is the same command the ribbon,
the keyboard and the MCP server use — so a document built by an agent is a document Word and
WPS open cleanly.

**The one rule that matters:** build documents by *applying commands*, never by unzipping the
`.docx` and editing `word/document.xml` by hand. WordCraft has a real layout engine, a style
sheet, a section model and a command registry; hand-editing XML skips all of it and produces
files that Word renders differently from what you meant.

## Which interface to use

Pick one. They run the same commands.

| Interface | Use when | Command |
|---|---|---|
| **CLI** | Default. Batch work, scripts, anything reproducible. | `wordcraft-cli run --script job.json` |
| **MCP** | You are an agent with an MCP client and want a live session. | `wordcraft-cli mcp` (headless) or `--connect HOST:PORT` (drive the running app) |
| **App + control channel** | You need screenshots of the real window. | `wordcraft --control 7981`, then JSON lines on `127.0.0.1:7981` |

See [references/interfaces.md](references/interfaces.md) for the MCP tool list and the control
protocol.

## The workflow

Always: **plan → build → verify → save.** Skipping verification is how documents ship broken.

### 1. Plan

Decide the document's *structure* before its content: title, heading levels, what goes in
tables, what needs page numbers. Write the plan as a list of styles to use, not as prose.

Look up the exact command and its parameters before using it — do not guess parameter names:

```bash
wordcraft-cli commands                      # every command with its location
wordcraft-cli commands --json | less        # machine-readable
```

`references/commands.md` has the ones used most.

### 2. Build

Prefer a **script file** over a long `--cmd` chain. It is JSON, it is readable, it is
re-runnable, and it is what you can show the user:

```json
{
  "template": "blank",
  "commands": [
    { "command": "text.insert", "params": { "text": "Quarterly Report" } },
    "caret.docEnd",
    { "command": "styles.apply", "params": { "style": "Heading1" } },
    { "command": "text.insert", "params": { "text": "Summary" } },
    "caret.docEnd",
    { "command": "insert.table", "params": { "rows": 3, "cols": 2 } }
  ],
  "save": ["report.docx", "report.pdf"]
}
```

```bash
wordcraft-cli run --script job.json
```

**Gotchas that cost real time:**

- **`save` paths resolve against the script file's folder**, not your working directory. A script
  at `jobs/report.json` saving `"report.docx"` writes `jobs/report.docx`. Use paths relative to
  the script, or put the script where you want the output.
- **A command is `{command, params}`** or just the id as a string (`"caret.docEnd"`).
- **`\n` inside `text.insert` does not create a paragraph.** Use `text.newParagraph` between
  paragraphs, or `type_text` with `paragraphs: true`.
- **`params` is JSON, not shell.** On Windows, quoting is painful — put the commands in a script
  file instead of fighting `cmd.exe` quoting.
- **Commands act on the selection.** After inserting, the caret moves; `styles.apply` applies to
  the selection. Sequence matters: insert text, then style it.
- **Only include fields you mean.** `format.set` / `para.set` / `styles.modify` take a props
  object where every field is optional; a field you leave out keeps what the style chain gives.
  An unknown field name is **ignored, not refused** — so a typo silently does nothing. Check
  spelling in [references/properties.md](references/properties.md).

### 3. Verify — do not skip this

Two checks, and they catch different failures:

```bash
wordcraft-cli inspect report.docx    # structure: blocks, runs, styles, sections, tables
wordcraft-cli text report.docx       # the plain text
wordcraft-cli render report.docx page1.png --page 1   # the rendered page
```

- `inspect` catches **structural** mistakes: wrong style, missing heading, text in the wrong
  paragraph, a table that landed in the wrong place.
- `render` catches **visual** mistakes: text overflowing its cell, a heading that broke across
  pages, a table wider than the page.

Then **read the PNG**. Rendering without looking at the result is not verification.

If you have the app running, `render_page` over MCP does the same thing.

### 4. Save and export

`save` in the script handles it. From the CLI, `wordcraft-cli convert in.docx out.pdf` converts
between formats.

For PDF: `save: ["report.docx", "report.pdf"]`. The PDF has selectable text, links and
bookmarks — it is a real export, not a screenshot.

## Editing an existing document

Same commands, one extra step: open it, and **do not assume the structure you remember**.

```bash
wordcraft-cli run --file input.docx --cmd "document.inspect" --print
```

1. `inspect` first. Find the text you need to change with `find`/`replace` or
   `select.text={"text":"..."}`.
2. Apply the change.
3. `inspect` again and compare. Editing must not disturb anything else.
4. Save to a **new** file unless the user asked to overwrite.

## Common tasks

| Task | Commands |
|---|---|
| New document | `file.new` with `template: blank\|sample\|letter\|resume\|report` |
| Type text | `text.insert` |
| New paragraph | `text.newParagraph` (or `text.insert` with `\n`? no — use this) |
| Apply a style | `styles.apply {"style":"Heading1"}` |
| Character formatting | `format.set {"props":{"bold":true,"size":14}}` |
| Paragraph formatting | `para.set {"props":{"align":"center","lineSpacing":{"rule":"multiple","value":1.5}}}` |
| Create a style | `styles.create` then `styles.modify` |
| Table | `insert.table {"rows":3,"cols":4}` |
| Find and replace | `edit.find` (find), `edit.replace` / `edit.replaceAll` (replace), `edit.findReplace` (the dialog) |
| Page numbers | `insert.pageNumber` (header/footer) |
| Table of contents | `references.toc` |
| Citation | `references.citation` + `references.citationStyle` |
| Export PDF | `save: [..., "out.pdf"]` |

## Chinese documents (中文文档)

WordCraft's Chinese support is real, and a few things differ from the Latin workflow:

- **Word count counts characters.** Chinese has no spaces, so `review.wordCount` reports
  中文字数 per character, matching Word's Chinese behaviour. A 24-character sentence is ~22
  words, not 1.
- **Set the East Asian font explicitly.** `font` sets the Latin face; `fontEa` sets the Chinese
  one. For 宋体/黑体/楷体/仿宋 use `fontEa`.
- **A4 is the default** for a Chinese region (it follows the system's locale, like Word).
- **First-line indent** is in characters, not points: `indentFirstChars: 2` gives the Chinese
  2-character indent. `indentFirst` is in points.
- **References use GB/T 7714**: `references.citationStyle {"style":"GB/T 7714"}`. The other
  built-in styles are APA, MLA, Chicago, IEEE and others.
- **Chinese number formats** (一、二、三 and 壹、贰、叁) are in the numbering formats.

For the specific conventions of 公文 (GB/T 9704) and 论文 (GB/T 7713), see
[references/chinese-formats.md](references/chinese-formats.md).

## What WordCraft cannot do

Know these before promising something:

- **No Chinese spell check yet.** `ProofLang` has no Chinese variant, so Chinese text gets no
  squiggles. Check Chinese text yourself; do not claim it was proofed.
- **No vertical text layout in the app**, no pinyin sorting, no simplified↔traditional
  conversion.
- **Charts and SmartArt** are read and kept, not authored.
- **OLE objects** (embedded spreadsheets) are dropped on save.
- **Print** works through the platform; there is no built-in print engine.
- Layout fidelity is ~60% of Word's — good enough for real work, not pixel-identical.

## Reference files

- [references/interfaces.md](references/interfaces.md) — CLI, MCP server, control channel, app
- [references/commands.md](references/commands.md) — the commands used most, with parameters
- [references/properties.md](references/properties.md) — CharProps and ParaProps fields
- [references/chinese-formats.md](references/chinese-formats.md) — 公文 / 论文 / 中文排版 conventions
- [references/troubleshooting.md](references/troubleshooting.md) — when something goes wrong
