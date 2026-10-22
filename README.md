<p align="center">
  <img src="assets/app-icon/hicolor/256x256/apps/ai.storyteller.wordcraft.png" width="112" height="112" alt="TTWriter">
</p>

<h1 align="center">蜻蜓文档 TTWriter</h1>

<p align="center">
  <b>A WordCraft distribution for writing in Chinese</b><br>
  National-standard layout for Chinese official documents, theses and legal papers, plus
  Chinese word counting and East Asian font / language marking.
</p>

<p align="center">
  <b>English</b> · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://www.ttread.cn"><img alt="Homepage www.ttread.cn" src="https://img.shields.io/badge/homepage-www.ttread.cn-b33420?style=flat-square"></a>
  <a href="https://github.com/storytold/wordcraft"><img alt="Based on WordCraft" src="https://img.shields.io/badge/based%20on-WordCraft%20v0.9.0-0d9488?style=flat-square"></a>
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-2b47b5?style=flat-square&logo=rust&logoColor=white">
  <img alt="Runs on macOS, Windows, Linux, BSD and the web" src="https://img.shields.io/badge/runs%20on-macOS%20%C2%B7%20Windows%20%C2%B7%20Linux%20%C2%B7%20BSD%20%C2%B7%20Web-3b5bdb?style=flat-square">
  <img alt="UI: Simplified Chinese" src="https://img.shields.io/badge/UI-%E7%AE%80%E4%BD%93%E4%B8%AD%E6%96%87-2b47b5?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-2b47b5?style=flat-square">
  <img alt="Agent-drivable over MCP" src="https://img.shields.io/badge/agents-MCP%20%C2%B7%20CLI-3b5bdb?style=flat-square">
</p>

![TTWriter with a Chinese official document open](docs/images/zh/hero.png)

---

## What this is

**蜻蜓文档 TTWriter** is a **Chinese edition of [WordCraft](https://github.com/storytold/wordcraft)**:
a complete source repository that builds, runs and can be changed, with the layer that writing in
Chinese actually needs added on top.

WordCraft is an open-source word processor written from scratch in Rust, aiming at Microsoft Word
parity while being faster, more open, and drivable by other programs. Upstream already ships a
Simplified Chinese interface (`zh-hans`), CJK glyphs and CJK line breaking. What we add is the
**writing-standard layer**: how a national-standard document is laid out, how characters are
counted, and how fonts are written into the file.

| | |
|---|---|
| **Full upstream source** | Not a plugin, not a skin. `crates/` *is* WordCraft. One clone, and it builds. |
| **Chinese word counting** | A Chinese "word count" counts characters, not space-separated tokens. Counts match Word's Chinese word count (G2). |
| **East Asian fonts and language** | `w:rFonts/@w:eastAsia` and `w:lang/@w:eastAsia` round-trip correctly, so a Chinese document keeps its fonts and its language marking when Word opens it. |
| **Four layout skills** | Chinese official documents (GB/T 9704), theses (GB/T 7713 / 7714), legal papers, and a general skill that documents every WordCraft command. |
| **Four Chinese presets** | General, official document, thesis, legal — applied in one step. |
| **Offline** | No network, no upload. Your writing stays on your machine. |

---

## Install and use

### Prebuilt binaries

Download the build for your platform from
[Releases](https://github.com/matou1118/ttwriter/releases) and run `wordcraft`.

### Build from source

```bash
git clone https://github.com/matou1118/ttwriter
cd ttwriter
cargo build --release -p wordcraft
```

Needs Rust 1.95+ (`rust-version` in `Cargo.toml`). The full recipe for Windows (GNU toolchain +
MinGW) is `build.ps1`; `test-all.ps1` runs build, tests, formatting, skill validation and an
end-to-end layout smoke test in one command.

**Chinese fonts are not committed** (font binaries never are). The official Chinese edition embeds
阿里巴巴普惠体 (Alibaba PuHuiTi) as an optional build input, so its binary still shows Chinese on
a machine with no Chinese fonts installed:

```text
craft-fonts/fonts/manifest.txt   # plus the .otf files; both gitignored
./build.ps1                      # picks them up automatically
```

A plain `cargo build` gives upstream behaviour: Chinese comes from the system (仿宋/宋体/楷体/黑体
on Windows, Noto CJK on Linux). PuHuiTi is a sans face used only as a fallback, so 公文 正文 still
wants 仿宋 from the system.

### Command line and automation

Everything is a command. The ribbon, keyboard shortcuts, the CLI, the control channel and MCP all
dispatch through the same command registry.

```bash
wordcraft-cli convert draft.docx draft.pdf          # convert formats
wordcraft-cli render draft.docx page1.png --page 1  # render to an image
wordcraft-cli pdf merge all.pdf in1.pdf in2.pdf     # merge PDFs; also info / rotate / delete / extract
wordcraft-cli commands                              # list every command
wordcraft-cli mcp                                   # run as an MCP server for AI agents
```

---

## The Chinese layout layer

### Skills

Under `skills/`, these are operating manuals for AI assistants: which command to run when, what the
standard requires, and where the traps are. Each ships runnable Python scripts.

| Skill | What it does | Standard |
|---|---|---|
| [`skills/wordcraft`](skills/wordcraft/) | How to use every WordCraft command, how to verify availability, plus an evidence ledger | — |
| [`skills/gongwen`](skills/gongwen/) | Chinese official documents: notices, requests, reports, letters, minutes, replies | GB/T 9704—2012 |
| [`skills/thesis`](skills/thesis/) | Theses: title page, running head, table of contents, figure/table numbering, references | GB/T 7713.1, GB/T 7714—2015 |
| [`skills/legal-docs`](skills/legal-docs/) | Civil complaints, lawyer's letters, legal opinions, contract review reports | — |

All four are **layout only**: the content must already be final, and a skill will never write,
invent or edit a word of it. Writing the content is a job for a writing skill.

### Presets

Four sets of Chinese page setup and default styles under `presets/`, each a runnable script:

```bash
wordcraft-cli run --file draft.docx --script presets/zh-gongwen.json --save out.docx
```

| Preset | For |
|---|---|
| `zh-general.json` | General Chinese documents |
| `zh-gongwen.json` | Official documents (type area, sizes, line spacing) |
| `zh-thesis.json` | Theses |
| `zh-legal.json` | Legal papers |

### Pages

| Official document | Thesis | Civil complaint |
|---|---|---|
| ![](docs/images/zh/page-gongwen.png) | ![](docs/images/zh/page-thesis.png) | ![](docs/images/zh/page-legal.png) |

<table>
<tr>
  <td width="50%"><img src="docs/images/zh/app-thesis.png" alt="Thesis layout"></td>
  <td width="50%"><img src="docs/images/zh/app-legal.png" alt="Civil complaint layout"></td>
</tr>
</table>

---

## What changed relative to upstream

The baseline is WordCraft **v0.9.0**. Changes fall into two groups, both described in
[`OPEN-SOURCE-STRATEGY.md`](OPEN-SOURCE-STRATEGY.md).

**Offered upstream** (prepared as PRs; patches in [`upstream-prs/`](upstream-prs/))

1. **Count Chinese characters as words** — a run of Chinese used to be split on spaces and counted,
   which badly under-counted. It now follows Word's Chinese word count (G2).
2. **Read and write `w:lang/@w:eastAsia`** — Chinese paragraphs used to lose their East Asian
   language marking, so Word would run its spell check in English.

**This edition only**

3. **Four layout skills and four presets** — see above.
4. **Fixed the font names used by the official-document layout** — `仿宋_GB2312` and `楷体_GB2312`
   are not installed family names on modern Windows (the family in `simfang.ttf` is `FangSong` /
   `仿宋`), so asking for them made the renderer **silently fall back to a sans face** and lose the
   FangSong/Kai distinction that a 公文 depends on. The names are now resolved in order against what
   is actually installed.
5. **Fixed the command order in the thesis script** — inserting a header moves the caret into the
   header story, after which main-text-only commands fail. The table of contents and the bibliography
   now come before the footer and header.

Known limits, documented in the skills rather than hidden:

- **No Chinese spelling or grammar proofing.** WordCraft proofs English, Russian, Polish, Portuguese
  and Italian only; this edition does not pretend otherwise.
- **The GB/T 7714 entry template has no `{issue}` slot**, so a journal issue number must be added by
  hand.
- **The bibliography heading defaults to the English `References`**; pass `title` to
  `references.bibliography` to get 参考文献.
- **方正小标宋简体** (the 2-point 小标宋 a 公文 title requires) is not bundled with Windows; without
  it the script falls back to a sans title and says so.

---

## Relationship to upstream

This is a **thin distribution**: divergence from upstream is kept small, and generic improvements go
upstream as PRs.

- The repository holds the **complete upstream source** plus our patches. Merging upstream is easy:
  there are only two patches, both inside small functions, and both `git apply --check` clean against
  v0.9.0.
- **Internal identifiers are unchanged** — crate and binary names, config directories, the bundle id
  `ai.storyteller.wordcraft`. Only the *displayed* name differs: the window title, the About box, the
  `.exe`'s properties, the installer entries — about twenty-five lines in all. That keeps syncing
  with upstream cheap; the generic patches are offered upstream as PRs.
- We **use no ArtCraft marks** and do not represent ArtCraft. Under upstream's trademark terms this
  repository says, in plain text, that it is based on WordCraft by the ArtCraft team. All brand art
  has been replaced with our own ([`ATTRIBUTION.md`](ATTRIBUTION.md)).
- Upstream code stays **MIT OR Apache-2.0**. What this repository adds (skills, presets, tooling,
  docs) is Apache-2.0.

Issues and PRs are welcome. Upstream has an excellent development guide in [`AGENTS.md`](AGENTS.md)
and this repository follows it; see [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow and
[`SECURITY.md`](SECURITY.md) to report a vulnerability.

---

## Homepage and provenance

- Homepage / portfolio: **[www.ttread.cn](https://www.ttread.cn)** (蜻蜓伴读 · 码头哥)
- Upstream: **[storytold/wordcraft](https://github.com/storytold/wordcraft)** — WordCraft is
  developed by the ArtCraft team, MIT OR Apache-2.0

## Credits

- **The ArtCraft team and every WordCraft contributor.** This Rust word processor written from
  scratch is the ground everything here stands on; upstream built the architecture and the OOXML
  compatibility solidly enough that we only had to add one layer.
- **The Simplified Chinese interface (`zh-hans`) comes from upstream**, not from this edition.
- The open fonts (Inter, JetBrains Mono, Source Sans/Serif), hyphenation and spelling lists we use
  are credited file by file, with licences, in [`ATTRIBUTION.md`](ATTRIBUTION.md).
- Upstream's public sources include ECMA-376 / ISO 29500 (OOXML).

Release-by-release notes and thanks are in [`CHANGELOG.md`](CHANGELOG.md).

## Licence

- Code in this repository: **MIT OR Apache-2.0** ([`LICENSE-MIT`](LICENSE-MIT),
  [`LICENSE-APACHE`](LICENSE-APACHE)), the same as upstream.
- What this repository adds (`skills/`, `presets/`, `tools/`, docs): **Apache-2.0**.
- Assets and third-party material: see [`ATTRIBUTION.md`](ATTRIBUTION.md).
