#!/usr/bin/env python3
"""借条 / 欠条 / 收条生成器：参数进、合规文书出。

九要素内置（check_legal.py 的体检项在生成时就满足）：
  双方身份、金额大小写并存、用途、期限、利率（超 LPR 四倍拒绝生成）、
  交付方式、还款方式、签名手印、日期。

用法：
  python jietiao.py --kind jietiao -o 借条.docx \
      --borrower 李四 --borrower-id 110101199001011234 \
      --lender 张三 --lender-id 110101198805052345 \
      --amount 50000 --rate 8 --years 0 --months 6 \
      --purpose 经营周转 --delivery transfer --repay onetime

  python jietiao.py --kind qiantiao -o 欠条.docx ...   # 欠条（--reason 货款）
  python jietiao.py --kind shoutiao -o 收条.docx ...   # 收条（--for 房租）

利率规则：--rate 超过一年期 LPR 四倍（默认按 15.4% 上限）直接拒绝生成——
超出部分法院不支持，写上去是自欺。--rate 0 视为无息（写明）。

大写金额算法：元角分 → 壹贰叁……佰仟萬亿，零的连读规则按会计惯例。
"""
import argparse
import calendar
import datetime
import json
import math
import re
import subprocess
import sys
import tempfile
from decimal import ROUND_HALF_UP, Decimal
from pathlib import Path

LPR4X_CAP = 15.4
DIGITS = "零壹贰叁肆伍陆柒捌玖"
UNITS = ["", "拾", "佰", "仟"]
BIGUNITS = ["", "萬", "亿", "兆"]


def to_capital(amount: float) -> str:
    """金额 → 人民币大写（会计惯例的零连读规则）。"""
    if not math.isfinite(amount) or amount < 0 or amount >= 10**13:
        raise ValueError(f"金额超出可表达范围：{amount}")
    cents = int((Decimal(str(amount)) * 100).to_integral_value(rounding=ROUND_HALF_UP))
    if cents == 0:
        return "人民币零元整"
    yuan, fen = divmod(cents, 100)
    jiao, fen = divmod(fen, 10)

    def four(n: int) -> str:
        s = ""
        zero_pending = False
        for pos in range(3, -1, -1):
            d = n // 10**pos % 10
            if d == 0:
                if s and not zero_pending:
                    zero_pending = True
            else:
                if zero_pending:
                    s += "零"
                    zero_pending = False
                s += DIGITS[d] + UNITS[pos]
        return s

    def full_whole(n: int) -> str:
        # 分组转写：每组四位 + 组单位；组间「零」补位且只补一个（100001 → 壹拾万零壹元）。
        groups: list[int] = []
        while n > 0:
            groups.append(n % 10_000)
            n //= 10_000
        # 从高位组往低位组写；skip_zero 在零组与非零组交界处补一个零。
        out = ""
        pending_zero = False
        for gi in range(len(groups) - 1, -1, -1):
            chunk = groups[gi]
            if chunk == 0:
                pending_zero = bool(out)
                continue
            if out:
                # 与前一组的衔接：前一零组（pending）或本组高位缺位（<1000）→ 补零。
                if pending_zero or chunk < 1000:
                    out += "零"
            out += four(chunk) + BIGUNITS[gi]
            pending_zero = False
        return out or "零"

    cap = full_whole(yuan) + "元" if yuan > 0 else ""
    if yuan > 0 and jiao == 0 and fen == 0:
        cap += "整"
    elif not cap:
        # 纯角/分金额：零伍分、零叁角
        cap = "零" + (DIGITS[jiao] + "角" if jiao > 0 else "") + (DIGITS[fen] + "分" if fen > 0 else "")
    else:
        if cap and jiao == 0:
            cap += "零"
        if jiao > 0:
            cap += DIGITS[jiao] + "角"
        if fen > 0:
            cap += DIGITS[fen] + "分"
    return f"人民币{cap}"


def fmt_amount(amount: float) -> str:
    return f"¥{amount:,.2f}"


def add_months(d: datetime.date, years: int, months: int) -> datetime.date:
    m = d.month + years * 12 + months
    y = d.year + (m - 1) // 12
    m = (m - 1) % 12 + 1
    # 月末钳到目标月最后一天
    return d.replace(year=y, month=m, day=min(d.day, calendar.monthrange(y, m)[1]))


def build_blocks(a) -> dict:
    """参数 → format_legal.py 的 JSON（blocks 结构）。"""
    today = datetime.date.today()
    due = add_months(today, a.years, a.months) if (a.years or a.months) else None
    capital = to_capital(a.amount)
    small = fmt_amount(a.amount)

    if a.kind == "jietiao":
        lines = [
            {"type": "paragraph", "text": f"借款人：{a.borrower}（身份证号：{a.borrower_id}）"},
            {"type": "paragraph", "text": f"出借人：{a.lender}（身份证号：{a.lender_id}）"},
            {"type": "paragraph", "text": f"今借到 {a.lender} {capital}（小写：{small}）。"},
            {"type": "paragraph", "text": f"借款用途：{a.purpose}。"},
        ]
        if due:
            lines.append({"type": "paragraph",
                          "text": f"借款期限：自 {today.year} 年 {today.month} 月 {today.day} 日"
                                  f"至 {due.year} 年 {due.month} 月 {due.day} 日。"})
        else:
            lines.append({"type": "paragraph", "text": "借款期限：双方另行约定。"})
        if a.rate and a.rate > 0:
            lines.append({"type": "paragraph",
                          "text": f"利率：年利率 {a.rate:g}%（单利，逾期按同利率继续计算）。"})
        else:
            lines.append({"type": "paragraph", "text": "利率：本笔借款无息。"})
        deliver = "银行转账至借款人指定账户（户名：{}）".format(a.borrower) if a.delivery == "transfer" else "现金交付（出借人当面点清）"
        lines.append({"type": "paragraph", "text": f"交付方式：{deliver}。"})
        repay = "到期一次性归还本金及利息" if a.repay == "onetime" else "分期归还（每期金额与日期由双方另行书面确认）"
        lines.append({"type": "paragraph", "text": f"还款方式：{repay}。"})
        lines.append({"type": "sign", "lines": [
            "借款人（签名 + 按手印）：",
            "",
            "",
            "",
            f"{today.year} 年 {today.month} 月 {today.day} 日",
        ]})
        return {"kind": "letter", "title": "借条", "blocks": lines, "masked": False}

    if a.kind == "qiantiao":
        lines = [
            {"type": "paragraph", "text": f"欠款人：{a.borrower}（身份证号：{a.borrower_id}）"},
            {"type": "paragraph", "text": f"债权人：{a.lender}（身份证号：{a.lender_id}）"},
            {"type": "paragraph", "text": f"因 {a.reason or '[待补充事由]'}，{a.borrower} 尚欠 {a.lender} {capital}（小写：{small}）。"},
        ]
        if due:
            lines.append({"type": "paragraph",
                          "text": f"还款日期：{due.year} 年 {due.month} 月 {due.day} 日前。"})
        else:
            lines.append({"type": "paragraph", "text": "还款日期：双方另行约定。"})
        if a.rate and a.rate > 0:
            lines.append({"type": "paragraph", "text": f"利率：年利率 {a.rate:g}%。"})
        else:
            lines.append({"type": "paragraph", "text": "利率：本笔欠款无息。"})
        lines.append({"type": "sign", "lines": [
            "欠款人（签名 + 按手印）：",
            "",
            "",
            "",
            f"{today.year} 年 {today.month} 月 {today.day} 日",
        ]})
        return {"kind": "letter", "title": "欠条", "blocks": lines, "masked": False}

    # 收条
    lines = [
        {"type": "paragraph", "text": f"交款人：{a.lender}（身份证号：{a.lender_id}）"},
        {"type": "paragraph", "text": f"收款人：{a.borrower}（身份证号：{a.borrower_id}）"},
        {"type": "paragraph", "text": f"今收到 {a.lender} 交来{a.for_ or '[待补充款项性质]'}{capital}（小写：{small}）。"},
        {"type": "sign", "lines": [
            "收款人（签名）：",
            "",
            "",
            "",
            f"{today.year} 年 {today.month} 月 {today.day} 日",
        ]},
    ]
    return {"kind": "letter", "title": "收条", "blocks": lines, "masked": False}


def main() -> int:
    ap = argparse.ArgumentParser(description="借条/欠条/收条生成器（九要素合规）")
    ap.add_argument("--kind", default="jietiao", choices=["jietiao", "qiantiao", "shoutiao"])
    ap.add_argument("-o", "--out", required=True)
    ap.add_argument("--lender", required=True, help="出借人/交款人姓名")
    ap.add_argument("--lender-id", required=True, help="出借人身份证号（18 位）")
    ap.add_argument("--borrower", required=True, help="借款人/收款人姓名")
    ap.add_argument("--borrower-id", required=True, help="借款人身份证号（18 位）")
    ap.add_argument("--amount", type=float, required=True, help="金额（元）")
    ap.add_argument("--rate", type=float, default=0.0, help="年利率 %%（0=无息）")
    ap.add_argument("--years", type=int, default=0)
    ap.add_argument("--months", type=int, default=0, help="期限月数")
    ap.add_argument("--purpose", default="经营周转", help="借款用途")
    ap.add_argument("--reason", default="", help="欠条：欠款事由")
    ap.add_argument("--for", dest="for_", default="", help="收条：款项性质")
    ap.add_argument("--delivery", default="transfer", choices=["transfer", "cash"])
    ap.add_argument("--repay", default="onetime", choices=["onetime", "installment"])
    ap.add_argument("--pdf", action="store_true")
    args = ap.parse_args()

    # 参数校验（拒绝生成不合规文书）
    if not math.isfinite(args.amount) or args.amount <= 0:
        sys.exit("金额必须为正数")
    if not math.isfinite(args.rate) or args.rate < 0:
        sys.exit("利率不能为负或非数值")
    if args.rate > LPR4X_CAP:
        sys.exit(f"年利率 {args.rate}% 超过 LPR 四倍上限（≈{LPR4X_CAP}%）——"
                 f"超出部分法院不支持，拒绝生成。降到上限内再试。")
    if not args.lender.strip() or not args.borrower.strip():
        sys.exit("双方姓名不能为空")
    for who, pid in (("出借人", args.lender_id), ("借款人", args.borrower_id)):
        if not re.fullmatch(r"\d{6}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:[0-2]\d|3[01])\d{3}[\dXx]", pid):
            sys.exit(f"{who}身份证号「{pid}」不是合法的 18 位身份证号——借条要全文身份号")
    if args.years < 0 or args.months < 0:
        sys.exit("期限不能为负")
    if args.years * 12 + args.months > 600:
        sys.exit("期限不能超过 50 年")

    data = build_blocks(args)
    script_dir = Path(__file__).resolve().parent
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=1)
        data_path = f.name

    try:
        # 生成器输出走 format_legal.py 的排版（同一版式规范）
        layout = [sys.executable, str(script_dir / "format_legal.py"), data_path, "-o", args.out]
        if args.pdf:
            layout.append("--pdf")
        r = subprocess.run(layout, capture_output=True)
        if r.returncode != 0:
            sys.exit(r.stderr.decode("utf-8", "replace"))
    finally:
        Path(data_path).unlink(missing_ok=True)

    print(f"[生成] {args.out}")
    print(f"[要素] 金额 {to_capital(args.amount)} / {fmt_amount(args.amount)}；"
          f"利率 {'无息' if not args.rate else f'{args.rate:g}%'}；期限 {args.years * 12 + args.months} 个月")
    print("[提醒] 打印后签名+按手印；转账留流水；现金当场点清并写明。本工具不构成法律意见。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
