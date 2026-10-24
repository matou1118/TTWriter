#!/usr/bin/env python3
"""错题本：录入、间隔重复调度、重练卷生成。

数据是用户自己的（默认 ~/Documents/wrongbook.json，--db 改路径）：
不联网、不进仓库、forget 只标记不物理删。

间隔阶梯（艾宾浩斯）：1 → 2 → 4 → 7 → 15 → 30 → 60 天。
答对进下一档；答错重置回 1 天；满 7 档仍对 → 已掌握（不再排期）。

用法见 SKILL.md；子命令：
  add / review / quiz / list / stats / forget
"""

from __future__ import annotations

import argparse
import datetime
import json
import subprocess
import sys
import tempfile
from pathlib import Path

STAGES = [1, 2, 4, 7, 15, 30, 60]  # 天
DB_VERSION = 1


def default_db() -> Path:
    docs = Path.home() / "Documents"
    return docs / "wrongbook.json" if docs.is_dir() else Path.home() / "wrongbook.json"


def today() -> datetime.date:
    return datetime.date.today()


def load(db: Path) -> dict:
    if not db.exists():
        return {"version": DB_VERSION, "items": []}
    try:
        # utf-8-sig：带不带 BOM 都读得进（记事本存 JSON 常带 BOM）。
        data = json.loads(db.read_text(encoding="utf-8-sig"))
    except (json.JSONDecodeError, OSError) as e:
        sys.exit(f"题库读不了（{db}）：{e}——先修好或换个 --db")
    if not isinstance(data.get("items"), list):
        sys.exit(f"题库格式不对（items 不是列表）——{db}")
    return data


def save(db: Path, data: dict) -> None:
    db.parent.mkdir(parents=True, exist_ok=True)
    db.write_text(json.dumps(data, ensure_ascii=False, indent=1), encoding="utf-8")


def next_id(data: dict) -> int:
    return max((it.get("id", 0) for it in data["items"]), default=0) + 1


def parse_date(s: str) -> datetime.date:
    return datetime.date.fromisoformat(s)


def fmt_date(d: datetime.date) -> str:
    return d.isoformat()


# ---------------------------------------------------------------- 录入 ----

def cmd_add(a: argparse.Namespace) -> int:
    db = Path(a.db)
    data = load(db)
    q = a.q.strip()
    if not q:
        sys.exit("题目（--q）不能为空")
    answer = (a.answer or "").strip()
    if not answer:
        sys.exit("答案（--answer）不能为空——没有答案的题没法复习（真的不知道答案就先抄标准答案进来）")
    options = [o.strip() for o in (a.options or "").split("|") if o.strip()] if a.options else []
    item = {
        "id": next_id(data),
        "subject": (a.subject or "未分科").strip(),
        "q": q,
        "options": options,
        "answer": answer,
        "cause": (a.cause or "未归类").strip(),
        "source": (a.source or "").strip(),
        "added": fmt_date(today()),
        "stage": 0,
        "due": fmt_date(today() + datetime.timedelta(days=STAGES[0])),
        "history": [],
        "mastered": False,
        "forgotten": False,
    }
    data["items"].append(item)
    save(db, data)
    print(f"[录入] #{item['id']}（{item['subject']}·{item['cause']}）明日到期复习")
    return 0


# ---------------------------------------------------------------- 复习 ----

def due_items(data: dict) -> list[dict]:
    t = fmt_date(today())
    return [it for it in data["items"] if not it.get("forgotten") and not it.get("mastered") and it.get("due", "") <= t]


def cmd_review(a: argparse.Namespace) -> int:
    data = load(Path(a.db))
    items = due_items(data)
    if not items:
        print("今天没有到期的错题——明天再来（或 `quiz --due-only false` 全库练）。")
        return 0
    if not a.do:
        print(f"今日到期 {len(items)} 题：")
        for it in items:
            opts = f"　选项：{'；'.join(it['options'])}" if it.get("options") else ""
            print(f"  #{it['id']} [{it['subject']}·{it['cause']}]（第 {it['stage'] + 1} 档，答对进 {STAGES[min(it['stage'] + 1, len(STAGES) - 1)]} 天档）")
            print(f"       {it['q']}{opts}")
        print("\n做完自评：--do 编号,编号 --result ok|miss")
        return 0
    ids = []
    for part in a.do.split(","):
        part = part.strip()
        if part.isdigit():
            ids.append(int(part))
        else:
            sys.exit(f"--do 里只能是逗号分隔的编号：{a.do!r}")
    if a.result not in ("ok", "miss"):
        sys.exit("--result 只能是 ok 或 miss")
    by_id = {it["id"]: it for it in data["items"]}
    t = fmt_date(today())
    for i in ids:
        it = by_id.get(i)
        if it is None:
            sys.exit(f"没有 #{i} 这道题")
        if i not in {x["id"] for x in items}:
            print(f"[跳过] #{i} 今天不在到期列表里（due={it.get('due')}）——只记录到期题")
            continue
        it.setdefault("history", []).append({"date": t, "result": a.result})
        if a.result == "ok":
            it["stage"] = min(it["stage"] + 1, len(STAGES) - 1)
            if it["stage"] >= len(STAGES) - 1:
                # 60 天档答对：已掌握（历史还在，可随时重开）。
                it["mastered"] = True
                it["due"] = ""
                print(f"[掌握] #{i} 满 7 档仍对——毕业！")
            else:
                gap = STAGES[it["stage"]]
                it["due"] = fmt_date(today() + datetime.timedelta(days=gap))
                print(f"[答对] #{i} 进 {gap} 天档，下次 {it['due']}")
        else:
            it["stage"] = 0
            it["due"] = fmt_date(today() + datetime.timedelta(days=STAGES[0]))
            print(f"[答错] #{i} 重置回 {STAGES[0]} 天档，明天再战")
    save(Path(a.db), data)
    return 0


# ---------------------------------------------------------------- 组卷 ----

def cmd_quiz(a: argparse.Namespace) -> int:
    data = load(Path(a.db))
    pool = data["items"] if not a.due_only else due_items(data)
    if a.subject:
        pool = [it for it in pool if it.get("subject") == a.subject]
    if a.cause:
        pool = [it for it in pool if it.get("cause") == a.cause]
    pool = [it for it in pool if not it.get("forgotten")]
    if not a.mastered:
        pool = [it for it in pool if not it.get("mastered")]
    # 组卷顺序：已掌握的垫底，其余先到期的（最急）在前，同一天按录入时间。
    pool = sorted(pool, key=lambda it: (bool(it.get("mastered")), it.get("due") or "9999", it.get("added", "")))
    if a.limit and a.limit > 0:
        pool = pool[: a.limit]
    if not pool:
        sys.exit("没有符合条件的题（放宽 --due-only false / --subject / --limit）")

    # ---- blocks JSON（与 legal-docs 的 format 管线同契约；试卷走通用信纸版式）----
    # 标题由 letter 版式的 title 字段出，blocks 里不再放 heading（避免双标题）。
    blocks: list[dict] = [
        {"type": "paragraph", "text": f"共 {len(pool)} 题　·　{fmt_date(today())}　·　姓名：____________"},
        {"type": "paragraph", "text": ""},
    ]
    for n, it in enumerate(pool, 1):
        blocks.append({"type": "paragraph", "text": f"{n}.（{it['subject']}·{it['cause']}）{it['q']}"})
        for opt in it.get("options", []):
            blocks.append({"type": "paragraph", "text": f"　　{opt}"})
        blocks.append({"type": "paragraph", "text": ""})
    # 参考答案独立成节（打印前删）。
    blocks.append({"type": "heading", "text": "参考答案（打印前删除本节）"})
    for n, it in enumerate(pool, 1):
        blocks.append({"type": "paragraph", "text": f"{n}. {it['answer']}"})

    out = {"kind": "letter", "title": a.title or "错题重练卷", "blocks": blocks, "masked": False}
    # 排版借 legal-docs 的 format_legal.py（同一 blocks 契约、通用信纸版式）。
    legal = Path(__file__).resolve().parent.parent.parent / "legal-docs" / "scripts" / "format_legal.py"
    if not legal.exists():
        sys.exit(f"找不到排版器：{legal}（wrongbook 的组卷依赖 legal-docs 技能的 format_legal.py）")
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)
        data_path = f.name
    try:
        layout = [sys.executable, str(legal), data_path, "-o", a.out]
        if a.pdf:
            layout.append("--pdf")
        r = subprocess.run(layout, capture_output=True)
        if r.returncode != 0:
            sys.exit(r.stderr.decode("utf-8", "replace"))
    finally:
        Path(data_path).unlink(missing_ok=True)
    print(f"[组卷] {a.out}（{len(pool)} 题；答案在文末节）")
    print("[提示] 要考场形态（A3 双栏密封线）→ exam 技能；平时自练这份就够。")
    return 0


# ---------------------------------------------------------------- 查看 ----

def cmd_list(a: argparse.Namespace) -> int:
    data = load(Path(a.db))
    items = data["items"]
    if a.subject:
        items = [it for it in items if it.get("subject") == a.subject]
    if not items:
        print("题库是空的（或该科目没有题）——先 add。")
        return 0
    for it in items:
        state = "已删" if it.get("forgotten") else ("已掌握" if it.get("mastered") else f"due {it.get('due', '?')}")
        print(f"  #{it['id']:>3} [{it.get('subject', '?')}·{it.get('cause', '?')}] {state}　{it['q'][:60]}{'…' if len(it['q']) > 60 else ''}")
    return 0


def cmd_stats(a: argparse.Namespace) -> int:
    data = load(Path(a.db))
    items = [it for it in data["items"] if not it.get("forgotten")]
    due = [it for it in items if not it.get("mastered") and it.get("due", "") <= fmt_date(today())]
    mastered = [it for it in items if it.get("mastered")]
    causes: dict[str, int] = {}
    subjects: dict[str, int] = {}
    for it in items:
        causes[it.get("cause", "?")] = causes.get(it.get("cause", "?"), 0) + 1
        subjects[it.get("subject", "?")] = subjects.get(it.get("subject", "?"), 0) + 1
    print(f"在库 {len(items)} 题（已删 {len(data['items']) - len(items)} 不计）：今日待复习 {len(due)}，已掌握 {len(mastered)}")
    for name, d in (("科目", subjects), ("错因", causes)):
        top = sorted(d.items(), key=lambda kv: -kv[1])[:6]
        if top:
            print(f"{name}分布：" + "、".join(f"{k}×{v}" for k, v in top))
    return 0


def cmd_forget(a: argparse.Namespace) -> int:
    data = load(Path(a.db))
    for it in data["items"]:
        if it.get("id") == a.id:
            if it.get("forgotten"):
                it["forgotten"] = False
                it["mastered"] = False
                it["stage"] = 0
                it["due"] = fmt_date(today() + datetime.timedelta(days=STAGES[0]))
                save(Path(a.db), data)
                print(f"[恢复] #{a.id} 回到题库，明天到期")
            else:
                it["forgotten"] = True
                save(Path(a.db), data)
                print(f"[标记删除] #{a.id}（数据保留，forget 再次执行可恢复）")
            return 0
    sys.exit(f"没有 #{a.id} 这道题")


# ---------------------------------------------------------------- 入口 ----

def main() -> int:
    ap = argparse.ArgumentParser(description="错题本：录入 / 间隔重复 / 重练卷")
    ap.add_argument("--db", default=str(default_db()), help=f"题库 JSON（默认 {default_db()}）")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("add", help="录入一道错题")
    p.add_argument("--subject", default="", help="科目（数学/物理/英语…）")
    p.add_argument("--q", required=True, help="题干（定稿文本）")
    p.add_argument("--options", default="", help="选项，竖线分隔：'A.1|B.2|C.3|D.4'")
    p.add_argument("--answer", required=True, help="正确答案（含过程）")
    p.add_argument("--cause", default="", help="错因（概念不清/计算失误/审题错误/方法不会/粗心大意/时间不够）")
    p.add_argument("--source", default="", help="来源（哪次考试第几题）")
    p.set_defaults(fn=cmd_add)

    p = sub.add_parser("review", help="今日到期清单 / 记录结果")
    p.add_argument("--do", default="", help="做完的题号（逗号分隔）")
    p.add_argument("--result", default="", choices=["", "ok", "miss"], help="自评结果")
    p.set_defaults(fn=cmd_review)

    p = sub.add_parser("quiz", help="出重练卷（docx，可 --pdf）")
    p.add_argument("-o", "--out", required=True)
    p.add_argument("--pdf", action="store_true")
    p.add_argument("--subject", default="")
    p.add_argument("--cause", default="")
    p.add_argument("--due-only", type=lambda s: s.lower() != "false", default=True, help="只取到期题（false=全库）")
    p.add_argument("--mastered", action="store_true", help="包含已掌握的题")
    p.add_argument("--limit", type=int, default=0, help="最多出几题（0=不限）")
    p.add_argument("--title", default="错题重练卷")
    p.set_defaults(fn=cmd_quiz)

    p = sub.add_parser("list", help="看题库")
    p.add_argument("--subject", default="")
    p.set_defaults(fn=cmd_list)

    p = sub.add_parser("stats", help="统计")
    p.set_defaults(fn=cmd_stats)

    p = sub.add_parser("forget", help="标记删除（再执行一次恢复）")
    p.add_argument("id", type=int)
    p.set_defaults(fn=cmd_forget)

    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
