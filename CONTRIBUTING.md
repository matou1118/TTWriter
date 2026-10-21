# 贡献指南

欢迎提 Issue 和 PR。这个仓库是 [WordCraft](https://github.com/storytold/wordcraft) 的中文发行版，所以有两条路：

- **通用改进**（不限于中文、任何 Word 用户都受益）→ 请优先提到**上游** `storytold/wordcraft`，我们也欢迎 PR 到这里，我们会帮你把它整理成上游补丁。
- **中文层改进**（技能、预设、国标排版、中文字体与语言处理）→ 提到这里。

先读 [`AGENTS.md`](AGENTS.md)：那是上游的开发守则，也是本仓库的守则。它比这份文档详细得多，尤其是「永不崩溃」和「素材绝不能来自 Microsoft / Adobe / Avid / Autodesk」两条。

## 开发环境

- Rust **1.95+**（见 `Cargo.toml` 的 `rust-version`）。
- Windows 上完整的 GNU 工具链配方：`./build.ps1`。
- 一条命令跑完构建、测试、格式检查、技能校验与端到端排版冒烟：`./test-all.ps1`。

```bash
cargo build --release -p wordcraft      # 应用
cargo run --release -p wordcraft -- --sample   # 跑起来看
cargo test --workspace --release        # 全量测试
```

## 合并前必须过

```bash
cargo xtask ci
```

它依次跑 `fmt`、`clippy -D warnings`、全量测试、素材台账（每个素材文件都要在 `ATTRIBUTION.md` 有行）、分层检查（L6 以下的 crate 不许依赖 UI）、wasm 构建。

跑不动的部分（比如本机没有 `trunk`，wasm 那步跳过）请**在 PR 里说明跳过了哪一步**，不要装作过了。

## 硬性规则

1. **不许崩。** 非测试代码里没有 `unwrap()` / `expect()` / `panic!` / `unreachable!` / `todo!` / `unimplemented!`，没有 `unsafe`。错误走 `Result`。输入（文件、命令参数、MCP 请求）一律当成敌意数据：用 `get()` 不用 `[i]`，字符串只在字符边界切，加法检查溢出，尺寸和递归都设上限。
2. **不许碰 Microsoft / Adobe / Avid / Autodesk 的素材。** 图标、模板、主题、字体、光标，一律不许复制、描摹或截屏裁切。每个素材文件都要在 [`ATTRIBUTION.md`](ATTRIBUTION.md) 有一行（作者、来源、许可），和文件在同一个 commit 里。
3. **改行为就改文档和测试。** 用户可见的行为必须是一条命令（`crates/engine/src/cmd/*.rs` 里的 `CommandSpec`），并且带测试。
4. **变得更好要能被测量。** 多语言界面改动要动 `crates/ui-egui/src/i18n/`，并且跑那 5 个守卫测试。

## 提交信息

一次能构建的工作一个 commit，前缀写清范围：

```
feat(skills): add the gongwen skill (GB/T 9704-2012 layout)
fix(gongwen): resolve real font family names on modern Windows
```

## 改中文排版层

- 技能放 `skills/<name>/`，必须有 `SKILL.md`（带合法 frontmatter，`tools/validate_skill.py` 会校验）和 `scripts/`。
- 技能**只排版**：内容必须已经定稿，技能不许替你写、编或改一个字。写内容的活属于写作技能。
- 引用 WordCraft 命令前，先对着 `wordcraft-cli commands --json` 核一遍命令名和参数名——写了不存在的命令会静默什么都不做。
- 结论要有证据。`skills/wordcraft/evidence.md` 就是台账：每条结论对应的命令、文件、渲染结果。新增结论请补台账。
- 国标规则以**公开标准文本**为依据自行实现，不复制商业软件的字模、模板或代码。

## 提交 PR

- 用 PR 模板填完。
- 说清楚**你实际跑了什么**、跳过了什么、以及已知没解决的地方。诚实比漂亮重要。
- 如果改动属于「通用改进」，在描述里说明我们可以把它整理成上游补丁。

## 许可

提交即表示你的贡献按 **MIT OR Apache-2.0**（代码）或 **Apache-2.0**（技能、预设、工具、文档）授权。
