# Troubleshooting

Every item here was hit while building documents with WordCraft, not guessed.

---

## "the command did nothing"

**Most likely: a misspelled field name in a props object.** `format.set`, `para.set` and
`styles.modify` take `CharProps` / `ParaProps`, where **an unknown field is ignored, not
refused**. `{"props": {"bol": true}}` succeeds and does nothing.

Check the spelling against [properties.md](properties.md). Field names are camelCase:
`fontEa`, `indentFirstChars`, `langEa`, `doubleStrike`, `underlineColor` — not `font_ea`,
`indent_first_chars`.

**Also possible:** the command applies to the *selection*, and the selection is not what you
think. `styles.apply` on an empty selection does nothing.

---

## "the text went in the wrong place"

Commands act on the caret. After `text.insert` the caret is after the inserted text, so the
next command applies there. If the order is wrong, the formatting lands on the wrong text.

The fix is to be explicit: `caret.docEnd`, `caret.docStart`, `select.all`,
`select.text {"text": "...", "occurrence": 2}`, or `caret.set {"anchor": {...}, "focus": {...}}`.

**Inside a table**, `caret.docEnd` goes to the end of the *document* — after the table — so
text lands outside it and the cells stay empty. Use `caret.right` to step between cells.

---

## "everything is one paragraph"

`\n` inside `text.insert` is a literal newline character, **not** a paragraph break. Use:

```json
{ "command": "text.newParagraph" }
```

or the MCP tool `type_text` with `paragraphs: true`.

---

## "the output file is not where I expected it"

`save` paths in a script resolve **against the script file's folder**, not your working
directory. A script at `jobs/report.json` with `"save": ["report.docx"]` writes
`jobs/report.docx`.

Either write the path relative to the script, or put the script in the directory you want the
output in. `wordcraft-cli` says so in the error message ("folder … does not exist") — read it.

---

## "the JSON is mangled by the shell"

`--cmd 'id={"key":"value"}'` needs careful quoting, and Windows shells make it worse. **Use a
script file.** If you must use `--cmd`, single-quote the whole argument; on Windows, write the
script to a file and use `--script`.

---

## "my style change did not show up"

In order of likelihood:

1. The style was applied to a selection that was not what you meant.
2. The paragraph's own direct formatting overrides the style. Direct formatting wins over a
   style — clear it with `format.clear` and `para.set` on the properties you actually want.
3. The style change is on the style, but the paragraph uses a *different* style. Check with
   `document.inspect` — it reports each block's `style`.

---

## "the table is too wide / text overflows"

Table layout in WordCraft is close to Word but not identical.

- Set the table width explicitly rather than letting it autofit: `table.columnWidth`.
- Turn off autofit if it is fighting you: `table.autofit`.
- Distribute the columns: `table.distributeColumns`.
- Cell padding: `table.cellMargins`.
- Check the margins against the page: `layout.margins`.

**Then render the page and look at it.** This class of problem is invisible in the structure.

---

## "the table of contents is empty or wrong"

- Headings must have a heading style **or** an explicit `outlineLevel`. A custom style needs
  `outlineLevel` set.
- `references.toc` with `{}` updates an existing TOC; with options it replaces it.
- After editing, run `references.updateFields` (or `references.toc` again) — the TOC is a
  field, it does not update by itself.
- `references.toc {"levels": 3}` limits the depth.

---

## "the page numbers are wrong or missing"

Page numbers are fields. Refresh them with `references.updateFields` before saving, or save
with `--update-fields` (`convert` has the same flag).

---

## "the PDF has no text / looks wrong"

The PDF export embeds font subsets and keeps selectable text. If it looks wrong:

- The document's fonts may not be installed. WordCraft substitutes metric-compatible open fonts
  for missing ones (Calibri → Carlito, Times New Roman → Liberation Serif, Cambria → Caladea).
- East Asian fonts are **not** bundled — 宋体/微软雅黑 must be installed on the machine.
- Render the page to PNG and compare with the PDF.

---

## "the Chinese text has no spell-check marks"

Expected. WordCraft has no Chinese proofing: `ProofLang` has no Chinese variant, so Chinese
text is left alone (no false squiggles). Proofing works for English, Polish, Russian, Czech,
Turkish, Italian, Portuguese, Croatian, Bulgarian and Indonesian.

Do not tell the user the Chinese text was proofed.

---

## "the list number shows as 1. but should be 一、"

`wordcraft-cli text` extracts list labels with a simplified path, so Chinese numbering appears
as decimal in the text output. **The rendered page is correct.** Render and look.

If the rendered page really shows a decimal, the `format` string was not accepted —
`list.define` silently falls back to `decimal` for a name it does not know. The valid names
are the OOXML ones (`chineseCounting`, `chineseLegalSimplified`, `ideographDigital`, …) — see
[chinese-formats.md](chinese-formats.md).

---

## "the command errored but the document still changed"

`Session::run` turns a panic into an error and restores the document, so a failed command
should leave the document alone. If it did not, that is a bug worth reporting with the command
and its parameters.

---

## "the binary will not start on Windows"

The app needs DirectX 12 by default. If the window flashes and exits, the log says why.

```powershell
$env:WGPU_BACKEND = "vulkan"   # or "dx12", or "gl"
& .\wordcraft.exe
Remove-Item Env:WGPU_BACKEND
```

The log is at `%APPDATA%\WordCraft\logs\wordcraft.log` (previous runs roll to
`wordcraft.1.log`). **Document text is never logged.**

---

## "the script ran but nothing was saved"

- `save` must be present in the script object.
- The output path is relative to the script's folder (see above).
- `--print` prints the commands' results; without it the run is silent. Add `--print` while
  debugging.

---

## "it is slower than expected"

- The first run of a build is slow (compiling). Later runs are fast.
- `wordcraft-cli run` is headless and fast. The GUI app is for humans.
- For bulk work, one script with many commands beats many `run` invocations — each `run`
  starts a fresh session.

---

## Still stuck?

1. `wordcraft-cli commands --json` — the authoritative parameter list.
2. `wordcraft-cli inspect <file>` — what the document actually contains.
3. `wordcraft-cli render <file> out.png --page N` — what it actually looks like.
4. The logs, for anything that crashed.
