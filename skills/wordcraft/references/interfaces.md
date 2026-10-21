# Interfaces

Three ways to drive WordCraft. They all dispatch the same commands, so anything here works
everywhere.

---

## 1. CLI (the default)

```bash
wordcraft-cli --help
```

| Subcommand | What it does |
|---|---|
| `run` | Run commands, then save. `--file F` or `--template T`, `--cmd 'id={json}'` (repeatable), `--script FILE`, `--save OUT`, `--print` |
| `convert <in> <out>` | Between formats: docx, doc (read only), pdf, odt, rtf, html, md, tex, txt, json, png, xml |
| `inspect <file>` | Document structure as JSON |
| `text <file>` | Plain text |
| `info <file>` | Pages, words, paragraphs, properties |
| `render <file> <out.png>` | Render a page (`--page N`, `--scale S`) |
| `commands` | Every command (`--json`) |
| `parity` | Feature coverage against Microsoft Word |
| `mcp` | The MCP server (see below) |
| `zotero` | Zotero citations and bibliography |

### Script files

`run --script FILE` (or `-` for stdin) is the best way to do anything multi-step. The file is
JSON:

```json
{
  "file": "in.docx",
  "template": "sample",
  "commands": [
    { "command": "text.insert", "params": { "text": "Hi" } },
    "caret.docEnd"
  ],
  "save": ["out.docx", "out.pdf"]
}
```

- `file` and `template` are optional; `--file` / `--template` on the command line win.
- A command is `{"command": ..., "params": {...}}` or just the id as a string.
- **Relative `file` and `save` paths resolve against the script file's folder** (the working
  folder for stdin). This surprises people: a script in `jobs/` saving `"out.docx"` writes
  `jobs/out.docx`.
- Limits: 4 MiB script, 100 000 commands, 64 outputs.

### Quoting

`params` is JSON, which is painful to pass through a shell — especially on Windows. **Put the
commands in a script file.** If you must use `--cmd`, single-quote the whole
`id={"key":"value"}` argument.

---

## 2. MCP server

```bash
wordcraft-cli mcp                                  # headless, documents only
wordcraft-cli mcp --connect 127.0.0.1:7981         # drive the running app
```

| Tool | What it does |
|---|---|
| `list_commands` | Every command (filter with `query`) |
| `execute` | Run one command: `{command: "insert.table", params: {rows: 3, cols: 4}}` |
| `batch` | Run several commands in order |
| `new_document` | blank, sample, letter, resume, report |
| `open_document` / `save_document` | docx, doc (open only), odt, rtf, html, md, tex, txt, json; save also pdf, png |
| `type_text` | Type at the caret (`paragraphs: true` splits lines into paragraphs) |
| `select_text` | Select the n-th occurrence of some text |
| `get_text` / `inspect_document` | Read the document back — **verify without screenshots** |
| `render_page` | A page as PNG |
| `parity` | Feature coverage |
| `screenshot`, `click`, `key`, `ui_inspect` | The running app only (`--connect`) |

Resources: `wordcraft://document` (inspect) and `wordcraft://commands`.

### Confining the agent

`mcp --automation-read-root DIR --automation-write-root DIR` limits which files the agent can
read and write. A path outside the roots is an ordinary tool error the agent can read and
correct — not a crash. **Use this whenever the agent is untrusted or the task is narrow.**

`--author NAME` names who tracked changes and comments are.

---

## 3. The app and the control channel

```bash
wordcraft --control 7981
```

Then send JSON lines on `127.0.0.1:7981`. Each request carries the window's key, found in
`~/.config/wordcraft/control-key.local-7981` (macOS:
`~/Library/Application Support/WordCraft/`; Windows: `%APPDATA%\WordCraft\`):

```json
{"id":1,"key":"…","method":"engine.execute","params":{"command":"text.insert","params":{"text":"Hello"}}}
{"id":2,"key":"…","method":"ui.screenshot","params":{"path":"/tmp/shot.png"}}
```

Methods are listed in `docs/control-protocol.md` in the WordCraft source. `engine.execute` takes
the same command ids as everything else.

### Headless screenshots

To look at the real window without stealing focus (works on a locked screen):

```bash
cargo run --release -p wordcraft-ui-egui --example ui_shot -- script.jsonl
```

The script is JSON lines: control requests, `{"shot": "/abs/out.png"}` to save, `{"steps": n}`
to run extra frames, `{"wait": ms}`. The window is 1440×900 pt at 2× and opens the sample
document unless the first line is `{"empty": true}`.

---

## Which to use

| Situation | Use |
|---|---|
| A one-off document, reproducible | CLI + script file |
| You are an agent with an MCP client | MCP server |
| You need to see the real window | App + control channel, or `ui_shot` |
| Untrusted agent, narrow task | MCP with `--automation-read-root` / `--automation-write-root` |
| Batch conversion | `convert` |

## Logs

The desktop app writes to stderr and to `logs/wordcraft.log` next to its preferences
(Linux `$XDG_CONFIG_HOME/wordcraft/logs/`, macOS `~/Library/Application Support/WordCraft/logs/`,
Windows `%APPDATA%\WordCraft\logs\`). Each launch rolls the previous log to `wordcraft.1.log`.
`RUST_LOG=debug` (and similar) changes the level. **Document text is never logged.**
