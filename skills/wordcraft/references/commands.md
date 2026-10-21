# Command reference

The commands used most. `wordcraft-cli commands` lists all 633 with their ribbon location;
`wordcraft-cli commands --json` is machine-readable. **Look up the exact parameter names there
before using a command** — a wrong field name is ignored silently, not refused.

Parameter objects (`CharProps`, `ParaProps`) use **camelCase**. Lengths are **points**. Colours
are `"RRGGBB"` or `"#RRGGBB"`, or `[r, g, b]`.

---

## Document lifecycle

| Command | Parameters | What it does |
|---|---|---|
| `file.new` | `{"template"?: "blank\|sample\|letter\|resume\|report"}` | New document. `blank` is the Normal template. |
| `file.open` | `{"path": string, "password"?: string}` | Open a file. |
| `file.save` | `{"path"?: string}` | Save. Without a path, saves over the current file. |
| `file.saveAs` | `{"path": string, ...}` | Save under a new name. |
| `file.exportPdf` | `{"path": string, ...}` | Export PDF. |
| `file.exportPng` | `{"path": string, ...}` | Export page images. |
| `file.print` | `{}` | Print. Goes through the platform. |
| `file.inspect` | `{"text"?: bool}` | Structure as JSON: blocks, runs, styles, sections, tables, selection, pages. |

## Text and the caret

| Command | Parameters | Notes |
|---|---|---|
| `text.insert` | `{"text": string}` | Inserts at the caret. **A `\n` does not start a paragraph.** |
| `text.newParagraph` | `{}` | Press Enter's worth of paragraph break. |
| `text.lineBreak` | `{}` | Shift+Enter line break (not a new paragraph). |
| `text.delete` | `{}` | Delete the selection. |
| `text.tab` / `text.backTab` | `{}` | Tab / Shift+Tab (demote a list level). |
| `caret.docStart` / `caret.docEnd` | `{}` | Jump to the start / end. |
| `caret.home` / `caret.end` | `{}` | Start / end of the line. |
| `caret.paraUp` / `caret.paraDown` | `{}` | Previous / next paragraph. |
| `caret.set` | `{"anchor": {...}, "focus": {...}}` | Put the caret somewhere specific. |
| `select.all` | `{}` | Select everything. |
| `select.text` | `{"text": string, "occurrence"?: n}` | Select the n-th occurrence. |
| `edit.find` | `{"text": string, ...}` | Find. |
| `edit.findNext` / `edit.findPrevious` | `{}` | Next / previous match. |
| `edit.replace` | `{"text": string, "with": string, ...}` | Replace one. |
| `edit.replaceAll` | `{"text": string, "with": string, ...}` | Replace all. `with` understands `^p` `^t` `^&` `^c` and `\1` with wildcards. |
| `edit.findReplace` | `{"tab"?: "find\|replace\|goto"}` | The find/replace dialog. |

## Styles and formatting

| Command | Parameters | Notes |
|---|---|---|
| `styles.apply` | `{"style"?: string}` | Apply a style by name or id to the selection. |
| `styles.create` | `{"name": string, ...}` | Create a style. |
| `styles.modify` | `{"style": string, "chr"?: CharProps, "para"?: ParaProps, "name"?: string, "next"?: string}` | Change a style's formatting. |
| `format.set` | `{"props": CharProps}` | Character formatting on the selection. |
| `para.set` | `{"props": ParaProps, "onPage"?: {...}}` | Paragraph formatting on the selection. |
| `para.indent` / `para.spacing` / `para.align` | see `commands --json` | Narrower paragraph commands. |
| `edit.formatPainter` | `{}` | Format Painter: copy formatting, then apply. |
| `edit.copyFormat` / `edit.pasteFormat` | `{}` | Copy / paste formatting only. |
| `format.clear` | `{}` | Clear character formatting. |
| `format.font` / `format.size` / `format.color` | `{"font"?: string, "size"?: pt, "color"?: "RRGGBB"}` | Narrower character commands. |

## Inserting

| Command | Parameters | Notes |
|---|---|---|
| `insert.table` | `{"rows": n, "cols": n, "style"?: string}` | Insert a table at the caret. **The caret lands in the first cell.** |
| `insert.picture` | `{"path": string, ...}` | Insert a picture. **The caret stays after it**, so typing next does not replace it. |
| `insert.pageNumber` | `{"position"?: "top\|bottom\|current", "align"?: "left\|center\|right", "format"?: "x of y"}` | Page number field. |
| `insert.pageBreak` | `{}` | Page break. |
| `insert.header` / `insert.footer` | `{"kind": "default\|first\|even"}` | Header/footer. |
| `insert.dropCap` | `{"lines"?: n, ...}` | Drop cap. |
| `insert.crossReference` | `{"target": string, ...}` | Cross-reference field. |
| `insert.equation` | `{"latex"?: string, ...}` | Equation. |
| `insert.bookmark` | `{"name": string}` | Bookmark. |
| `insert.link` | see `commands --json` | Hyperlink. (`insert.removeLink` removes it.) |
| `insert.smartArt` / `insert.chart` | see `commands --json` | **Read and kept, not authored** — do not build these. |

## Tables

| Command | Parameters | Notes |
|---|---|---|
| `table.insertRowAbove` / `table.insertRowBelow` | `{"count"?: n}` | Insert rows into the existing table. |
| `table.insertColumnLeft` / `table.insertColumnRight` | `{"count"?: n}` | Insert columns. |
| `table.deleteRow` / `table.deleteColumn` | `{"count"?: n}` | Delete rows/columns. |
| `table.deleteCells` | `{}` | Delete the selected cells. |
| `table.merge` / `table.split` | see `commands --json` | Merge / split cells. |
| `table.style` | `{"style": string}` | Apply a table style. |
| `table.newStyle` | `{"name": string, "basedOn"?: string, ...parts...}` | Create a table style. |
| `table.modifyStyle` | `{"style": string, "part"?: name, ...}` | Modify one. |
| `table.borders` / `table.shading` | see `commands --json` | Cell borders and fill. |
| `table.sort` | `{"column": n, "ascending"?: bool}` | Sort rows. |
| `table.formula` | `{"formula": string, ...}` | Table formula (=SUM(ABOVE)). |
| `table.repeatHeader` | `{"enabled": bool}` | Repeat the header row on each page. |

## References (目录、题注、引用)

| Command | Parameters | Notes |
|---|---|---|
| `references.toc` | `{"levels"?: 1-9, "title"?: string, "pageNumbers"?: bool, "rightAlign"?: bool, "tabLeader"?: "dot\|hyphen\|underscore\|none", "hyperlinks"?: bool, ...}` | Build or update the table of contents. `{}` updates an existing one. Headings need `outlineLevel` or a heading style to appear. |
| `references.caption` | `{"kind": "figure\|table", ...}` | Caption. |
| `references.citation` | `{"tag"?: string, "source"?: {...}, "pages"?: string}` | Insert a citation. |
| `references.citationStyle` | `{"style": "APA\|Chicago\|GB/T 7714\|GOST (name sort)\|Harvard (Anglia)\|IEEE\|ISO 690 (author-date)\|ISO 690 (numerical)\|MLA\|SIST02\|Turabian"}` | Set the bibliography style. Word's own names work too. |
| `references.bibliography` | `{"title"?: string}` | Build the bibliography. |
| `references.sources` | `{}` | Manage sources. |
| `references.index` | `{}` | Index. |
| `references.updateFields` | `{}` | Refresh fields (TOC, page numbers). |

## Review

| Command | Parameters | Notes |
|---|---|---|
| `review.wordCount` | `{"includeTextBoxes"?: bool}` | Pages, words, characters, paragraphs, lines. **Chinese counts per character.** |
| `review.spelling` | `{"text": string}` | Spelling issues. |
| `review.issues` | `{}` | All issues in the document. |
| `review.proofing` | `{}` | Proofing options. |
| `review.suggestions` | `{"text": string}` | Suggestions for a word. |
| `review.thesaurus` | `{"text": string}` | Thesaurus. |
| `review.language` | `{"value"?: string}` | Set the proofing language of the selection. |
| `review.compare` | `{"path": string, "original": string}` | Compare two documents. |
| `review.acceptAll` / `review.rejectAll` | `{}` | Accept or reject tracked changes. |
| `review.newComment` | `{"text": string, ...}` | Add a comment. |
| `review.trackChanges` | `{"enabled": bool}` | Turn track changes on/off. |

## Page and section layout

| Command | Parameters | Notes |
|---|---|---|
| `layout.size` | `{"size"?: "A4\|Letter\|Legal", ...}` | Paper size. **Leave it alone for a Chinese region** — WordCraft already follows the system locale. |
| `layout.margins` | `{"top"?: pt, "bottom"?: pt, "left"?: pt, "right"?: pt, ...}` | Margins. |
| `layout.orientation` | `{"landscape": bool}` | Orientation. |
| `layout.columns` | `{"count": n, "separator"?: bool}` | Columns. |
| `layout.break` | `{"kind": "page\|column\|section"}` | Section or column break. |
| `layout.lineNumbers` | `{"count"?: n, ...}` | Line numbers. |
| `layout.pageSetup` | `{"size"?: ..., "margins"?: ...}` | Everything at once. |
| `layout.hyphenation` | `{"auto": bool}` | Automatic hyphenation. |
| `layout.manualHyphenation` | see `commands --json` | Manual hyphen. |
| `layout.documentGrid` | `{"type"?: ..., ...}` | **Document grid** (文档网格) — the East Asian grid. |
| `layout.textDirection` | `{"direction": ...}` | Text direction. |
| `layout.verticalAlign` | `{"align": ...}` | Vertical alignment on the page. |
| `layout.differentFirstPage` / `layout.differentOddEven` | `{"enabled": bool}` | First page / odd-even headers. |

> **There is no `page.*` namespace** — page setup lives under `layout.*`.

## Filling a table

`insert.table` leaves the caret in the **first cell**. **`caret.right` moves to the next
cell** (and wraps to the next row), so the way to fill a table is to type and step:

```json
{ "command": "insert.table", "params": { "rows": 2, "cols": 3 } },
{ "command": "text.insert", "params": { "text": "A1" } },
{ "command": "caret.right" },
{ "command": "text.insert", "params": { "text": "B1" } },
{ "command": "caret.right" },
{ "command": "text.insert", "params": { "text": "C1" } },
{ "command": "caret.right" },
{ "command": "text.insert", "params": { "text": "A2" } }
```

> **⚠️ Do not use `caret.docEnd` to move between cells.** It jumps to the end of the
> *document*, which is *after* the table — the text lands outside the table and the cells
> stay empty. This is the single easiest way to get a wrong document.

There is no "go to cell N" command. `table.selectAt` takes page coordinates, which you would
have to compute — use `caret.right` instead.

---

## Verification and output

| Command | Notes |
|---|---|
| `document.inspect` | Structure as JSON. The first verification step. |
| `wordcraft-cli text <file>` | Plain text. |
| `wordcraft-cli render <file> out.png --page N` | Render a page to PNG. **Then look at the PNG.** |
| `wordcraft-cli info <file>` | Pages, words, properties. |
| `wordcraft-cli convert <in> <out>` | Between formats: docx, doc (read), pdf, odt, rtf, html, md, tex, txt, json, png, xml. |
