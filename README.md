# WordCraft 中文版

**基于 [WordCraft](https://github.com/storytold/wordcraft)（ArtCraft 团队）的中文增强层。**

> WordCraft 是一个用 Rust 从零写起的开源文字处理器（clean-room 重实现 Microsoft Word 的
> 功能，MIT OR Apache-2.0）。本仓库**不是** WordCraft 的替代品，也不包含 WordCraft 的源码——
> 它是叠在 WordCraft 之上的**中文层**：中文版式规范、中文校对，以及把"内容"变成"合规中文文档"
> 的那一段流水线。
>
> 上游源码的唯一真相源在 `storytold/wordcraft`。本仓库只在构建时按指定 tag 拉取并打包。

---

## 这个仓库做什么

```
内容层（别人的能力，本仓库不重做）        排版层（本仓库）
────────────────────────────────      ──────────────────
公文/汇报/纪要/研究        ──┐          wordcraft   核心命令与验证
法务：咨询/起诉状/合同审查   ──┴──>       gongwen     公文版式 GB/T 9704
                                          thesis      论文版式 GB/T 7713
                                          legal-docs  法务文书版式
                                          presets     中文默认样式与术语表
```

**分工原则**：内容层不碰排版，排版层不碰事实。
排版层的 skill 只接受**已定稿**的内容——不生成、不改写、不补全；缺的事实留 `[待补充]`。

---

## 目录

| 路径 | 内容 |
|---|---|
| [`skills/wordcraft/`](skills/wordcraft/SKILL.md) | 核心 skill：用 WordCraft 建/改/排/验/出文档。含中文排版约定与[核验台账](skills/wordcraft/evidence.md) |
| [`skills/gongwen/`](skills/gongwen/SKILL.md) | 公文排版，GB/T 9704—2012：版心、仿宋/黑体/楷体、固定行距 28 磅、页码；含一键套版式脚本 |
| [`skills/thesis/`](skills/thesis/SKILL.md) | 论文排版，GB/T 7713 / 7714：封面、页眉、目录、图表编号、参考文献悬挂缩进 |
| [`skills/legal-docs/`](skills/legal-docs/SKILL.md) | 法务文书版式：起诉状、律师函、法律意见书、合同审查报告；脱敏版标注 |
| [`presets/`](presets/) | 中文默认样式预设（通用 / 公文 / 论文 / 法务） |
| [`build/`](build/) | 拉上游 → 打补丁 → 编译 → 出包 |

## 文档

| 文档 | 内容 |
|---|---|
| [`PLAN.md`](PLAN.md) | 与上游/cikw 的对比、中文版的能力差距（G1–G6）、技术方案、里程碑 |
| [`ASSESSMENT.md`](ASSESSMENT.md) | 优化与垂直拓展评估（P0–P3），含实测依据 |
| [`SKILLS-PLAN.md`](SKILLS-PLAN.md) | 垂直行业 skill 体系：GitHub 调研、什么复用、什么不做 |
| [`OPEN-SOURCE-STRATEGY.md`](OPEN-SOURCE-STRATEGY.md) | 与上游的关系：哪些推上游、哪些留本仓库、商标义务 |
| [`README.upstream.md`](README.upstream.md) | 上游 WordCraft 的原始 README（未改动） |

---

## 上游贡献

本仓库的多数能力**推回上游**，不留在自己代码里：

| 改动 | 状态 |
|---|---|
| `w:lang/@w:eastAsia` 被丢弃（通用缺陷，上游 v0.9.0 仍未修） | 补丁已备好，待提 PR |
| 中文字数按字计（中日韩通用） | 补丁已备好，待提 PR |
| 简繁转换 | 计划中 |
| 拼音排序 | 计划中 |
| 默认样式跟随地区/语言 | 计划中（把上游已有的 `region` 设置从纸张扩展到字体/字号/行距） |
| `wordcraft` agent skill | 计划中，回应上游 issue [#1302](https://github.com/storytold/wordcraft/issues/1302) |

---

## 许可

本仓库自己的内容（skills、presets、build 脚本）以 **Apache-2.0** 授权。

上游 WordCraft 的源码与资产以 **MIT OR Apache-2.0** 授权，版权归 ArtCraft 团队与贡献者，
`NOTICE` 与 `LICENSE-MIT` / `LICENSE-APACHE` 照录。

**ArtCraft 的名称、字标和标识是商标**，不随本仓库授权，也**不在本仓库中分发**。
本仓库以平文本说明"基于 WordCraft（ArtCraft 团队）"。
本仓库不使用 WordCraft / ArtCraft 的图标——所有图标自绘。
详见上游 `docs/brand/LICENSE-brand.txt`。

新增的数据或资产请在 `ATTRIBUTION.md` 记录作者、来源和许可。

## 标准来源

- 《党政机关公文格式》GB/T 9704—2012（现行）
  <https://openstd.samr.gov.cn/bzgk/gb/newGbInfo?hcno=F3CC9BEF482524C895FDA7A08BB4A70E>
- 《信息与文献 参考文献著录规则》GB/T 7714—2015
- 《信息与文献 论文编写规则》GB/T 7713—2006

按标准实现是合法的；**不要抄 WPS 或 Word 的模板文件**——那是别人的著作权内容。
