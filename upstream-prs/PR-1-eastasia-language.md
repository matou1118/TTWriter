# PR #1 — 保留 run 的东亚语言（`w:lang/@w:eastAsia`）

**类型**：修复（bug fix）
**影响面**：通用缺陷，不限于中文（日文/韩文同理）
**补丁**：`target-gnu/pr-patches/0002-fix-docx-keep-a-run-s-East-Asian-language-w-lang-w-e.patch`

---

## 问题

Word 给一个 run 写三个语言属性：

```xml
<w:rPr><w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/></w:rPr>
```

- `w:val` —— 拉丁文本（西文）的语言
- `w:eastAsia` —— 东亚文本（中文/日文/韩文）的语言
- `w:bidi` —— 复杂文种（阿拉伯/波斯/希伯来）的语言

WordCraft 的文档模型只有**两个**槽位：`CharProps.lang`（→ `w:val`）和
`CharProps.lang_bidi`（→ `w:bidi`）。`w:eastAsia` **没有对应字段**，
于是它在读取时被丢弃，在写回时消失。

## 复现

```powershell
# 1. 造一个最小 docx，run 上带 w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"
#    （脚本见 target-gnu/make-lang-ea-docx.py）
python target-gnu/make-lang-ea-docx.py

# 2. 读进来 —— eastAsia 消失
wordcraft-cli inspect target-gnu/lang-ea-test.docx
#   修复前：{"lang": "en-US", "langBidi": "ar-SA"}
#   修复后：{"lang": "en-US", "langBidi": "ar-SA", "langEa": "zh-CN"}

# 3. 往返 —— eastAsia 彻底丢失
wordcraft-cli convert target-gnu/lang-ea-test.docx out.docx
#   修复前 document.xml：<w:lang w:val="en-US" w:bidi="ar-SA"/>
#   修复后 document.xml：<w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/>
```

## 影响

这不是只影响 WordCraft 内部：**用 Word 打开 WordCraft 保存的中文文档时**，
中文段落因为丢了 `w:eastAsia`，Word 会：

- 用**拉丁文的规则**去做拼写/语法校对 → 中文段落上出现误报下划线
- 按**拉丁文字体**回退 → 中文字形不对
- 忽略 Word 的**中文版式选项**（中西文自动间距、标点压缩、避头尾等）

## 修复

照抄 `lang_bidi` 的现成模式，4 处：

| 文件 | 改动 |
|---|---|
| `crates/doc/src/props.rs` | 新增 `CharProps.lang_ea`，并加入 `overlay_fields!` / `drop_same_fields!` |
| `crates/docx/src/read/props.rs` | `"w:lang"` 分支读 `w:eastAsia` |
| `crates/docx/src/write/props.rs` | `w:lang` 写出三个属性 |
| `crates/docx/tests/east_asian_language.rs` | 新增往返测试 |

属性顺序按 ECMA-376 §17.3.2.17 写成 `w:val` / `w:eastAsia` / `w:bidi`，与 Word 一致。

## 测试

`crates/docx/tests/east_asian_language.rs`（4 个）：
- 读出 run 的三个语言属性
- 没有 `w:eastAsia` 时为 `None`
- **往返后 `w:eastAsia` 仍在**
- 三个属性一起写出

另：`crates/docx/tests/roundtrip.rs` 的 `every_char_prop_round_trips` 加了 `lang_ea`。

全量 `cargo test --release --workspace` 通过。

## 备注

- 与 issue #1034（"Saving a .docx silently drops settings and parts the model doesn't
  represent"）是同一类问题，但 #1034 没有单独提到 `w:eastAsia`。
- 本 PR 只补**模型缺口 + 往返**，不涉及校对逻辑。
  有了它，调用方才能知道一段文本是中文（这也是中文校对的前置条件）。
