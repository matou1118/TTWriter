# PR #2 — 中文字数按字计

**类型**：修复（bug fix）
**影响面**：通用改进（中文/日文/韩文的字数统计都受益）
**补丁**：`target-gnu/pr-patches/0001-fix-doc-count-East-Asian-characters-as-words-G2.patch`

---

## 问题

`wordcraft_doc::count_words()` 按**空白**切词：

```rust
s.split(|c: char| c.is_whitespace() || c == para::OBJ)
    .filter(|w| w.chars().any(char::is_alphanumeric))
    .count()
```

对拉丁文没问题，但**中文/日文/韩文没有词间空格**，
所以一整段中文被算成 **1 个词**。

Word 的口径是**每个汉字算一个词**。

## 复现

```powershell
wordcraft-cli run --cmd "text.insert={""text"":""这是一段中文文本，用于验证字数统计功能是否正确。""}" --cmd review.wordCount
```

```
修复前：{"characters":24, ... "words":1}     ← 整段中文算 1 个词
修复后：{"characters":24, ... "words":22}    ← 24 字符中 2 个是标点，标点不计词
```

界面上就是"字数统计"对话框里的**字数**（Word 的中文界面同样按字计）。

## 影响

- **字数是中文排版最核心的指标之一**（投稿、合同、申报材料都按字数算），
  当前完全不可用
- 状态栏右下角的词数同样受影响
- 日文、韩文同理

## 修复

`crates/doc/src/lib.rs::count_words()` 改为：
**汉字/假名/谚文逐字计 + 拉丁文按词计**。

中英混排的句子两种口径都算对：

```rust
count_words("中文 mixed 文本")  // 2 + 1 + 2 = 5
```

## 为什么放在 `doc` crate

`doc` 是 L1 层，`layout`（L2）不能被它依赖。`layout/src/kinsoku.rs` 里已有
`is_east_asian()`，但它在下一层，所以 `doc` 保留自己的一份（`is_hanzi`），
注释里说明了原因。

## 测试

`crates/doc/src/tests.rs::word_count_counts_east_asian_characters_as_words`：
- 24 字中文句 → 22 词（标点不计）
- 中英混排 → 汉字数 + 拉丁词数
- 纯标点 → 0
- 日文假名 → 逐字计

全量 `cargo test --release --workspace` 通过。

## 备注

- 这**不是**要加中文分词。Word 对中文的字数统计也是按字，不分词。
  真正需要分词的是**中文拼写检查**（`w:proofErr`），那是另一件事。
- 界面上 `zh-hans` 词条已有"字数统计"这个标签，无需改翻译。
