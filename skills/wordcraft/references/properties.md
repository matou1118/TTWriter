# Properties: CharProps and ParaProps

`format.set`, `para.set`, `styles.modify`, `table.newStyle` and `table.modifyStyle` all take
these objects. **Every field is optional**: a field you leave out keeps what the style chain
gives. An unknown field name is **ignored, not refused** — so a typo silently does nothing.
Check the spelling.

Field names are **camelCase**. Lengths are **points** (1 pt = 1/72 inch). Colours are
`"RRGGBB"`, `"#RRGGBB"`, or `[r, g, b]` with three numbers 0–255.

---

## CharProps — character formatting

| Field | Type | Meaning |
|---|---|---|
| `style` | string | A character style id |
| `font` | string | Latin font family |
| `fontEa` | string | **East Asian font** (中文) — 宋体, 黑体, 楷体, 仿宋, 等线 |
| `fontCs` | string | Complex-script font (Arabic, Hebrew…) |
| `size` | number | Points |
| `sizeCs` | number | Complex-script size, points |
| `bold` / `italic` | bool | |
| `boldCs` / `italicCs` | bool | Complex-script bold / italic |
| `underline` | string | `single`, `double`, `thick`, `words`, `dotted`, `dash`, `wave`, `none` |
| `underlineColor` | string | `"RRGGBB"` |
| `strike` / `doubleStrike` | bool | |
| `caps` / `smallCaps` | bool | All caps / small caps |
| `hidden` | bool | Hidden text |
| `color` | string | `"RRGGBB"` |
| `highlight` | string | Highlight colour |
| `shading` | string | `"RRGGBB"` |
| `spacing` | number | Character spacing, points (can be negative) |
| `scale` | number | Character scaling, percent |
| `position` | number | Raised / lowered, points |
| `kern` | number | Kerning, points |
| `outline` / `shadow` / `emboss` / `engrave` | bool | Text effects |
| `vertAlign` | string | `superscript`, `subscript`, `baseline` |
| `textEffects` | object | WordArt-style effects |
| `lang` | string | BCP 47 tag for Latin text (`en-US`) |
| `langEa` | string | BCP 47 tag for East Asian text (`zh-CN`) |
| `langBidi` | string | BCP 47 tag for complex script |
| `noProof` | bool | Do not spell-check this run |
| `rtl` | bool | Right-to-left run |
| `cs` | bool | Format the run with the complex-script properties |
| `ligatures` | object | OpenType ligatures |
| `eastAsianLayout` | object | Two lines in one, horizontal in vertical |
| `link` | string | Hyperlink target |
| `ins` / `del` | number | Tracked insertion / deletion revision id |
| `fmtChange` | object | Formatting-change revision |

**`font` vs `fontEa`:** `font` sets the Latin face, `fontEa` the Chinese one. A Chinese
document usually wants both — a Latin face for the numbers and English, and 宋体/等线 for the
Chinese.

---

## ParaProps — paragraph formatting

| Field | Type | Meaning |
|---|---|---|
| `style` | string | A paragraph style id |
| `align` | string | `left`, `center`, `right`, `justify`, `distribute` |
| `indentLeft` / `indentRight` | number | Points |
| `indentFirst` | number | First-line indent, points (negative = hanging) |
| `indentLeftChars` / `indentRightChars` / `indentFirstChars` | number | **The same in characters** — the Chinese way |
| `spaceBefore` / `spaceAfter` | number | Points |
| `spaceBeforeAuto` / `spaceAfterAuto` | bool | Auto spacing (HTML-style) |
| `lineSpacing` | object | See below |
| `contextualSpacing` | bool | Skip space between same-style paragraphs |
| `keepNext` / `keepLines` | bool | Keep with next / keep lines together |
| `pageBreakBefore` | bool | |
| `widowControl` | bool | |
| `outlineLevel` | number | 1–9, for the table of contents |
| `numbering` | object | List reference |
| `tabs` | array | Tab stops |
| `shading` | string | `"RRGGBB"` |
| `borders` | object | Paragraph borders |
| `suppressHyphens` | bool | |
| `suppressLineNumbers` | bool | |
| `bidi` | bool | Right-to-left paragraph |
| `dropCap` | object | Drop cap |
| `frame` | object | Text frame (a paragraph in a frame) |
| `kinsoku` | bool | East Asian line-breaking rules (禁则) |
| `wordWrap` | bool | Allow a Latin word to break mid-word |
| `overflowPunct` | bool | Allow punctuation to hang past the margin |
| `topLinePunct` | bool | Compress punctuation at the line start |
| `autoSpaceDe` | bool | Auto space between Asian and Latin text |
| `autoSpaceDn` | bool | Auto space between Asian text and digits |
| `snapToGrid` | bool | Snap to the document grid |
| `mirrorIndents` | bool | |
| `numChange` / `fmtChange` | object | Tracked list / formatting revisions |

### lineSpacing

Tagged by `rule`:

```json
{"rule": "multiple", "value": 1.5}
{"rule": "atLeast", "value": 24}
{"rule": "exactly", "value": 28}
```

- `multiple` — 1.0 single, 1.15, 1.5, 2.0 double
- `atLeast` — at least this many points
- `exactly` — exactly this many points

**For Chinese documents**, `exactly` 28pt is the common 公文 setting; `multiple` 1.5 is the
general Chinese body-text setting.

### Indents in characters

Chinese typography indents by **characters**, not points: two characters is the standard
first-line indent. Use `indentFirstChars: 2`, not `indentFirst: 24`.

`indentFirstChars` and `indentFirst` can both be set; Word's rule is that a character-unit
value of 0 cancels the point value.

---

## Style sheet

`styles.modify {"style": "Heading1", "chr": {...}, "para": {...}}` changes a style. The `chr`
and `para` objects are the ones above.

Built-in style ids you will use: `Normal`, `Heading1`–`Heading9`, `Title`, `Subtitle`,
`Quote`, `IntenseQuote`, `ListParagraph`, `Caption`, `FootnoteText`, `EndnoteText`,
`TOCHeading`, `TableGrid`, `DefaultParagraphFont`, `NoSpacing`.

`styles.apply {"style": "Heading1"}` takes a **name or an id** — "Heading 1" and "Heading1"
both work.

A heading appears in a table of contents because of its style or its `outlineLevel`. A custom
style needs `outlineLevel` set explicitly.
