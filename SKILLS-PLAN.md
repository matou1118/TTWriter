# 垂直行业 Skill 体系 — 调研与方案

> 调研于 2026-10-21。结论均有出处（GitHub API / 本地文件 / 实测）。

---

## 一、核心结论

**不要在 WordCraft 里重做各行业的"内容能力"，只做"排版交付层"。**

原因有两个，都很硬：

1. **内容层已经有人做好了，而且是高质量的。**
   - 公文/汇报/纪要/研究：`rien9036/cn-gov-soe-writing-router`（Apache-2.0）
   - 法务（咨询/起诉状/合同审查/意见书）：本机 `D:\App\WorkMind`（MIT，v7.4）
   - 通用 WordCraft 驱动：`midasmatterhorn/artcraft-skills`（MIT，13 个 skill，含 wordcraft）
2. **它们自己都留了"排版"的接口，但没人实现。**
   `cn-gov-soe-writing-router` 的路由矩阵里明确有一行：

   > | Word 排版 | 内容锁定 → 格式处理 → 版面检查 | 模板套用 | 排版时改事实 |

   它的"排版"职责定义得很清楚：**字体、字号、段落、页码、标题层级、表格、Word 版式；
   不负责实质内容修改**。但这个仓库里**没有排版 skill**，
   检查清单里只有一句"Word 表格、分页、页码无明显错误"。

**WordCraft 中文版正好就是这一层。** 定位：**中文文档的排版引擎 + 各行业版式**。

---

## 二、GitHub 调研结果（按可复用性排序）

| 仓库 | star | 许可 | 内容 | 结论 |
|---|---|---|---|---|
| `rien9036/cn-gov-soe-writing-router` | 0 | **Apache-2.0** | 中国机关/国企公文、汇报、纪要、研究、审核、**排版交付** | ✅ **直接复用**（内容层） |
| `midasmatterhorn/artcraft-skills` | 30 | **MIT** | ArtCraft 全家桶 13 个 skill，含 `wordcraft`；`evidence.md` 逐条标注证据 | ✅ **方法论可借鉴**（见下） |
| `D:\App\WorkMind`（本机） | — | MIT | 法务子系统 v7.4：72 部法条离线库、咨询/起诉状/合同审查/意见书、脱敏、时效计算 | ✅ **直接复用**（内容层） |
| `zubair-trabzada/ai-legal-claude` | 1792 | **无 LICENSE** | 14 个法务 skill | ❌ 无许可＝保留所有权利，不可引入 |
| `XimilalaXiang/Law` | 6 | **无 LICENSE** | 中文法务 skill | ❌ 同上 |
| `anthropics/skills/skills/docx` | 182k | **Proprietary** | docx/pptx/xlsx/pdf 文档 skill | ❌ 不可 vendor，且教的是 unzip 改 XML（与 WordCraft 相反） |
| `anthropics/skills/skills/skill-creator` | — | Apache-2.0 | 写 skill + 评测的方法论 | ✅ 可选 |
| `ShaishavMaisuria/research-paper-lifecycle-skills` | 55 | 未查 | 42 个学术写作 skill | 🟡 论文方向可参考（优先找 Apache/MIT 的） |
| `FOURTEEN1416/academic-agent-toolkit` | 11 | 未查 | 250+ 学术 skill | 🟡 同上 |

**GitHub 代码搜索需要登录**，所以用仓库搜索 + 直接读文件的方式逐个核实了许可。

### 2.1 `artcraft-skills` 的两个可借鉴点

**(a) `evidence.md` 模式** —— 把 skill 里的事实性断言和证据分离：

| # | Claim in SKILL.md | Evidence (repo-relative) | Tier |
|---|---|---|---|
| 1 | clean-room 重实现；MIT OR Apache-2.0 | README.md:13; Cargo.toml:9 | implemented |
| 3 | 命令数各文档不一致（422/433/270+） | README.md:98; ROADMAP.md:26 | claimed |

> Tier 区分 `implemented` / `claimed` / `gap`。**上游代码一周三版，skill 里的事实会过期——
> 把证据单独放一个文件，标上"核对时的 commit"，过期时只改一个文件。**

**(b) 定位不同，不是重复**：
- 他们的 wordcraft skill 回答"**WordCraft 能做什么、还不能做什么**"（能力说明书 + 证据）
- 我写的 wordcraft skill 回答"**怎么把一个文档任务做对**"（工作流 + 验证）

**结论：两个都保留，各司其职。** 我们的版本加中文，能力说明部分引用他们的。

---

## 三、WorkMind 法务：可以用，但要把渲染层换掉

### 3.1 WorkMind 有什么（本机实测）

| 模块 | 内容 |
|---|---|
| 离线法条库 | **72 部**现行有效法规（国家法 58 + 地方法规），JSON，带版本与废止标记 |
| legal-assistant Agent | 所有法条的唯一出口，废止过滤、涉密拦截、上位法优先 |
| 模块 A 咨询 | 五步流程 + 时效自动计算 + Logic Doctor 五维自检 |
| 模块 B 起诉状 | 11 类案由 + 通用模板 → docx |
| 模块 C 意见书 | 16 领域标准问卷 + 案例比对 + 七步工作流 → docx |
| contract-review | 8 模块 + 智能路由 + 属地检索 |
| 脱敏 | 身份证/手机/地址，对外脱敏版 vs 内部完整版 |

**质量很高，是资产。**

### 3.2 但它的渲染层是弱环

```
generate_complaint.py:  from docx import Document          ← python-docx
generate_opinion.py:    from docx import Document          ← python-docx
```

`python-docx` 产出的是**没有排版引擎的 docx**：无样式继承、无目录域、无真实页码、
无东亚排版（禁则/中西文间距）、页边距是 Word 默认值。

对**起诉状/意见书**这种要正式提交、对方会打印归档的文件，这是硬伤。

### 3.3 建议的接法（不复制，只换渲染层）

```
WorkMind（内容层：法条检索 / 定性 / 风险清单 / 结构）
        │  内容锁定
        ▼
WordCraft 中文版（排版层：版式 / 样式 / 域 / 页码 / 东亚排版 / PDF）
        │
        ▼
合规的 .docx + PDF
```

**具体做法**（不动 WorkMind 的法条库和业务流程）：
1. 在 WorkMind 侧保留 `kb_query.py` / `legal-assistant` / 问卷 / 风险规则——**一行不改**
2. 把 `generate_complaint.py` / `generate_opinion.py` 的**输出环节**换成调用 WordCraft
   （本文档的 `skills/gongwen/scripts/` 给出了同构的做法）
3. 版式规则由本仓库的 skill 提供（诉状/意见书版式 ≈ 公文版式 + 签名栏）

**注意**：WorkMind 已有自己的 `knowledge/`（72 部法规 JSON）。**不要复制进本仓库**——
它是独立项目的资产，且体积/授权需要单独确认。本仓库只做排版层。

---

## 四、要做的 skill（按优先级）

### P0 — `gongwen`：公文排版（中文版独家）

**定位**：内容锁定后的**排版交付**层。直接补上 `cn-gov-soe-writing-router` 留的接口。

| 内容 | 依据 |
|---|---|
| 版心 156×225mm，上白边 37mm、下白边 35mm、左白边 28mm、右白边 26mm | GB/T 9704—2012 |
| 正文 3 号仿宋_GB2312，行距固定值 28-30 磅 | GB/T 9704—2012 |
| 标题 2 号小标宋；一级标题 3 号黑体；二级 3 号楷体 | GB/T 9704—2012 |
| 页码 4 号半角宋体，单页码居右空一字，双页码居左空一字 | GB/T 9704—2012 |
| 每面 22 行、每行 28 字 | GB/T 9704—2012 |
| 版记、密级、紧急程度、份号、签发人、印发机关和印发日期 | GB/T 9704—2012 |
| 12 个法定文种的要素结构 | GB/T 9704—2012 |

**交付物**：`skills/gongwen/` + 一个把任意内容 docx 套上公文版式的脚本。

### P1 — `legal-docs`：法务文书排版（对接 WorkMind）

起诉状、律师函、法律意见书、合同审查报告的版式 + 脱敏标记 + 签署区。
**内容来自 WorkMind，本仓库只提供版式。**

### P2 — `thesis`：论文排版（GB/T 7713 / 7714）

上游已有 GB/T 7714 引用（实测可用），缺的是版式：封面、页眉、目录、图表编号、
参考文献格式、页码。**这个是纯增量，且不与 WorkMind 重叠。**

### P3 — 医疗 / 科研院所

⚠️ **先别做**。理由：
- 医疗文书（病历、知情同意书、诊断证明）有**行业规范和法律责任**，写错了是真问题
- 病历系统的数据口径、编码（ICD/手术操作编码）不在 WordCraft 的能力范围内
- 需要行业专家校对，不是写个 skill 就能用
- **性价比低于 P0-P2**

真要做，**先做"科研论文"**（P2），医疗往后放。

### P4 — 自媒体文案

⚠️ **这个方向和前面不同，需要重新想清楚定位。**

自媒体的核心痛点是**平台适配**（公众号 / 小红书 / 知乎 / 头条的排版规则、封面、字数限制），
不是"生成文案"。WordCraft 在这里能做的：
- 把 Markdown/纯文本 → 排版好的 docx / PDF（稿子、方案书）
- 批量出多平台版本（同一内容，不同版式）
- 公众号长图的排版

**但它不是"自媒体运营"工具**（选题、投放、数据分析、封面图都不在）。
如果目标是"自媒体文案生成"，那 WordCraft 是**排版末端**，不是主战场。

**建议**：把它定位成 `content-layout`（内容排版）而不是"自媒体 skill"，
面向所有需要"把内容变成规范文档"的场景——自媒体只是其中一种。

---

## 五、架构

```
┌──────────────────────────────────────────────┐
│  内容层（已有，Apache-2.0 / MIT，**不重做**）      │
│                                                │
│  cn-gov-soe-writing-router   WorkMind 法务      │
│  公文/汇报/纪要/研究/审核        咨询/起诉状/审查   │
└───────────────┬──────────────────────────────┘
                │  内容锁定（不再改事实）
                ▼
┌──────────────────────────────────────────────┐
│  排版层（本仓库做）                              │
│                                                │
│  wordcraft（核心 skill：命令、验证、中文排版）      │
│  gongwen    公文版式 GB/T 9704                  │
│  legal-docs 法务文书版式                          │
│  thesis     论文版式 GB/T 7713                   │
└───────────────┬──────────────────────────────┘
                ▼
        合规的 .docx / PDF
```

**分工原则**：
- 内容层不碰排版，排版层不碰事实
- 排版层的 skill **只接受"已锁定的内容"**，不生成、不改写、不补全
- 每层都能独立被使用（不装 cn-gov-soe，也能用 gongwen）

---

## 六、下一步

| # | 事项 | 工作量 | 说明 |
|---|---|---|---|
| 1 | `skills/gongwen/`（SKILL.md + GB/T 9704 版式表 + 版式脚本） | 中 | **P0，先做这个** |
| 2 | 给 `wordcraft` skill 加 `evidence.md`（借鉴 artcraft-skills 的模式） | 小 | 事实断言标注 commit |
| 3 | `skills/legal-docs/` | 中 | 对接 WorkMind 的渲染层 |
| 4 | `skills/thesis/` | 中 | GB/T 7713 版式 |
| 5 | 把 `cn-gov-soe-writing-router` 作为**可选依赖**写进文档（不 vendor） | 小 | 它自己会路由到排版 skill |

**明确不做**：医疗文书（行业责任）、自媒体运营（不是 WordCraft 的战场）、
重做内容层（已有更好的）。

---

## 附：本次调研的核实方法

| 结论 | 怎么核的 |
|---|---|
| `cn-gov-soe-writing-router` 的排版缺口 | 读了它的 `SKILL.md` 全文 + `references/routing-matrix.md` + `material-types.md` + `quality-checklist.md`；全文无 "WordCraft" 字样，版式只有一句检查项 |
| WorkMind 用 python-docx | 读了 `generate_complaint.py` / `generate_opinion.py` 的 import 段 |
| WorkMind 法条库 72 部 | 读 `README.md` 与 `knowledge/` 目录 |
| Anthropic docx skill 是 proprietary | 读 `skills/docx/LICENSE.txt` 与仓库 README 的许可说明段 |
| artcraft-skills 是 MIT 且有 wordcraft skill | GitHub API 读 license + 文件树 + `LICENSE` 全文 |
| 两个法务 skill 不可用 | GitHub API 文件树里**没有** LICENSE 文件 |
| GB/T 9704 数值 | 公开标准全文（openstd.samr.gov.cn）与多所高校转载件交叉核对：上白边 37mm、左白边 28mm、版心 156×225mm、正文 3 号仿宋、单页码居右空一字 |
| WordCraft 命令与参数 | `wordcraft-cli commands --json`（633 条）逐条核对 |
