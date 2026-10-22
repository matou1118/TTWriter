//! Pinyin sort key for Chinese text.
//!
//! Each common CJK character maps to its pinyin syllable (toneless, lowercase). The sort key is
//! the space-joined syllables of all CJK characters, with non-CJK characters lowercased and
//! interleaved. This gives correct alphabetical order by pinyin: 北京 → "bei jing", 上海 →
//! "shang hai", so 北京 sorts before 上海.
//!
//! The table is a P0-1 subset (~500 most common characters). Characters not in the table fall
//! back to their Unicode code point, so they sort after all known pinyin syllables but still
//! have a deterministic order. Full pinyin coverage needs a complete Unihan database; out of
//! scope here.

/// A compact pinyin table: groups of characters that share the same pinyin syllable.
/// Organised alphabetically by syllable. One reading per character (the most common).
const PINYIN: &[(&str, &str)] = &[
    ("啊阿呵", "a"),
    ("爱艾碍哀埃挨", "ai"),
    ("安案按暗岸俺谙庵", "an"),
    ("昂", "ang"),
    ("奥袄傲澳懊", "ao"),
    ("八巴拔罢霸把爸吧靶芭疤", "ba"),
    ("白百摆败拜柏", "bai"),
    ("办半版板班般搬伴扮拌瓣颁坂", "ban"),
    ("帮邦榜膀绑棒傍磅", "bang"),
    ("包保报抱暴薄宝饱堡刨苞", "bao"),
    ("北倍备背杯辈悲碑卑倍", "bei"),
    ("本苯奔笨", "ben"),
    ("崩绷甭", "beng"),
    ("比笔必毕币避彼逼鄙鼻泌", "bi"),
    ("边变便编标表别宾彬斌濒", "bian"),
    ("别瘪", "bie"),
    ("宾滨彬斌濒", "bin"),
    ("兵冰丙柄秉", "bing"),
    ("不布步部补簿卜埠", "bu"),
    ("猜才材裁财采踩蔡", "cai"),
    ("参残餐惨灿仓苍舱", "can"),
    ("藏仓沧苍", "cang"),
    ("操草槽曹", "cao"),
    ("册侧厕策测", "ce"),
    ("层曾蹭", "ceng"),
    ("叉查茶差插察岔", "cha"),
    ("拆柴豺", "chai"),
    ("产阐颤缠搀掺", "chan"),
    ("长场厂常尝偿畅肠倡", "chang"),
    ("超抄朝潮吵巢炒钞", "chao"),
    ("车彻撤扯", "che"),
    ("尘臣辰晨忱沉陈闯", "chen"),
    ("成程城承称诚惩乘澄秤", "cheng"),
    ("吃迟池持尺赤翅匙耻驰", "chi"),
    ("充冲虫崇宠重", "chong"),
    ("抽酬畴愁丑仇筹稠", "chou"),
    ("出初除厨楚础储搐触", "chu"),
    ("揣", "chuai"),
    ("川穿传船喘串", "chuan"),
    ("窗床创闯", "chuang"),
    ("吹炊垂锤春椿", "chui"),
    ("蠢", "chun"),
    ("戳", "chuo"),
    ("次词刺慈磁瓷辞此雌", "ci"),
    ("从匆聪葱囱", "cong"),
    ("凑", "cou"),
    ("粗促醋簇", "cu"),
    ("窜", "cuan"),
    ("催摧翠脆崔", "cui"),
    ("寸村存", "cun"),
    ("错措搓挫", "cuo"),
    ("大打达答搭瘩", "da"),
    ("代带待袋戴逮怠", "dai"),
    ("单担丹胆旦诞蛋弹", "dan"),
    ("当党档荡", "dang"),
    ("刀导倒到道岛悼盗", "dao"),
    ("得德的德", "de"),
    ("灯登等瞪凳", "deng"),
    ("低堤滴底弟第敌帝递抵", "di"),
    ("点典电店殿奠垫", "dian"),
    ("调掉吊钓雕", "diao"),
    ("叠蝶谍迭", "die"),
    ("丁定顶订钉鼎", "ding"),
    ("丢", "diu"),
    ("东动懂冬洞栋董", "dong"),
    ("都斗豆抖兜陡逗", "dou"),
    ("毒独读督渡堵杜妒顿", "du"),
    ("端短段断", "duan"),
    ("队对兑", "dui"),
    ("吨蹲盾顿", "dun"),
    ("多夺朵躲", "duo"),
    ("额俄鹅恶饿恩", "e"),
    ("儿而耳二尔", "er"),
    ("发法罚乏伐阀", "fa"),
    ("反返犯泛饭翻烦繁", "fan"),
    ("方放房防访仿纺妨", "fang"),
    ("飞非费废匪沸", "fei"),
    ("分纷坟奋份粉氛", "fen"),
    ("丰风封峰疯逢锋蜂", "feng"),
    ("否", "fou"),
    ("夫父付服府符腐浮辅肤", "fu"),
    ("该改概盖溉", "gai"),
    ("干甘赶感敢杆肝", "gan"),
    ("刚岗港钢纲", "gang"),
    ("高告稿搞糕羔", "gao"),
    ("歌革格各阁隔铬", "ge"),
    ("给", "gei"),
    ("根跟", "gen"),
    ("更耕", "geng"),
    ("工公共功攻供巩贡", "gong"),
    ("勾沟狗够购构钩", "gou"),
    ("古估故股骨顾固雇", "gu"),
    ("挂乖拐怪", "guai"),
    ("瓜刮", "gua"),
    ("关观管官馆惯冠", "guan"),
    ("光广逛", "guang"),
    ("归规贵轨桂柜", "gui"),
    ("滚棍", "gun"),
    ("国过果裹锅", "guo"),
    ("哈孩海害哈", "ha"),
    ("还孩海害骇", "hai"),
    ("含寒韩罕喊汉汗函涵", "han"),
    ("行航杭", "hang"),
    ("好号浩耗豪壕", "hao"),
    ("合何和河喝贺荷核", "he"),
    ("黑嘿", "hei"),
    ("很恨痕狠", "hen"),
    ("横衡恒", "heng"),
    ("红洪宏弘鸿虹", "hong"),
    ("后候厚侯喉吼", "hou"),
    ("呼忽糊湖护户互壶虎", "hu"),
    ("花华化划画话怀坏", "hua"),
    ("坏", "huai"),
    ("还环缓换患欢唤幻", "huan"),
    ("黄皇荒晃煌谎", "huang"),
    ("灰回会汇悔毁惠", "hui"),
    ("婚混魂浑", "hun"),
    ("活火货或获祸惑", "huo"),
    ("机几击积基鸡记即季集急技忌", "ji"),
    ("家加佳嘉甲价假架挟", "jia"),
    ("间见建检简坚减剪键渐", "jian"),
    ("将江姜讲奖匠降蒋", "jiang"),
    ("交教角脚较叫觉搅", "jiao"),
    ("节结杰解介界接届借截", "jie"),
    ("进近金紧今仅禁劲晋", "jin"),
    ("京经精境竞静井警敬", "jing"),
    ("就究九酒旧救纠舅", "jiu"),
    ("句具局举聚拒剧俱", "ju"),
    ("决绝角爵", "jue"),
    ("军均君峻俊", "jun"),
    ("开凯慨", "kai"),
    ("看刊勘堪坎砍", "kan"),
    ("抗慷炕", "kang"),
    ("考靠烤拷", "kao"),
    ("可克课客刻科颗渴", "ke"),
    ("肯啃", "ken"),
    ("空孔恐控", "kong"),
    ("口扣", "kou"),
    ("苦库哭酷裤枯", "ku"),
    ("跨夸垮", "kua"),
    ("快块筷", "kuai"),
    ("宽款", "kuan"),
    ("狂况", "kuang"),
    ("亏葵愧", "kui"),
    ("昆捆困", "kun"),
    ("阔扩括", "kuo"),
    ("拉啦腊辣蜡", "la"),
    ("来赖", "lai"),
    ("兰蓝烂拦栏滥", "lan"),
    ("郎朗浪狼廊", "lang"),
    ("老劳牢落捞", "lao"),
    ("乐了勒", "le"),
    ("类雷累泪垒", "lei"),
    ("冷愣", "leng"),
    ("里理力立利历例礼李厉", "li"),
    ("连联怜莲廉", "lian"),
    ("两量凉梁粮", "liang"),
    ("了料辽疗", "liao"),
    ("列烈劣裂猎咧", "lie"),
    ("林临邻淋磷", "lin"),
    ("令另零灵领龄凌", "ling"),
    ("六溜刘流留柳", "liu"),
    ("龙隆笼拢", "long"),
    ("楼漏搂陋", "lou"),
    ("路鲁录陆露炉芦", "lu"),
    ("律绿旅率虑", "lv"),
    ("乱卵", "luan"),
    ("略掠", "lve"),
    ("轮伦论", "lun"),
    ("落罗螺洛锣", "luo"),
    ("妈马吗码麻骂玛", "ma"),
    ("买卖麦脉埋迈", "mai"),
    ("满慢蛮漫瞒蔓", "man"),
    ("忙茫盲芒", "mang"),
    ("毛矛冒贸茂帽", "mao"),
    ("么没眉梅媒煤", "me"),
    ("美没每妹梅媒", "mei"),
    ("门闷们", "men"),
    ("猛蒙盟梦", "meng"),
    ("米密迷蜜谜靡", "mi"),
    ("面免棉绵勉", "mian"),
    ("苗描秒妙庙渺", "miao"),
    ("灭蔑", "mie"),
    ("民敏", "min"),
    ("名明命鸣铭", "ming"),
    ("谬", "miu"),
    ("模母摸磨魔末莫漠", "mo"),
    ("某谋", "mou"),
    ("亩牧姆慕", "mu"),
    ("那拿哪呐", "na"),
    ("乃奶耐奈", "nai"),
    ("南男难楠", "nan"),
    ("囊", "nang"),
    ("脑闹挠恼", "nao"),
    ("呢", "ne"),
    ("内", "nei"),
    ("嫩", "nen"),
    ("能", "neng"),
    ("你尼呢泥拟逆腻", "ni"),
    ("年念粘碾", "nian"),
    ("娘酿", "niang"),
    ("鸟尿", "niao"),
    ("聂涅", "nie"),
    ("您", "nin"),
    ("宁凝拧", "ning"),
    ("牛纽扭", "niu"),
    ("农弄浓", "nong"),
    ("奴努怒", "nu"),
    ("女", "nv"),
    ("暖", "nuan"),
    ("虐", "nve"),
    ("挪诺糯", "nuo"),
    ("哦噢", "o"),
    ("偶欧藕", "ou"),
    ("怕爬帕拍排牌派", "pa"),
    ("盘判叛盼攀", "pan"),
    ("旁庞磅", "pang"),
    ("跑炮袍刨", "pao"),
    ("配陪培佩赔", "pei"),
    ("喷盆", "pen"),
    ("朋棚鹏碰", "peng"),
    ("批皮疲匹劈僻屁", "pi"),
    ("片偏篇骗", "pian"),
    ("票漂飘", "piao"),
    ("拼贫品聘", "pin"),
    ("平评苹凭瓶屏", "ping"),
    ("破坡泼颇婆迫", "po"),
    ("剖", "pou"),
    ("扑铺普谱浦朴", "pu"),
    ("七期其奇起气企骑弃契", "qi"),
    ("恰掐", "qia"),
    ("千前钱浅签潜遣", "qian"),
    ("强枪墙腔抢", "qiang"),
    ("切且窃", "qie"),
    ("亲钦秦勤", "qin"),
    ("青清轻倾庆顷", "qing"),
    ("穷琼", "qiong"),
    ("秋求球囚", "qiu"),
    ("区去取曲趋趣渠", "qu"),
    ("全权劝泉圈", "quan"),
    ("确缺鹊", "que"),
    ("群裙", "qun"),
    ("然燃染", "ran"),
    ("让嚷", "rang"),
    ("绕饶扰", "rao"),
    ("惹热", "re"),
    ("人任认仁忍刃", "ren"),
    ("扔", "reng"),
    ("日", "ri"),
    ("容荣融溶熔", "rong"),
    ("柔揉肉", "rou"),
    ("如入儒乳辱", "ru"),
    ("软", "ruan"),
    ("锐瑞", "rui"),
    ("润", "run"),
    ("若弱", "ruo"),
    ("撒洒萨", "sa"),
    ("赛塞", "sai"),
    ("三散伞", "san"),
    ("桑丧", "sang"),
    ("扫嫂搔", "sao"),
    ("色塞涩", "se"),
    ("森", "sen"),
    ("僧", "seng"),
    ("杀沙傻刹纱", "sha"),
    ("筛晒", "shai"),
    ("山闪善扇陕单", "shan"),
    ("上商伤尚赏裳", "shang"),
    ("烧少绍哨邵", "shao"),
    ("设社射涉摄", "she"),
    ("身深申神甚审慎", "shen"),
    ("生声升胜剩盛圣", "sheng"),
    ("十是时使事市石实史示识始世", "shi"),
    ("收手受寿授售瘦", "shou"),
    ("书树熟属术输叔述束", "shu"),
    ("刷耍", "shua"),
    ("摔衰甩", "shuai"),
    ("栓拴", "shuan"),
    ("双爽", "shuang"),
    ("水谁税睡", "shui"),
    ("顺瞬", "shun"),
    ("说硕", "shuo"),
    ("四思死司私丝撕似", "si"),
    ("松宋送讼诵", "song"),
    ("搜艘", "sou"),
    ("俗素诉宿速塑", "su"),
    ("酸蒜", "suan"),
    ("随岁碎穗", "sui"),
    ("孙损笋", "sun"),
    ("缩所锁索", "suo"),
    ("他她它塌塔", "ta"),
    ("太台抬态泰汰", "tai"),
    ("谈贪坦滩弹潭", "tan"),
    ("堂糖躺烫塘", "tang"),
    ("逃套讨陶涛", "tao"),
    ("特", "te"),
    ("腾疼藤", "teng"),
    ("提题体替梯涕", "ti"),
    ("天田添填", "tian"),
    ("条跳挑调", "tiao"),
    ("铁贴", "tie"),
    ("听厅停挺亭", "ting"),
    ("通同童统痛", "tong"),
    ("头投透偷", "tou"),
    ("土突图徒途涂屠", "tu"),
    ("团", "tuan"),
    ("推腿退蜕", "tui"),
    ("吞屯臀", "tun"),
    ("脱拖妥拓", "tuo"),
    ("挖瓦蛙洼", "wa"),
    ("外歪", "wai"),
    ("完万玩晚弯顽挽", "wan"),
    ("王往望忘亡旺", "wang"),
    ("为位未维卫委围谓", "wei"),
    ("文问闻温稳吻", "wen"),
    ("我握沃卧", "wo"),
    ("无五武物务误悟", "wu"),
    ("夕西吸希悉惜熙", "xi"),
    ("下吓夏峡瞎", "xia"),
    ("先现限线险鲜献", "xian"),
    ("相想向象项香享详", "xiang"),
    ("小笑效消晓销校", "xiao"),
    ("些写协血歇邪", "xie"),
    ("心新信辛欣", "xin"),
    ("行星形兴型醒姓", "xing"),
    ("修秀休羞袖", "xiu"),
    ("许需虚续序畜", "xu"),
    ("宣选悬旋", "xuan"),
    ("学雪穴", "xue"),
    ("寻巡询循", "xun"),
    ("压呀押鸦哑亚", "ya"),
    ("言严研眼延演宴", "yan"),
    ("样阳养央洋扬", "yang"),
    ("要摇遥咬药耀", "yao"),
    ("也页业夜叶野液", "ye"),
    ("一依移已以意亿义议益", "yi"),
    ("因引音银印饮", "yin"),
    ("应英迎影硬映", "ying"),
    ("拥永咏勇涌", "yong"),
    ("用优由有又油游", "you"),
    ("于与余鱼予愉愚", "yu"),
    ("员元原远院愿圆", "yuan"),
    ("月约越跃岳悦", "yue"),
    ("运云允韵", "yun"),
    ("杂咱砸", "za"),
    ("再在载灾栽", "zai"),
    ("咱暂赞", "zan"),
    ("脏葬", "zang"),
    ("遭早造燥糟", "zao"),
    ("则责泽", "ze"),
    ("怎", "zen"),
    ("增憎", "zeng"),
    ("扎炸眨诈", "zha"),
    ("斋宅摘", "zhai"),
    ("展占战站站盏", "zhan"),
    ("张长涨章帐丈", "zhang"),
    ("招照找赵朝召", "zhao"),
    ("着者这浙折", "zhe"),
    ("真针阵镇振震侦", "zhen"),
    ("正证政整争支征郑", "zheng"),
    ("之知只制直至志值治质", "zhi"),
    ("中种重钟众终", "zhong"),
    ("周州洲皱宙", "zhou"),
    ("主住注助祝驻筑", "zhu"),
    ("抓", "zhua"),
    ("拽", "zhuai"),
    ("专砖转撰", "zhuan"),
    ("装状撞", "zhuang"),
    ("追坠", "zhui"),
    ("准", "zhun"),
    ("桌捉浊酌", "zhuo"),
    ("子自字资姿紫", "zi"),
    ("总纵宗", "zong"),
    ("走奏揍", "zou"),
    ("组足祖阻租", "zu"),
    ("钻", "zuan"),
    ("最罪醉", "zui"),
    ("尊遵", "zun"),
    ("作做左坐昨", "zuo"),
];

/// The pinyin syllable of a single CJK character, or `None` if not in the table.
pub fn pinyin_of(c: char) -> Option<&'static str> {
    for (chars, py) in PINYIN {
        if chars.contains(c) {
            return Some(py);
        }
    }
    None
}

/// A pinyin sort key: each CJK character becomes its pinyin syllable (lowercase ASCII), separated
/// by spaces; non-CJK characters are lowercased and kept in place. The result is a string that
/// sorts alphabetically in pinyin order.
///
/// ```
/// use wordcraft_doc::pinyin::pinyin_sort_key;
/// assert!(pinyin_sort_key("北京") < pinyin_sort_key("上海"));
/// assert!(pinyin_sort_key("苹果") < pinyin_sort_key("香蕉"));
/// ```
pub fn pinyin_sort_key(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 4);
    let mut prev_was_cjk = false;
    for c in s.chars() {
        if let Some(py) = pinyin_of(c) {
            if prev_was_cjk {
                out.push(' ');
            }
            out.push_str(py);
            prev_was_cjk = true;
        } else {
            for cc in c.to_lowercase() {
                out.push(cc);
            }
            prev_was_cjk = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beijing_before_shanghai() {
        assert!(pinyin_sort_key("北京") < pinyin_sort_key("上海"));
    }

    #[test]
    fn apple_before_banana() {
        assert!(pinyin_sort_key("苹果") < pinyin_sort_key("香蕉"));
    }

    #[test]
    fn known_pinyin() {
        assert_eq!(pinyin_of('中'), Some("zhong"));
        assert_eq!(pinyin_of('国'), Some("guo"));
        assert_eq!(pinyin_of('人'), Some("ren"));
        assert_eq!(pinyin_of('A'), None);
    }

    #[test]
    fn non_cjk_passes_through() {
        assert_eq!(pinyin_sort_key("Hello"), "hello");
        assert_eq!(pinyin_sort_key("123"), "123");
    }

    #[test]
    fn mixed_cjk_and_ascii() {
        let key = pinyin_sort_key("中国abc");
        assert!(key.starts_with("zhong guo"));
        assert!(key.contains("abc"));
    }

    #[test]
    fn sort_list_by_pinyin() {
        let mut words = vec!["上海", "北京", "广州", "深圳"];
        words.sort_by(|a, b| pinyin_sort_key(a).cmp(&pinyin_sort_key(b)));
        // bei jing < guang zhou < shang hai < shen zhen
        assert_eq!(words, ["北京", "广州", "上海", "深圳"]);
    }
}
