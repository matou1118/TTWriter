#!/usr/bin/env python3
"""合同生成器：参数进、条款骨架出——四类高频合同的法定必备条款打底。

生成的合同是**骨架**：每项必备条款都在、结构合规，但具体事实（工资数额、
房屋地址、验收标准）留给填写方——不编、不猜。缺的参数落 `[待补充]`。

四类（--kind）：
  labor    劳动合同（劳动合同法 §17 必备条款全套）
  service  服务合同
  sales    买卖合同（民法典 §510/§511 约定不明时的填补规则对应的必备项）
  lease    房屋租赁合同

红线（拒绝生成，同 jietiao 的 LPR 四倍思路）：
  labor: 试用期超劳动合同法 §19 上限（合同期<1年试≤1月；1–3年试≤2月；≥3年试≤6月），
         合同期不满 3 个月的不得约定试用期（§19 第三款）
  lease: 押金超过 2 个月租金只警告（各地规定不一，写清城市后自行核对）

用法：
  python contract_gen.py --kind labor -o 合同.docx \
      --party-a "××科技有限公司" --party-b "李四" \
      --term-months 36 --probation-months 3 --wage 12000

  python contract_gen.py --kind lease -o 租约.docx \
      --party-a "王五" --party-b "赵六" --rent 3500 --deposit 7000

生成后建议跑 check_legal.py 体检，再人工过一遍条款。
本工具不构成法律意见；重大交易请律师审。
"""

from __future__ import annotations

import argparse
import datetime
import json
import subprocess
import sys
import tempfile
from pathlib import Path

# 大写金额与 jietiao.py 同一实现（会计惯例的零连读）。
from jietiao import to_capital


def ph(v: str | None, what: str) -> str:
    """值或占位（不编事实）。"""
    v = (v or "").strip()
    return v if v else f"[待补充{what}]"


def money(v: float | None, what: str) -> str:
    """金额行：数值 + 大写并存（check_legal 的体检项，jietiao 同款惯例）。"""
    if v is None:
        return f"[待补充{what}]"
    return f"{v:,.2f} 元（大写：{to_capital(v)}）"


def party_line(name: str | None, id_: str | None, role: str) -> str:
    """甲方/乙方行：名称 + 证件号（身份证/统一社会信用代码都可）。"""
    n = ph(name, f"{role}名称")
    i = ph(id_, f"{role}证件号")
    return f"{role}（{n}，证件号：{i}）"


def date_or(d: str | None, offset_days: int = 0) -> str:
    if d:
        return d
    base = datetime.date.today() + datetime.timedelta(days=offset_days)
    return f"[签署时填写：约 {base.isoformat()}]"


# ---------------------------------------------------------------- 条款库 ----

def labor_blocks(a: argparse.Namespace) -> tuple[list[dict], list[str]]:
    today = datetime.date.today()
    end_hint = today + datetime.timedelta(days=a.term_months * 30) if a.term_months else None
    blocks = [
        {"type": "heading", "text": "劳动合同"},
        {"type": "paragraph", "text": f"签订日期：{date_or(None)}"},
        {"type": "paragraph", "text": f"签订地点：{ph(a.place, '签订地点')}"},

        {"type": "heading", "text": "一、合同双方"},
        {"type": "paragraph", "text": f"用人单位（甲方）：{ph(a.party_a, '单位全称')}（统一社会信用代码：{ph(a.party_a_id, '信用代码')}）"},
        {"type": "paragraph", "text": f"地址：{ph(a.party_a_addr, '单位地址')}"},
        {"type": "paragraph", "text": f"劳动者（乙方）：{ph(a.party_b, '姓名')}（身份证号：{ph(a.party_b_id, '身份证号')}）"},
        {"type": "paragraph", "text": f"住址：{ph(a.party_b_addr, '住址')}"},

        {"type": "heading", "text": "二、合同期限"},
        {"type": "paragraph", "text": f"本合同为固定期限劳动合同，自 {date_or(None)} 起至 "
                                       f"{end_hint.isoformat() if end_hint else '[待补充终止日期]'} 止，共 {a.term_months or '[待补充]'} 个月。"},
        {"type": "paragraph", "text": f"其中试用期 {a.probation_months or 0} 个月，自合同生效之日起算。"},
        {"type": "paragraph", "text": "（提示：以完成一定工作任务为期限或无固定期限的，改写本条并删除不适用的表述。）"},

        {"type": "heading", "text": "三、工作内容与工作地点"},
        {"type": "paragraph", "text": f"乙方担任 {ph(a.position, '岗位')} 岗位，工作地点为 {ph(a.workplace, '工作地点')}。"},
        {"type": "paragraph", "text": "岗位职责以双方确认的岗位说明书为准；岗位说明书作为本合同附件。"},

        {"type": "heading", "text": "四、工作时间和休息休假"},
        {"type": "paragraph", "text": "甲方实行标准工时制度：每日工作不超过 8 小时、每周不超过 40 小时，每周至少休息一日。"},
        {"type": "paragraph", "text": "加班依法支付加班费或安排补休；法定节假日、年休假等按国家规定执行。"},
        {"type": "paragraph", "text": "（提示：经批准实行综合计算工时或不定时工作制的，应在本条写明审批文号。）"},

        {"type": "heading", "text": "五、劳动报酬"},
        {"type": "paragraph", "text": f"乙方月工资为 {money(a.wage, '月工资')}（税前），于每月 {a.payday or '[待补充发薪日]'} 日支付上月工资。"},
        {"type": "paragraph", "text": "工资以货币形式足额支付，不得克扣或者无故拖欠。试用期工资不低于约定工资的百分之八十，并不得低于当地最低工资标准。"},

        {"type": "heading", "text": "六、社会保险与福利"},
        {"type": "paragraph", "text": "甲方依法为乙方缴纳养老、医疗、失业、工伤、生育等社会保险费；个人缴纳部分由甲方从乙方工资中代扣代缴。"},
        {"type": "paragraph", "text": "住房公积金按当地规定缴存。"},

        {"type": "heading", "text": "七、劳动保护、劳动条件和职业危害防护"},
        {"type": "paragraph", "text": "甲方提供符合国家规定的劳动安全卫生条件和必要的劳动防护用品；对从事有职业危害作业的乙方，如实告知职业危害及其后果，并定期组织职业健康检查。"},

        {"type": "heading", "text": "八、合同的解除与终止"},
        {"type": "numbered", "items": [
            "双方协商一致，可以解除本合同。",
            "乙方提前三十日（试用期内提前三日）书面通知甲方，可以解除本合同。",
            "甲方有《劳动合同法》第三十八条情形之一的，乙方可以解除本合同且无需通知（危及人身安全的紧急情况除外需立即）。",
            "乙方有《劳动合同法》第三十九条情形之一的，甲方可以解除本合同；第四十条、四十一条情形的，按规定程序与补偿执行。",
            "合同期满或出现法定终止情形的，合同终止；符合法定条件的，依法支付经济补偿。",
        ]},

        {"type": "heading", "text": "九、争议解决"},
        {"type": "paragraph", "text": "因本合同发生争议，双方应先协商；协商不成的，可向劳动争议调解组织申请调解，或向劳动争议仲裁委员会申请仲裁；对仲裁裁决不服的，可依法向人民法院提起诉讼。"},
        {"type": "paragraph", "text": "（提示：劳动争议仲裁是诉讼的前置程序，本条不得约定排除。）"},

        {"type": "sign", "lines": [
            "甲方（盖章）：", "法定代表人或委托代理人（签名）：", "",
            "乙方（签名）：", "", "", "",
            "签署日期：____年__月__日　　签署日期：____年__月__日",
        ]},
    ]
    risks = [
        "试用期与合同期限要匹配法定上限（本工具已校验），超期部分无效。",
        "工资低于当地最低工资标准的条款无效——签署前核对当地标准。",
        "竞业限制条款如需添加，须另行约定经济补偿，期限不超过两年。",
        "社保缴纳是法定义务，「自愿放弃社保」协议无效。",
    ]
    return blocks, risks


def service_blocks(a: argparse.Namespace) -> tuple[list[dict], list[str]]:
    blocks = [
        {"type": "heading", "text": f"{ph(a.subject, '服务事项')}服务合同"},
        {"type": "paragraph", "text": f"合同编号：{ph(a.contract_no, '编号')}"},
        {"type": "paragraph", "text": f"签订日期：{date_or(None)}"},

        {"type": "heading", "text": "一、合同双方"},
        {"type": "paragraph", "text": f"甲方（委托方）：{ph(a.party_a, '名称')}（证件号：{ph(a.party_a_id, '证件号')}）"},
        {"type": "paragraph", "text": f"乙方（服务方）：{ph(a.party_b, '名称')}（证件号：{ph(a.party_b_id, '证件号')}）"},

        {"type": "heading", "text": "二、服务内容与标准"},
        {"type": "paragraph", "text": f"乙方为甲方提供以下服务：{ph(a.subject, '服务内容')}。"},
        {"type": "paragraph", "text": f"服务标准与成果要求：{ph(a.standard, '验收标准（越具体越好：指标、口径、时限）')}。"},
        {"type": "paragraph", "text": "服务范围之外的变更，双方另行书面确认并调整费用与工期。"},

        {"type": "heading", "text": "三、服务期限"},
        {"type": "paragraph", "text": f"自 {date_or(None)} 起至 {ph(a.end_date, '终止日期')} 止；阶段性交付节点以双方确认的排期表为准。"},

        {"type": "heading", "text": "四、费用与支付"},
        {"type": "paragraph", "text": f"服务费总额 {money(a.amount, '总金额')}（含税）。"},
        {"type": "numbered", "items": [
            "首期款：合同签订后 __ 日内支付总额的 __%。",
            "进度款：阶段性成果验收通过后 __ 日内支付总额的 __%。",
            "尾款：全部成果验收通过后 __ 日内支付余额。",
        ]},
        {"type": "paragraph", "text": f"支付方式：{ph(a.pay_method, '支付方式')}；乙方开具 {ph(a.invoice_type, '发票类型')} 发票。"},

        {"type": "heading", "text": "五、验收"},
        {"type": "paragraph", "text": "甲方应在收到交付成果后 __ 日内按第二条标准完成验收并书面确认；逾期未提出书面异议视为验收通过。验收不通过的，乙方应在 __ 日内修改后重新提交。"},

        {"type": "heading", "text": "六、知识产权"},
        {"type": "paragraph", "text": "按本法域惯例二选一并删除另一项："},
        {"type": "numbered", "items": [
            "（1）成果知识产权自甲方付清全部费用之日起归甲方所有，乙方保留其在先技术与方法。",
            "（2）成果知识产权归乙方所有，甲方获得非独占的永久使用许可。",
        ]},

        {"type": "heading", "text": "七、保密"},
        {"type": "paragraph", "text": "双方对履行本合同过程中知悉的对方商业秘密与未公开信息负有保密义务，保密期限至该信息公开为止，本合同终止不影响保密义务的效力。"},

        {"type": "heading", "text": "八、违约责任"},
        {"type": "paragraph", "text": "乙方逾期交付的，每逾期一日按合同总额的 __% 支付违约金；逾期超过 __ 日的，甲方有权解除合同。"},
        {"type": "paragraph", "text": "甲方逾期付款的，每逾期一日按欠付金额的 __% 支付违约金。违约金不足以弥补实际损失的，守约方可另行主张。"},

        {"type": "heading", "text": "九、争议解决"},
        {"type": "paragraph", "text": "因本合同发生争议，双方协商解决；协商不成的，提交 ____________（约定仲裁机构全称，或删除本句改为法院管辖）仲裁/向被告住所地人民法院起诉。"},
        {"type": "paragraph", "text": "（提示：仲裁与法院管辖只能选一，选仲裁须写明准确的仲裁委员会名称。）"},

        {"type": "sign", "lines": [
            "甲方（盖章）：", "法定代表人或授权代表（签名）：", "",
            "乙方（盖章）：", "法定代表人或授权代表（签名）：", "", "",
            "签署日期：____年__月__日　　签署日期：____年__月__日",
        ]},
    ]
    risks = [
        "验收标准写不清是服务合同纠纷第一来源——第二条务必具体到指标与口径。",
        "「或裁或审」条款（仲裁和法院并列）会导致仲裁条款无效。",
        "违约金上限：法院可依当事人申请调整过分高于损失的违约金。",
        "成果知识产权归属要与付款节点联动（见第六条的两种写法）。",
    ]
    return blocks, risks


def sales_blocks(a: argparse.Namespace) -> tuple[list[dict], list[str]]:
    blocks = [
        {"type": "heading", "text": f"{ph(a.subject, '标的物')}买卖合同"},
        {"type": "paragraph", "text": f"合同编号：{ph(a.contract_no, '编号')}"},
        {"type": "paragraph", "text": f"签订日期：{date_or(None)}"},

        {"type": "heading", "text": "一、合同双方"},
        {"type": "paragraph", "text": f"甲方（出卖人）：{ph(a.party_a, '名称')}（证件号：{ph(a.party_a_id, '证件号')}）"},
        {"type": "paragraph", "text": f"乙方（买受人）：{ph(a.party_b, '名称')}（证件号：{ph(a.party_b_id, '证件号')}）"},

        {"type": "heading", "text": "二、标的物"},
        {"type": "paragraph", "text": f"标的物：{ph(a.subject, '名称、规格、型号')}。"},
        {"type": "paragraph", "text": f"数量：{ph(a.quantity, '数量与单位')}；质量标准：{ph(a.quality, '国家标准/行业标准/双方约定标准')}。"},
        {"type": "paragraph", "text": "标的物应为全新、无权利瑕疵且不侵犯第三方知识产权。"},

        {"type": "heading", "text": "三、价款"},
        {"type": "paragraph", "text": f"总价款 {money(a.amount, '总金额')}（含税/不含税：二选一删除）。"},
        {"type": "paragraph", "text": "价格包含运输、保险等费用的范围：____________。"},

        {"type": "heading", "text": "四、交付"},
        {"type": "paragraph", "text": f"交付方式：{ph(a.delivery, '交付方式（送货/自提/物流）')}；交付时间：{ph(a.delivery_date, '交付时间')}；交付地点：{ph(a.delivery_place, '交付地点')}。"},
        {"type": "paragraph", "text": "标的物毁损、灭失的风险，交付前由甲方承担，交付后由乙方承担，法律另有规定或双方另有约定的除外。"},

        {"type": "heading", "text": "五、验收"},
        {"type": "paragraph", "text": "乙方应在收到标的物后 __ 日内按第二条标准验收；数量或外观瑕疵当场提出，隐蔽瑕疵在发现后 __ 日内书面通知甲方。逾期未通知的，视为验收合格（不宜写「一经售出概不负责」）。"},

        {"type": "heading", "text": "六、所有权转移与付款"},
        {"type": "paragraph", "text": "标的物所有权自交付时起转移；约定保留所有权条款的（甲方收到全部价款前保留所有权），在此写明。"},
        {"type": "paragraph", "text": f"付款方式与期限：{ph(a.pay_method, '付款方式与期限')}。"},

        {"type": "heading", "text": "七、违约责任"},
        {"type": "paragraph", "text": "甲方逾期交付或交付不符的，乙方有权要求继续履行、补足数量、修理更换或退货，并主张违约金 __%。"},
        {"type": "paragraph", "text": "乙方逾期付款的，每逾期一日按未付款的 __% 支付违约金。"},

        {"type": "heading", "text": "八、争议解决"},
        {"type": "paragraph", "text": "因本合同发生争议，双方协商解决；协商不成的，向 ____________ 人民法院起诉（或约定仲裁——写明仲裁机构全称，与法院管辖二选一）。"},

        {"type": "sign", "lines": [
            "甲方（盖章）：", "法定代表人或授权代表（签名）：", "",
            "乙方（盖章）：", "法定代表人或授权代表（签名）：", "", "",
            "签署日期：____年__月__日　　签署日期：____年__月__日",
        ]},
    ]
    risks = [
        "质量标准只写「合格」两个字等于没写——落到国标编号或可测指标。",
        "验收期限不写，法律默认合理期限，争议空间大；写明更稳。",
        "「概不退换」条款与法定三包/七天无理由（网购）冲突，写了无效。",
        "大额交易建议保留所有权条款（见第六条）。",
    ]
    return blocks, risks


def lease_blocks(a: argparse.Namespace) -> tuple[list[dict], list[str]]:
    blocks = [
        {"type": "heading", "text": "房屋租赁合同"},
        {"type": "paragraph", "text": f"合同编号：{ph(a.contract_no, '编号')}"},
        {"type": "paragraph", "text": f"签订日期：{date_or(None)}"},

        {"type": "heading", "text": "一、合同双方"},
        {"type": "paragraph", "text": f"甲方（出租人）：{ph(a.party_a, '姓名')}（身份证号：{ph(a.party_a_id, '身份证号')}）"},
        {"type": "paragraph", "text": f"乙方（承租人）：{ph(a.party_b, '姓名')}（身份证号：{ph(a.party_b_id, '身份证号')}）"},

        {"type": "heading", "text": "二、房屋状况"},
        {"type": "paragraph", "text": f"租赁房屋坐落于 {ph(a.property, '房屋坐落地址')}（下称「该房屋」）。"},
        {"type": "paragraph", "text": f"建筑面积 {ph(a.area, '面积')}；权属证明：{ph(a.ownership, '产权证号/权属说明')}。甲方保证有权出租该房屋。"},
        {"type": "paragraph", "text": "附属设施与家具家电清单作为附件一，现状双方已核对。"},

        {"type": "heading", "text": "三、租期、租金与押金"},
        {"type": "paragraph", "text": f"租赁期限自 {date_or(None)} 起至 {ph(a.end_date, '终止日期')} 止。"},
        {"type": "paragraph", "text": f"月租金 {money(a.rent, '月租金')}，支付方式：{ph(a.pay_method, '押__付__')}。"},
        {"type": "paragraph", "text": f"押金 {money(a.deposit, '押金')}，租赁期满且结清费用、房屋及设施无损坏的，甲方于乙方返还房屋后 __ 日内无息退还。"},
        {"type": "paragraph", "text": "租期内租金调整：____________（未约定的，租期内不得单方涨租）。"},

        {"type": "heading", "text": "四、费用承担"},
        {"type": "paragraph", "text": "水、电、燃气、物业等费用由 __ 方承担；采暖费由 __ 方承担。"},

        {"type": "heading", "text": "五、维修与使用"},
        {"type": "paragraph", "text": "房屋自然损耗的维修由甲方负责；因乙方使用不当造成的损坏由乙方负责修复或赔偿。"},
        {"type": "paragraph", "text": "乙方不得擅自改变房屋结构或用途；需装修的，经甲方书面同意。"},

        {"type": "heading", "text": "六、转租"},
        {"type": "paragraph", "text": "乙方转租该房屋须经甲方书面同意；未经同意转租的，甲方可以解除合同。"},

        {"type": "heading", "text": "七、合同的解除"},
        {"type": "numbered", "items": [
            "双方协商一致可以解除本合同。",
            "甲方未按约交付房屋、或不履行维修义务致乙方无法正常使用的，乙方可以解除合同。",
            "乙方拖欠租金累计达 __ 日以上、或擅自改变房屋结构用途的，甲方可以解除合同。",
            "任何一方提前解除合同的，应提前 __ 日书面通知对方，并按第条承担违约责任。",
        ]},

        {"type": "heading", "text": "八、违约责任"},
        {"type": "paragraph", "text": "一方违约给对方造成损失的，应赔偿损失；双方另有约定的违约金：____________。"},

        {"type": "heading", "text": "九、争议解决"},
        {"type": "paragraph", "text": "因本合同发生争议，双方协商解决；协商不成的，向房屋所在地人民法院起诉。"},

        {"type": "sign", "lines": [
            "甲方（签名）：", "", "乙方（签名）：", "", "",
            "签署日期：____年__月__日　　签署日期：____年__月__日",
        ]},
    ]
    deposit_warn = ""
    if a.rent and a.deposit is not None and a.deposit > 2 * a.rent:
        deposit_warn = f"押金 {a.deposit:g} 元超过两个月租金（{2 * a.rent:g} 元）——部分城市规定押金不得超过一个月租金，按房屋所在城市核对后再签。"
    risks = [
        deposit_warn or "押金条款写明退还条件与期限（第三条），避免「押金不退」争议。",
        "「买卖不破租赁」：租期内房屋出售的，租赁合同对新房东继续有效。",
        "租期超过六个月未采用书面形式的，视为不定期租赁——双方都可随时解除，务必落纸面。",
        "房东要求代签物业费用条款时，先查该费用是否已由房东缴纳（历史欠费纠纷高发）。",
    ]
    return blocks, risks


KINDS = {
    "labor": (labor_blocks, ["--position", "--wage", "--term-months", "--probation-months", "--workplace", "--payday"]),
    "service": (service_blocks, ["--subject", "--amount", "--standard", "--end-date", "--pay-method"]),
    "sales": (sales_blocks, ["--subject", "--amount", "--quantity", "--quality", "--delivery", "--delivery-date", "--delivery-place", "--pay-method"]),
    "lease": (lease_blocks, ["--property", "--rent", "--deposit", "--end-date", "--pay-method", "--area", "--ownership"]),
}


def main() -> int:
    ap = argparse.ArgumentParser(description="合同生成器：四类高频合同的法定必备条款骨架")
    ap.add_argument("--kind", required=True, choices=sorted(KINDS))
    ap.add_argument("-o", "--out", required=True)
    # 双方信息（四类通用）。
    ap.add_argument("--party-a", default="", help="甲方/用人单位/出租人/出卖人名称")
    ap.add_argument("--party-a-id", default="", help="甲方证件号（身份证或统一社会信用代码）")
    ap.add_argument("--party-b", default="", help="乙方/劳动者/承租人/买受人名称")
    ap.add_argument("--party-b-id", default="", help="乙方证件号")
    ap.add_argument("--contract-no", default="", help="合同编号（缺省留占位）")
    # 类型专属参数：全部可选、缺省占位（argparse 不认识的会报错，所以全注册）。
    ap.add_argument("--position", default="")          # labor
    ap.add_argument("--wage", type=float, default=None)  # labor
    ap.add_argument("--term-months", type=int, default=0)  # labor
    ap.add_argument("--probation-months", type=int, default=0)  # labor
    ap.add_argument("--workplace", default="")         # labor
    ap.add_argument("--payday", default="")            # labor
    ap.add_argument("--party-a-addr", default="")      # labor
    ap.add_argument("--party-b-addr", default="")      # labor
    ap.add_argument("--place", default="")             # labor 签订地点
    ap.add_argument("--subject", default="")           # service/sales
    ap.add_argument("--amount", type=float, default=None)  # service/sales
    ap.add_argument("--standard", default="")          # service
    ap.add_argument("--end-date", default="")          # service/lease
    ap.add_argument("--pay-method", default="")        # service/sales/lease
    ap.add_argument("--invoice-type", default="")      # service
    ap.add_argument("--quantity", default="")          # sales
    ap.add_argument("--quality", default="")           # sales
    ap.add_argument("--delivery", default="")          # sales
    ap.add_argument("--delivery-date", default="")     # sales
    ap.add_argument("--delivery-place", default="")    # sales
    ap.add_argument("--property", default="")          # lease
    ap.add_argument("--rent", type=float, default=None)   # lease
    ap.add_argument("--deposit", type=float, default=None)  # lease
    ap.add_argument("--area", default="")              # lease
    ap.add_argument("--ownership", default="")         # lease
    ap.add_argument("--pdf", action="store_true")
    args = ap.parse_args()

    # ---- 红线校验（拒绝生成不合规骨架）----
    if args.kind == "labor":
        t, p = args.term_months, args.probation_months
        if t < 0 or p < 0:
            sys.exit("期限不能为负")
        if p > 0 and t <= 0:
            sys.exit("劳动合同试用期要有合同期限（--term-months）才能校验上限")
        if t < 3 and p > 0:
            sys.exit("合同期限不满三个月的不得约定试用期（《劳动合同法》第十九条第三款）——删掉试用期，或把 --term-months 加到 3 以上")
        cap = 1 if t < 12 else (2 if t < 36 else 6)
        if p > cap:
            sys.exit(
                f"试用期 {p} 个月超过法定上限：合同期 {t} 个月的试用期不得超过 {cap} 个月"
                f"（《劳动合同法》第十九条）。调整后重试。"
            )
        if args.wage is not None and args.wage < 0:
            sys.exit("工资不能为负")
    if args.kind in ("service", "sales") and args.amount is not None and args.amount < 0:
        sys.exit("金额不能为负")
    if args.kind == "lease":
        if args.rent is not None and args.rent < 0:
            sys.exit("租金不能为负")
        if args.deposit is not None and args.deposit < 0:
            sys.exit("押金不能为负")

    builder, _ = KINDS[args.kind]
    blocks, risks = builder(args)
    data = {"kind": "letter", "title": {"labor": "劳动合同", "service": "服务合同", "sales": "买卖合同", "lease": "房屋租赁合同"}[args.kind], "blocks": blocks, "masked": False}

    script_dir = Path(__file__).resolve().parent
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=1)
        data_path = f.name

    try:
        layout = [sys.executable, str(script_dir / "format_legal.py"), data_path, "-o", args.out]
        if args.pdf:
            layout.append("--pdf")
        r = subprocess.run(layout, capture_output=True)
        if r.returncode != 0:
            sys.exit(r.stderr.decode("utf-8", "replace"))
    finally:
        Path(data_path).unlink(missing_ok=True)

    print(f"[生成] {args.out}（{args.kind} 合同骨架，必备条款齐全）")
    for r in risks:
        if r:
            print(f"[风险] {r}")
    print("[提醒] 骨架 ≠ 成品：填齐 [待补充] 与 __ 空格，跑 check_legal.py 体检，重大交易请律师审。本工具不构成法律意见。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
