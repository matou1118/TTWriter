# 核验台账 — wordcraft skill

本文件记录 skill 里**每一条事实性断言**的核验方法与核验时的上游版本。
上游一周三版，**断言会过期**——过期时只改这一个文件，并在下表更新"核验于"列。

**核验基准**：上游 `storytold/wordcraft` tag **v0.9.0**
（commit `2e6db0bd88795e80375176c01665cff7dbe2b057`，2026-10-19）
**核验方式**：本机 `wordcraft-cli 0.9.0`（GNU 工具链构建）实跑。

---

## 一、命令面

| 断言 | 核验方法 | 层级 |
|---|---|---|
| 命令总数 633 | `wordcraft-cli commands \| wc -l` | 实测（**注意：上游各文档数字互相矛盾**——README 说 422、ROADMAP 说 433、mcp.md 说 "over 270"。**不要引用这些数字**，用 `commands` 现查） |
| 命令按命名空间分组 | `wordcraft-cli commands \| cut -d. -f1 \| sort \| uniq -c` | 实测 |
| `run --script FILE` 接受 JSON 脚本 | 实跑：`{"template":"blank","commands":[...],"save":["out.docx"]}` | 实测（v0.9.0 新增，v0.8.0 没有） |
| 脚本的 `file`/`save` 相对路径**相对脚本所在目录**解析 | 实跑：脚本放 `target-gnu/`，`save:["x.docx"]` → 输出到 `target-gnu/x.docx` | 实测（**易错点**） |
| `insert.table` 后光标落在**第一个单元格** | 实跑 + 渲染看图 | 实测 |
| 表格内用 `caret.right` 逐格移动 | 实跑 + 渲染看图 | 实测 |
| `caret.docEnd` 会跳到**文档末尾**（不是表格下一格） | 实跑：文本落到表外、单元格留空 | 实测（**最常见的错**） |
| `text.insert` 的 `\n` **不**产生新段落 | 实跑 | 实测 |
| 参数对象里**未知字段被忽略而非报错** | 实跑：`format.set {"props":{"bol":true}}` 成功但无效果 | 实测（**静默失败**） |
| `text` 命令的纯文本里**中文列表编号显示为 "1."**，渲染才是「一、」 | 实跑：`list.define {"format":"chineseCounting"}` → `text` 输出 "1."，`render` 输出「一、」 | 实测（**以渲染为准**） |
| `list.define` 的 `format` 接受 OOXML 名（`chineseCounting` 等），但命令文档只列了拉丁那几种 | `wordcraft-cli commands --json` 的 `list.define` 参数说明 vs `crates/doc/src/section.rs` 的 `NumFormat::ooxml()` | 实测（**传错名字静默退回 decimal**） |

## 二、属性对象

| 断言 | 核验方法 | 层级 |
|---|---|---|
| 字段名是 **camelCase** | `crates/doc/src/props.rs` 的 `#[serde(rename_all = "camelCase")]` | 源码 |
| 东亚字体字段是 `fontEa` | 同上，`pub font_ea: Option<String>` | 源码 |
| 东亚语言字段是 `langEa` | 本分支新增 `pub lang_ea`（commit `1fcd5d86`） | 源码 |
| 字符单位缩进字段是 `indentFirstChars` / `indentLeftChars` | `crates/doc/src/props.rs` | 源码 |
| `lineSpacing` 只有 `multiple` / `atLeast` / `exactly` **三种规则**（**没有 "auto"**） | `crates/doc/src/props.rs` 的 `enum LineSpacing` | 源码（**写 `"auto"` 会静默失效**） |
| 颜色形如 `"RRGGBB"` / `"#RRGGBB"` / `[r,g,b]` | `crates/doc/src/props.rs` 的 `Rgb::parse` | 源码 |

## 三、中文相关能力

| 断言 | 核验方法 | 层级 |
|---|---|---|
| `zh-hans` 词条 2203，与 `de`/`ja`/`zh-hant` **键集完全一致**；`ru` 多 367 条 | 逐键比对 `crates/ui-egui/src/i18n/*.tsv` | 实测 |
| **`zh-hans` 是上游的"参考键集基准"**：`de`/`pt-br`/`uk`/`sr`/`sr-latn` 键集必须与它**完全相等**，`et`/`es`/`tr`/`it` 必须是**超集** | `crates/ui-egui/src/i18n/tests.rs` 的 `is_subset` 断言 | 源码（**给 zh-hans 单独加词条会挂 5 条测试**） |
| `zh-hans` 缺 18 个内置样式名（`Normal`/`No Spacing`/`Subtitle`/…） | 与 `cs`/`ru` 逐键比对 | 实测（`cs`/`ru`/`ko`/`pl`/`vi` 等已有） |
| 缺 30 个 `tl!()` 字面量键（恢复/交叉引用对话框） | 扫 `ui-egui` 里 830 个 `tl!()` 键 vs 词条 | 实测 |
| **A4 对中文系统自动生效**（不需要改） | `crates/geom/src/region.rs` 的 LETTER 列表不含 CN；`file.options` 的 `region` 参数说明 | 源码 |
| 拉丁字体/东亚字体分开设 | `font` / `fontEa` | 源码 |
| 引用样式含 **GB/T 7714** | `wordcraft-cli run --cmd 'references.citationStyle={"style":"GB/T 7714"}'` → 输出 `[1] 张三. 共享空间研究[M]. 2021.` | 实测 |
| 参考文献标题恒为英文 "References" | `crates/engine/src/cmd/citations.rs` 的 `bib_title` 硬编码 | 源码（**`references.bibliography` 可传 `title` 覆盖**） |
| GB/T 7714 的条目模板**没有 `{issue}` 槽位**（期号丢失） | `citations.rs` 的 `article:` 模板串 | 源码（**实测输出缺 `(2)`**） |
| 中文字数按字计 | 本分支 commit `5aea619c` | 实测（上游 v0.9.0 仍按空白切词） |
| 东亚排版（禁则、中西文间距、文档网格、竖排、中文数字）已实现 | `crates/layout/src/kinsoku.rs`、`crates/doc/src/cjk_numbers.rs` | 源码 |
| 禁则/间距等**不需要你实现** | 渲染实测（"2026 年"之间自动有间距） | 实测 |

## 四、不能做的事（能力边界）

| 断言 | 核验方法 | 层级 |
|---|---|---|
| **没有中文拼写/语法检查** | `crates/proof/src/lang.rs` 的 `ProofLang` 无 `Zh`；`from_tag("zh-CN")` → `Other` | 源码（**不会误报，但也没有校对**） |
| 没有拼音排序 | `crates/doc/src/collate.rs` 只有拉丁字母表 | 源码 |
| 没有简繁转换 | 全仓库无相关实现 | 源码 |
| 图表/SmartArt 只读不创作 | `docs/gaps.md` | 上游自述 |
| 布局保真度约 Word 的 60% | `ROADMAP.md` | 上游自述 |

---

## 五、复核清单

上游发新版后，按这个顺序复核：

1. `wordcraft-cli commands --json` → 命令数、参数名有没有变
2. `crates/ui-egui/src/i18n/tests.rs` → 键集基准约束有没有变
3. `crates/doc/src/props.rs` → 属性字段有没有增删
4. `crates/engine/src/cmd/citations.rs` → 引用样式与模板有没有变
5. 跑一遍本 skill 的 `references/commands.md` 里的命令名核对
6. 更新本文件的"核验基准"行

**不要**因为上游变了就改 skill 正文里的**用法**——命令语义很少变，变的是名字和参数。
变了再改正文，并在这里记一笔。
