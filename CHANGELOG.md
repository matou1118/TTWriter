# 更新日志

本文件记录**蜻蜓文档 TTWriter** 的变更。版本号 `0.9.0-zh.N` 表示「基于上游 WordCraft v0.9.0 的中文发行版第 N 版」。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)；基线版本号与上游保持一致，便于对齐。

---

## [0.9.0-zh.1] — 2026-10-21

第一个公开发行版。基线：上游 [WordCraft v0.9.0](https://github.com/storytold/wordcraft)。

### 新增：中文排版层

- **四个技能**（`skills/`），每个都配了可直接运行的脚本：
  - `skills/gongwen` — 公文排版，按《党政机关公文格式》GB/T 9704—2012（A4、版心、2 号小标宋标题、3 号仿宋正文、固定行距 28 磅）。
  - `skills/thesis` — 学位论文排版，按 GB/T 7713 与 GB/T 7714—2015（封面、页眉页脚、页码、目录、图表编号、参考文献著录）。
  - `skills/legal-docs` — 法务文书排版（民事起诉状、律师函、法律意见书、合同审查报告、劳动仲裁申请书），支持对外脱敏版与内部完整版两条输出。
  - `skills/wordcraft` — WordCraft 全部命令的用法说明、可用性核查方法与证据台账。
- **四套中文预设**（`presets/`）：`zh-general`、`zh-gongwen`、`zh-thesis`、`zh-legal`，一次应用页面与默认样式。
- **一份验证台账**（`skills/wordcraft/evidence.md`）：每条结论对应的命令、文件与渲染结果，可复核。

### 修复：中文相关

- **公文排版的字体名在现代 Windows 上不存在**。`仿宋_GB2312`、`楷体_GB2312` 并不是安装后的字体名（`simfang.ttf` 的字体名是 `FangSong` / `仿宋`），写了之后渲染器会**静默回退成黑体**，公文的仿宋 / 楷体区分整个丢掉。现在按候选顺序选择真机上真实存在的字体名，并会打印用了哪一个。
- **论文排版脚本的命令顺序错误**。插入页眉会把光标移进页眉文本区，此后只能在正文执行的命令（目录、参考文献）会以「目录要放在正文」失败。现在目录与著录排在页脚、页眉之前。

### 变更：相对上游

- **品牌**：移除全部 ArtCraft 标识与图案，换成自己的应用图标（朱砂圆角方块 + 文，脚本 [`tools/make_icons.py`](tools/make_icons.py) 一次生成 SVG / PNG / ICO / ICNS）。本仓库不代表 ArtCraft，只在文档中以平文本说明「基于 WordCraft（ArtCraft 团队）」。
- **显示名改为「蜻蜓文档 TTWriter」**：窗口标题、关于对话框、exe 的版本信息、开始菜单与安装包条目。关于对话框里的上游链接与「Part of the Crafting Apps from the ArtCraft team」一句已移除，换成本站、本仓库与上游仓库三个链接。
- **程序内部标识保持不变**：crate 名、二进制名 `wordcraft.exe`、配置目录、包名 `ai.storyteller.wordcraft` 都照旧，以便与上游同步时减少冲突。与上游的分叉只有二十几行显示文案。
- **许可**：上游代码仍为 MIT OR Apache-2.0；本仓库新增内容（技能、预设、工具、文档）为 Apache-2.0。

### 已知限制

- 没有中文拼写 / 语法校对。WordCraft 的校对只覆盖英、俄、波、葡、意几种语言。
- GB/T 7714 的条目模板缺少 `{issue}`（期号）槽位，期刊论文的期号需要手工补。
- 参考文献默认标题是英文 `References`，用 `references.bibliography` 的 `title` 参数改为「参考文献」。
- 方正小标宋简体不是 Windows 自带字体；机器上没有时标题回退为黑体并给出提示。
- 中文拼写、繁简转换、拼音排序、按地区自动选默认样式，都还没做。

### 已提交上游

顺手做的两处通用修复（不限于中文，任何 Word 用户都受益），已整理成补丁放在 [`upstream-prs/`](upstream-prs/)：

1. **中文字数按字统计**：原先把连续中文按空格切分计数，一段中文的字数严重偏少。现在按 Word 的中文字数口径统计。
2. **`w:lang/@w:eastAsia` 读写**：中文段落之前会丢掉东亚语言标记，在 Word 里打开后拼写检查语言会变成英语。

两个补丁在 v0.9.0 上 `git apply --check` 均干净。

### 主要变化

- 首个公开发行版。

---

## 致谢

- **ArtCraft 团队与 [WordCraft](https://github.com/storytold/wordcraft) 的全部贡献者** —— 本项目全部工作的地基。这个从零写起的 Rust 文字处理器把一个商业级字处理器该有的架构、命令系统、OOXML 兼容都做出来了，我们才能只在上面补一层。上游 600+ 条命令、完整的排版与渲染管线，是这个中文层能成立的前提。
- **WordCraft 的中文界面（`zh-hans`）来自上游**，简体中文界面不是本发行版做的。
- **[storytold/craftrules](https://github.com/storytold/craftrules)** —— 开发守则与工程标准（永不崩溃、素材来源、进度文档等），本仓库照办。
- 开放字体与词表的作者：Inter（Rasmus Andersson / The Inter Project Authors）、JetBrains Mono、Source Sans 与 Source Serif（Adobe，OFL）、Moby 断字词表（Grady Ward）、各语言 TeX 断字模式（Hanna Kołodziejska 等）。逐条列在 [`ATTRIBUTION.md`](ATTRIBUTION.md)。
- 国标《党政机关公文格式》GB/T 9704—2012、《学位论文编写规则》GB/T 7713、《信息与文献 参考文献著录规则》GB/T 7714—2015，本项目的排版规则以公开标准文本为依据自行实现，未复制任何商业软件的字模、模板或代码。

[0.9.0-zh.1]: https://github.com/matou1118/ttwriter/releases/tag/v0.9.0-zh.1
