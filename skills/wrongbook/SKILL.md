---
name: wrongbook
description: 错题本——录入错题（题目、答案、错因）、艾宾浩斯间隔重复复习调度（1-2-4-7-15-30-60 天）、一键生成「错题重练卷」（按到期题组卷，复用 exam 排版出 docx/PDF）。Use when the user wants 错题本/错题整理/复习计划/重练卷/艾宾浩斯/间隔重复, mentions 错题回顾、把做错的题再练一遍、复习打卡、考前突击错题, or provides 错题照片文字/题目与错因 and asks to 建错题库、安排复习、出重练卷. This skill MANAGES the wrong-answer data and review schedule; the quiz PAPER layout reuses the exam skill; it never invents questions or answers.
license: Apache-2.0
compatibility: Needs the wordcraft-cli binary on PATH. No network access needed.
metadata:
  author: 蜻蜓文档 TTWriter
  version: "1.0"
---

# 错题本（间隔重复 + 重练卷）

三件事：**录错题** → **到点复习** → **攒够出卷**。题库是用户自己的 JSON 文件
（默认 `~/Documents/wrongbook.json`，改路径用 `--db`）。

## 铁律

1. **不编题**：题目、答案、错因必须来自录入（照片转写交给 OCR/人工，录入时校对）。
2. **不代答**：复习记录只存「对/错」，答题是人的事。
3. **数据是用户的**：题库文件不进仓库、不联网、不删（`forget` 只标记不物理删）。

## 工作流

```
① add 录入 → ② review 每日打卡 → ③ quiz 到期攒够出卷 → ④ exam 技能排版
```

### ① 录入

```bash
python scripts/wrongbook.py add \
  --subject 数学 --q "已知 f(x)=x²+2x，求 f'(1)" \
  --answer "f'(x)=2x+2，f'(1)=4" \
  --cause 计算失误 --source "10月月考·第12题"

# 选择题带选项：--options "A.1|B.2|C.3|D.4"
```

错因建议词表：概念不清 / 计算失误 / 审题错误 / 方法不会 / 粗心大意 /
时间不够（自定义亦可）。录入当天即进入第 1 次复习（明天到期）。

### ② 每日复习

```bash
python scripts/wrongbook.py review           # 列出今天到期的题
python scripts/wrongbook.py review --do 3,5 --result ok   # 第3、5题答对
python scripts/wrongbook.py review --do 7 --result miss    # 第7题又错了
```

间隔阶梯（艾宾浩斯）：**1 → 2 → 4 → 7 → 15 → 30 → 60 天**。答对进入下一档，
答错**重置回 1 天**。连续答对满 7 档（60 天后仍对）自动标记「已掌握」，不再排期。

### ③ 出重练卷

```bash
python scripts/wrongbook.py quiz -o 重练卷.docx --pdf \
  --subject 数学 --limit 20            # 到期+未掌握的题，最多 20 道
python scripts/wrongbook.py quiz -o 卷.docx --due-only false --limit 50   # 全库
```

输出是标准题目清单（题号、题目、选项、留白；答案在文末「参考答案」节，打印前删）。
卷面要考场形态（A3 双栏密封线）时，把 `quiz --list` 的题清单交给 **exam 技能**排版。

### ④ 数据与统计

```bash
python scripts/wrongbook.py list --subject 数学       # 按科目看全库
python scripts/wrongbook.py stats                     # 总量/待复习/已掌握/错因分布
python scripts/wrongbook.py forget 3                  # 标记删除第 3 题（不再出卷）
```

## 题库格式（JSON，用户数据）

```json
{"version": 1, "items": [{
  "id": 3, "subject": "数学", "q": "…", "options": ["A.1", "B.2"],
  "answer": "…", "cause": "计算失误", "source": "10月月考·第12题",
  "added": "2026-10-23", "stage": 0, "due": "2026-10-24",
  "history": [{"date": "2026-10-24", "result": "miss"}],
  "mastered": false, "forgotten": false
}]}
```

## 边界

- 题目**拍照识别**：用 `wordcraft-cli pdf ocr`（装 tesseract）转文字后人工校对录入——
  本技能不做识别，只管数据。
- **卷面排版**（A3 双栏、密封线、答题卡）：exam 技能的领域，见其 SKILL.md。
- 本工具不判断「学会了没有」——只按记录调度，判断在人。
