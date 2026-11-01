//! `Duckling.Time.ZH.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::computed::*;
use crate::dims::time::helpers::*;
use crate::dims::time::holiday_helpers::compute_earth_hour;
use crate::dims::time::types::{
    Form, TimeData,
    TimeIntervalType::Open,
};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower, to_upper};
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn td(t: &Token) -> Option<&TimeData> {
    match t {
        Token::Time(d) => Some(d),
        _ => None,
    }
}

fn rx(t: &Token, i: usize) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }
}

fn ttd(o: Option<TimeData>) -> Option<Token> {
    o.map(Token::Time)
}

fn grain_of(t: &Token) -> Option<G> {
    match t {
        Token::TimeGrain(g) => Some(*g),
        _ => None,
    }
}

fn dur(t: &Token) -> Option<&DurationData> {
    match t {
        Token::Duration(d) => Some(d),
        _ => None,
    }
}

fn ordinal_value(t: &Token) -> Option<i64> {
    match t {
        Token::Ordinal(o) => Some(o.value),
        _ => None,
    }
}

fn tod_hours(t: &Token) -> Option<i64> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), .. }), .. }) => Some(*h),
        _ => None,
    }
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

const TZ: &str = "\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b";

fn computed_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("耶稣升天节", "耶(稣|穌)升天(节|節|日)", cycle_nth_after(false, G::Day, 39, &easter_sunday())),
        ("大斋首日", "大(斋|齋)首日|(圣|聖)灰((礼仪|禮儀)?日|星期三)|灰日", cycle_nth_after(false, G::Day, -46, &easter_sunday())),
        ("阿舒拉节", "阿舒拉(节|節)", cycle_nth_after(false, G::Day, 9, &muharram())),
        ("克哈特普迦节", "克哈特普迦(节|節)", cycle_nth_after(false, G::Day, 8, &dhanteras())),
        ("春节", "春(节|節)|(农历|農曆|唐人)新年|新(正|春)|正月(正(时|時)|朔日)|岁首", chinese_new_year()),
        ("基督圣体节", "基督(圣体|聖體)((圣|聖)血)?((节|節)|瞻(礼|禮))", cycle_nth_after(false, G::Day, 60, &easter_sunday())),
        ("排灯节", "(排|万|萬|印度)(灯节|燈節)", cycle_nth_after(false, G::Day, 2, &dhanteras())),
        ("复活节星期一", "(复|復)活(节|節)星期一", cycle_nth_after(false, G::Day, 1, &easter_sunday())),
        ("复活节", "(复|復)活(节|節)|主(复|復)活日", easter_sunday()),
        ("古尔邦节", "古(尔|爾)邦(节|節)|宰牲(节|節)", eidal_adha()),
        ("开斋节", "(开斋|開齋|肉孜|(尔|爾)代)(节|節)", eidal_fitr()),
        ("耶稣受难日", "(耶(稣|穌)|主)受(难|難)(节|節|日)|(圣|聖|沈默)(周|週)五", cycle_nth_after(false, G::Day, -2, &easter_sunday())),
        ("侯丽节", "((侯|荷)(丽|麗)|洒红|灑紅|欢悦|歡悅|五彩|胡里|好利|霍利)(节|節)", cycle_nth_after(false, G::Day, 39, &vasant_panchami())),
        ("圣周六", "神?(圣周|聖週)六|(耶(稣|穌)|主)受(难|難)(节|節|日)翌日|(复|復)活(节|節)前夜|黑色星期六", cycle_nth_after(false, G::Day, -1, &easter_sunday())),
        ("伊斯兰新年", "伊斯兰(教)?(历)?新年", muharram()),
        ("登霄节", "(夜行)?登霄(节|節)", cycle_nth_after(false, G::Day, 26, &rajab())),
        ("印度丰收节第四天", "(印度(丰|豐)收|(庞|龐)格(尔|爾))(节|節)第四天", cycle_nth_after(false, G::Day, 2, &thai_pongal())),
        ("篝火节", "((犹|猶)太教)?篝火(节|節)", lag_ba_omer()),
        ("法令之夜", "(法令|命运|权力)之夜", cycle_nth_after(false, G::Day, 26, &ramadan())),
        ("拉撒路圣周六", "拉撒路(圣周|聖週)六|拉匝路(周|週)六", cycle_nth_after(false, G::Day, -8, &orthodox_easter())),
        ("印度丰收节第三天", "(印度(丰|豐)收|(庞|龐)格(尔|爾))(节|節)第三天", cycle_nth_after(false, G::Day, 1, &thai_pongal())),
        ("神圣星期四", "濯足(节|節)|神(圣|聖)星期四|(圣周|聖週)(星期)?四|(设|設)立(圣|聖)餐日", cycle_nth_after(false, G::Day, -3, &easter_sunday())),
        ("圣纪节", "圣纪节|聖紀節", mawlid()),
        ("东正教复活节星期一", "(东|東)正教(复|復)活(节|節)星期一", cycle_nth_after(false, G::Day, 1, &orthodox_easter())),
        ("东正教复活节", "(东|東)正教((复|復)活(节|節)|主(复|復)活日)", orthodox_easter()),
        ("东正教圣周六", "(东|東)正教(神?(圣周|聖週)六|(耶(稣|穌)|主)受(难|難)(节|節|日)翌日|(复|復)活(节|節)前夜)", cycle_nth_after(false, G::Day, -1, &orthodox_easter())),
        ("东正教耶稣受难日", "(东|東)正教((耶(稣|穌)|主)受(难|難)(节|節|日)|(圣|聖|沈默)(周|週)五)", cycle_nth_after(false, G::Day, -2, &orthodox_easter())),
        ("东正教棕枝主日", "(东|東)正教((棕|圣|聖)枝|圣树|聖樹|基督苦(难|難))主日", cycle_nth_after(false, G::Day, -7, &orthodox_easter())),
        ("棕枝主日", "((棕|圣|聖)枝|圣树|聖樹|基督苦(难|難))主日", cycle_nth_after(false, G::Day, -7, &easter_sunday())),
        ("五旬节", "五旬(节|節)|(圣灵|聖靈)降(临|臨)(日|节|節)", cycle_nth_after(false, G::Day, 49, &easter_sunday())),
        ("印度兄妹节", "(印度兄妹|拉克沙班丹)(节|節)", raksha_bandhan()),
        ("圣会节", "(圣|聖)会(节|節)", cycle_nth_after(false, G::Day, 21, &rosh_hashana())),
        ("忏悔节", "忏悔(节|節|火曜日)|煎(饼|餅)星期二", cycle_nth_after(false, G::Day, -47, &easter_sunday())),
        ("西赫托拉节", "(西赫(托拉|妥拉)|诵经|誦經|转经|轉經|律法|(欢庆圣|歡慶聖)法)(节|節)", cycle_nth_after(false, G::Day, 22, &rosh_hashana())),
        ("印度丰收节", "(印度|淡米(尔|爾))(丰|豐)收(节|節)", thai_pongal()),
        ("欧南节", "欧南(节|節)", thiru_onam()),
        ("圣殿被毁日", "((圣|聖)殿被毁|禁食)日", tisha_b_av()),
        ("圣三一主日", "((天主)?(圣|聖)?三一|(圣|聖)三)(主日|节|節)", cycle_nth_after(false, G::Day, 56, &easter_sunday())),
        ("十胜节", "(十(胜|勝)|(凯|凱)旋|(圣|聖)母)(节|節)", cycle_nth_after(false, G::Day, 9, &navaratri())),
        ("犹太植树节", "((犹|猶)太植(树|樹)|(图|圖)比舍巴特)(节|節)|(树|樹)木新年", tu_bishvat()),
        ("卫塞节", "((卫|衛)塞|威瑟|比(萨宝|薩寶)蕉)(节|節)", vesak()),
        ("以色列独立日", "以色列((独|獨)立日|国庆节|國慶節)", yom_haatzmaut()),
        ("赎罪日", "(赎|贖)罪日", cycle_nth_after(false, G::Day, 9, &rosh_hashana())),
        ("圣灵节庆日", "(圣灵节庆|聖靈節慶)日", cycle_nth_after(false, G::Day, 50, &easter_sunday())),
        ("老板节", "老(板节|闆節)", pred_nth_closest(0, &weekday(), &month_day(10, 16))),
    ])
}

fn computed_holidays_prime() -> Vec<Rule> {
    mk_rule_holidays_maybe(&[
        ("全球青年服务日", "全球青年服(务|務)日", {
            let start = global_youth_service_day();
            let end = cycle_nth_after(false, G::Day, 2, &global_youth_service_day());
            interval(Open, &start, &end)
        }),
        ("四旬节", "四旬(节|節)", {
            let start = cycle_nth_after(false, G::Day, -48, &orthodox_easter());
            let end = cycle_nth_after(false, G::Day, -9, &orthodox_easter());
            interval(Open, &start, &end)
        }),
        ("光明节", "(光明|修殿|(献|獻)殿|(烛|燭)光|哈努卡|(马|馬)加比)(节|節)", {
            let start = chanukah();
            let end = cycle_nth_after(false, G::Day, 7, &chanukah());
            interval(Open, &start, &end)
        }),
        ("大斋期", "大(斋|齋)(期|节|節)", {
            let start = cycle_nth_after(false, G::Day, -46, &easter_sunday());
            let end = cycle_nth_after(false, G::Day, -1, &easter_sunday());
            interval(Open, &start, &end)
        }),
        ("九夜节", "(九夜|(难|難)近母)(节|節)", {
            let start = navaratri();
            let end = cycle_nth_after(false, G::Day, 9, &navaratri());
            interval(Open, &start, &end)
        }),
        ("逾越节", "逾越(节|節)", {
            let start = passover();
            let end = cycle_nth_after(false, G::Day, 8, &passover());
            interval(Open, &start, &end)
        }),
        ("斋月", "(穆斯林)?(斋|齋)月", {
            let start = ramadan();
            let end = cycle_nth_after(false, G::Day, -1, &eidal_fitr());
            interval(Open, &start, &end)
        }),
        ("犹太新年", "(犹|猶)太新年", {
            let start = rosh_hashana();
            let end = cycle_nth_after(false, G::Day, 2, &rosh_hashana());
            interval(Open, &start, &end)
        }),
        ("七七节", "(七七|沙夫幼特|(周|週)日|收(获|穫)|新果(实|實))(节|節)", {
            let start = cycle_nth_after(false, G::Day, 50, &passover());
            let end = cycle_nth_after(false, G::Day, 52, &passover());
            interval(Open, &start, &end)
        }),
        ("住棚节", "住棚(节|節)", {
            let start = cycle_nth_after(false, G::Day, 14, &rosh_hashana());
            let end = cycle_nth_after(false, G::Day, 22, &rosh_hashana());
            interval(Open, &start, &end)
        }),
        ("地球一小时", "地球一小(时|時)", compute_earth_hour()),
    ])
}

fn periodic_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("中国共产党的生日", "中(国共产党诞|國共產黨誕)生日|(党|黨)的生日", month_day(7, 1)),
        ("愚人节", "愚人(节|節)", month_day(4, 1)),
        ("建军节", "(中国人民解放(军|軍)|八一)?建(军节|軍節)", month_day(8, 1)),
        ("植树节", "中(国植树节|國植樹節)", month_day(3, 12)),
        ("五四青年节", "(中(国|國))?(五四|54)?青年(节|節)", month_day(5, 4)),
        ("圣诞节", "(圣诞|聖誕)(节|節)?", month_day(12, 25)),
        ("平安夜", "(平安|聖誕)夜", month_day(12, 24)),
        ("哥伦布日", "哥(伦|倫)布日", month_day(10, 12)),
        ("双十一", "(双|雙)(十一|11)", month_day(11, 11)),
        ("万圣节", "万圣节|萬聖節", month_day(10, 31)),
        ("香港回归纪念日", "香港回(归纪|歸紀)念日", month_day(7, 1)),
        ("人权日", "人(权|權)日", month_day(12, 10)),
        ("美国独立日", "(美国)?(独|獨)立日", month_day(7, 4)),
        ("儿童节", "(国际|國際)?(六一|61)?(儿|兒)童(节|節)", month_day(6, 1)),
        ("国际慈善日", "(国际|國際)慈善日", month_day(9, 5)),
        ("国际瑜伽日", "(国际|國際)瑜伽日", month_day(6, 21)),
        ("国际爵士日", "(国际|國際)爵士日", month_day(4, 30)),
        ("国际奥林匹克日", "(国际|國際)奥林匹克日", month_day(6, 23)),
        ("妇女节", "(国际劳动|國際勞動|三八)?(妇|婦)女(节|節)", month_day(3, 8)),
        ("劳动节", "(五一|51)?(国际|國際)?(劳动|勞動)(节|節)", month_day(5, 1)),
        ("国际青年节", "(国际|國際)青年(节|節)", month_day(8, 12)),
        ("澳门回归纪念日", "澳(门|門)回(归纪|歸紀)念日", month_day(12, 20)),
        ("全国爱牙日", "全(国爱|國愛)牙日", month_day(9, 20)),
        ("全国爱耳日", "全(国爱|國愛)耳日", month_day(3, 3)),
        ("全国爱眼日", "全(国爱|國愛)眼日", month_day(6, 6)),
        ("南京大屠杀纪念日", "南京大屠(杀纪|殺紀)念日", month_day(12, 13)),
        ("辛亥革命纪念日", "辛亥革命(纪|紀)念日", month_day(10, 10)),
        ("元旦", "元旦(节|節)?|((公|(阳|陽))(历|曆))?新年", month_day(1, 1)),
        ("新年夜", "新年夜", month_day(12, 31)),
        ("情人节", "(情人|(圣瓦伦丁|聖瓦倫丁))(节|節)", month_day(2, 14)),
        ("清明节", "清明(节|節)", month_day(4, 5)),
        ("光棍节", "光棍(节|節)", month_day(11, 11)),
        ("圣帕特里克节", "圣帕特里克节|聖帕特里克節", month_day(3, 17)),
        ("教师节", "(中(国|國))?教师(节|節)", month_day(9, 10)),
        ("退伍军人节", "(退伍(军|軍)人|老兵)(节|節)", month_day(11, 11)),
        ("白色情人节", "白色情人(节|節)", month_day(3, 14)),
        ("世界艾滋病日", "世界艾滋病日", month_day(12, 1)),
        ("世界献血日", "世界(献|獻)血日", month_day(6, 14)),
        ("世界癌症日", "世界癌(症|癥)日", month_day(2, 4)),
        ("国际消费者权益日", "(国际|世界)?(消费者权益|消費者權益)日|三一五", month_day(3, 15)),
        ("世界糖尿病日", "世界糖尿病日", month_day(11, 14)),
        ("世界环境日", "世界(环|環)境日", month_day(6, 5)),
        ("世界粮食日", "世界((粮|糧)食|食物)日", month_day(10, 16)),
        ("世界心脏日", "世界心(脏|臟)日", month_day(9, 29)),
        ("世界海洋日", "世界海洋日", month_day(6, 8)),
        ("世界诗歌日", "世界(诗|詩)歌日", month_day(3, 21)),
        ("世界人口日", "世界人口日", month_day(7, 11)),
        ("世界难民日", "世界(难|難)民日", month_day(6, 20)),
        ("世界教师日", "世界教师日", month_day(10, 5)),
        ("世界旅游日", "世界旅游日", month_day(9, 27)),
        ("父亲节", "父(亲节|親節)", nth_dow_of_month(3, 7, 6)),
        ("马丁路德金日", "(马|馬)丁路德金((纪|紀)念)?日", nth_dow_of_month(3, 1, 1)),
        ("母亲节", "母(亲节|親節)", nth_dow_of_month(2, 7, 5)),
    ])
}

fn rules_main() -> Vec<Rule> {
    vec![
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
            ts.first().map(|t| (*t).clone())
        }),
        rule("afternoon", vec![regex("下午|中午|\u{664f}\u{665d}")], |_| {
            ttd(interval(Open, &hour(false, 12), &hour(false, 19)).map(part_of_day).map(mk_latent))
        }),
        rule("<dim time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule("<duration> ago", vec![dimension(Dim::Duration), regex("(之)?前")], |ts| {
            tt(duration_ago(dur(ts[0])?))
        }),
        rule("<duration> from now", vec![dimension(Dim::Duration), regex("后|後|之後")], |ts| {
            tt(in_duration(dur(ts[0])?))
        }),
        rule("evening|night", vec![regex("晚上|晚间")], |_| {
            ttd(interval(Open, &hour(false, 18), &hour(false, 0)).map(mk_latent).map(part_of_day))
        }),
        rule(
            "hhmm (military time-of-day)",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(mk_latent(hour_minute(false, h, m)))
            },
        ),
        rule("hh:mm (time-of-day)", vec![regex("((?:[01]?\\d)|(?:2[0-3])):([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(true, h, m))
        }),
        rule("in <duration>", vec![regex("再"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        rule("in|during the <part-of-day>", vec![pred(is_a_part_of_day), regex("点|點")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule(
            "<integer> (latent time-of-day)",
            vec![pred(is_integer_between_tok(0, 23))],
            |ts| tt(mk_latent(hour(true, get_int_value(ts[0])?))),
        ),
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "intersect by \",\"",
            vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        rule("last <cycle>", vec![regex("上(个|個)?"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, -1))
        }),
        rule(
            "last n <cycle>",
            vec![regex("上|前"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -v))
            },
        ),
        rule(
            "n <cycle> last",
            vec![pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain), regex("(之)?前")],
            |ts| {
                let v = get_int_value(ts[0])?;
                tt(cycle_n(true, grain_of(ts[1])?, -v))
            },
        ),
        rule("last night", vec![regex("昨晚|昨天晚上|\u{5c0b}晚")], |_| {
            let td1 = cycle_nth(G::Day, -1);
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&td1, &td2).map(part_of_day))
        }),
        rule("last <time>", vec![regex("去|上(个|個)?"), pred(is_ok_with_this_next)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        rule("last year", vec![regex("去年|上年")], |_| tt(cycle_nth(G::Year, -1))),
        rule("midnight", vec![regex("午夜|凌晨|半夜")], |_| {
            tt(mk_ok_for_this_next(hour(false, 0)))
        }),
        rule("mm/dd", vec![regex("(0?[1-9]|1[0-2])/(3[01]|[12]\\d|0?[1-9])")], |ts| {
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        rule(
            "mm/dd/yyyy",
            vec![regex("(0?[1-9]|1[0-2])/(3[01]|[12]\\d|0?[1-9])/(\\d{2,4})")],
            |ts| {
                let m = parse_int(&rx(ts[0], 0)?)?;
                let d = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        rule("morning", vec![regex("早上|早晨|\u{671d}(\u{982d})?早")], |_| {
            ttd(interval(Open, &hour(false, 4), &hour(false, 12)).map(part_of_day).map(mk_latent))
        }),
        rule(
            "<named-month> <day-of-month>",
            vec![pred(is_a_month), dimension(Dim::Numeral), regex("号|號|日")],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        rule("next <cycle>", vec![regex("下(个|個)?"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 1))
        }),
        rule(
            "next n <cycle>",
            vec![regex("下|后|後"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, v))
            },
        ),
        rule(
            "next n <cycle>",
            vec![
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
                regex("下|(之)?后|(之)?後"),
            ],
            |ts| {
                let v = get_int_value(ts[0])?;
                tt(cycle_n(true, grain_of(ts[1])?, v))
            },
        ),
        rule("next <time>", vec![regex("明|下(个|個)?"), pred(is_ok_with_this_next)], |ts| {
            tt(pred_nth(1, false, td(ts[1])?))
        }),
        rule("next year", vec![regex("明年|下年")], |_| tt(cycle_nth(G::Year, 1))),
        rule("noon", vec![regex("中午")], |_| tt(mk_ok_for_this_next(hour(false, 12)))),
        rule("now", vec![regex("现在|此时|此刻|当前|現在|此時|當前|\u{5b9c}\u{5bb6}|\u{800c}\u{5bb6}|\u{4f9d}\u{5bb6}")], |_| {
            tt(now())
        }),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), dimension(Dim::Time)],
            |ts| {
                let n = ordinal_value(ts[1])?;
                ttd(intersect(td(ts[2])?, td(ts[0])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Time), regex("的"), dimension(Dim::Ordinal), dimension(Dim::Time)],
            |ts| {
                let n = ordinal_value(ts[2])?;
                ttd(intersect(td(ts[3])?, td(ts[0])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        rule("<part-of-day> <dim time>", vec![pred(is_a_part_of_day), dimension(Dim::Time)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "relative (10-59) minutes after|past <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("点|點"), pred(is_integer_between_tok(10, 59))],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, get_int_value(ts[2])?)),
        ),
        rule(
            "relative minutes after|past noon|midnight",
            vec![pred(is_midnight_or_noon), regex("过|\u{904e}"), pred(is_integer_between_tok(1, 59))],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, get_int_value(ts[2])?)),
        ),
        rule(
            "relative minutes to|till|before <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("(点|點)差"), pred(is_integer_between_tok(1, 59))],
            |ts| ttd(minutes_before(get_int_value(ts[2])?, td(ts[0])?)),
        ),
        rule(
            "relative minutes to|till|before noon|midnight",
            vec![pred(is_midnight_or_noon), regex("差"), pred(is_integer_between_tok(1, 59))],
            |ts| ttd(minutes_before(get_int_value(ts[2])?, td(ts[0])?)),
        ),
        rule(
            "quarter after|past <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("点|點"), regex("一刻")],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 15)),
        ),
        rule(
            "quarter after|past noon|midnight",
            vec![pred(is_midnight_or_noon), regex("过"), regex("一刻")],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 15)),
        ),
        rule(
            "quarter to|till|before <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("(点|點)差"), regex("一刻")],
            |ts| ttd(minutes_before(15, td(ts[0])?)),
        ),
        rule(
            "quarter to|till|before noon|midnight",
            vec![pred(is_midnight_or_noon), regex("差"), regex("一刻")],
            |ts| ttd(minutes_before(15, td(ts[0])?)),
        ),
        rule(
            "half after|past <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("(点|點)(踏|搭)?"), regex("半")],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 30)),
        ),
        rule(
            "half after|past noon|midnight",
            vec![pred(is_midnight_or_noon), regex("过"), regex("半")],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 30)),
        ),
        rule(
            "half to|till|before <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("(点|點)差"), regex("半")],
            |ts| ttd(minutes_before(30, td(ts[0])?)),
        ),
        rule(
            "half to|till|before noon|midnight",
            vec![pred(is_midnight_or_noon), regex("差"), regex("半")],
            |ts| ttd(minutes_before(30, td(ts[0])?)),
        ),
        rule(
            "the <cycle> after <time>",
            vec![regex("那"), dimension(Dim::TimeGrain), regex("(之)?(后|後)"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
        ),
        rule(
            "the <cycle> before <time>",
            vec![regex("那"), dimension(Dim::TimeGrain), regex("(之)?前"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
        ),
        rule("the day after tomorrow", vec![regex("后天|後天|後日")], |_| tt(cycle_nth(G::Day, 2))),
        rule("two days after tomorrow", vec![regex("大后天|大後天|大後日")], |_| {
            tt(cycle_nth(G::Day, 3))
        }),
        rule("the day before yesterday", vec![regex("前天|前日")], |_| tt(cycle_nth(G::Day, -2))),
        rule("this <cycle>", vec![regex("(这|這)(一)?|今個"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 0))
        }),
        rule("this <day-of-week>", vec![regex("这|這|今(个|個)?"), pred(is_a_day_of_week)], |ts| {
            tt(pred_nth(0, false, td(ts[1])?))
        }),
        rule("this <time>", vec![regex("今(个|個)?|这(个)?|這(個)?"), pred(is_ok_with_this_next)], |ts| {
            tt(pred_nth(0, false, td(ts[1])?))
        }),
        rule("this year", vec![regex("今年")], |_| tt(cycle_nth(G::Year, 0))),
        rule("next <day-of-week>", vec![regex("明|下(个|個)?"), pred(is_a_day_of_week)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        rule(
            "<time-of-day> am|pm",
            vec![pred(is_a_time_of_day), regex("([ap])(\\s|\\.)?m?\\.?")],
            |ts| tt(time_of_day_ampm(&to_lower(&rx(ts[1], 0)?) == "a", td(ts[0])?)),
        ),
        rule("<time-of-day> o'clock", vec![pred(is_a_time_of_day), regex("點|点|時")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule("today", vec![regex("今天|今日")], |_| tt(today())),
        rule("tomorrow", vec![regex("明天|明日|\u{807d}日")], |_| tt(cycle_nth(G::Day, 1))),
        rule("tomorrow night", vec![regex("明晚|明天晚上|\u{807d}晚")], |_| {
            let td1 = cycle_nth(G::Day, 1);
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&td1, &td2).map(part_of_day))
        }),
        rule("tonight", vec![regex("今晚|今天晚上")], |_| {
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        rule("week-end", vec![regex("周末|週末")], |_| tt(mk_ok_for_this_next(weekend()))),
        rule(
            "year (numeric with year symbol)",
            vec![pred(is_integer_between_tok(1000, 9999)), regex("年")],
            |ts| tt(year(get_int_value(ts[0])?)),
        ),
        rule(
            "xxxx year",
            vec![
                dimension(Dim::Numeral),
                dimension(Dim::Numeral),
                dimension(Dim::Numeral),
                dimension(Dim::Numeral),
                regex("年"),
            ],
            |ts| match (ts[0], ts[1], ts[2], ts[3]) {
                (Token::Numeral(y1), Token::Numeral(y2), Token::Numeral(y3), Token::Numeral(y4)) => {
                    let v = (y1.value * 1000.0 + y2.value * 100.0 + y3.value * 10.0 + y4.value).floor() as i64;
                    tt(year(v))
                }
                _ => None,
            },
        ),
        rule("yesterday", vec![regex("昨天|昨日|\u{5c0b}日")], |_| tt(cycle_nth(G::Day, -1))),
        rule(
            "yyyy-mm-dd",
            vec![regex("(\\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\\d|0?[1-9])")],
            |ts| {
                let y = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let d = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        rule("<time> timezone", vec![pred(not_latent_tod), regex(TZ)], |ts| {
            ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?))
        }),
        rule(
            "relative (10-59) minutes after|past <integer> (hour-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("点|點"),
                pred(is_integer_between_tok(10, 59)),
                regex("分"),
            ],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, get_int_value(ts[2])?)),
        ),
        rule(
            "relative (1-9) minutes after|past <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("点零|點零"), pred(is_integer_between_tok(1, 9))],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, get_int_value(ts[2])?)),
        ),
        rule(
            "relative (1-9) minutes after|past <integer> (hour-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("点零|點零"),
                pred(is_integer_between_tok(1, 9)),
                regex("分"),
            ],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, get_int_value(ts[2])?)),
        ),
        rule(
            "number of 5 minutes after|past <integer> (hour-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("点踏|點踏|点搭|點搭"),
                pred(is_integer_between_tok(1, 11)),
            ],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 5 * get_int_value(ts[2])?)),
        ),
        rule(
            "number of 5 minutes after|past <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("点|點"), pred(is_integer_between_tok(1, 9))],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 5 * get_int_value(ts[2])?)),
        ),
        rule(
            "number of 5 minutes after|past <integer> (hour-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("点|點"),
                pred(is_integer_between_tok(1, 11)),
                regex("個字"),
            ],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 5 * get_int_value(ts[2])?)),
        ),
        rule("last <duration>", vec![regex("上(个|個)?"), dimension(Dim::Duration)], |ts| {
            tt(duration_ago(dur(ts[1])?))
        }),
        rule("<time> <day-of-month>", vec![pred(is_integer_between_tok(1, 31)), regex("号|號|日")], |ts| {
            tt(day_of_month(get_int_value(ts[0])?))
        }),
    ]
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = rules_main();
    v.extend(mk_rule_days_of_week(&[
        ("Monday", "星期一|周一|礼拜一|禮拜一|週一"),
        ("Tuesday", "星期二|周二|礼拜二|禮拜二|週二"),
        ("Wednesday", "星期三|周三|礼拜三|禮拜三|週三"),
        ("Thursday", "星期四|周四|礼拜四|禮拜四|週四"),
        ("Friday", "星期五|周五|礼拜五|禮拜五|週五"),
        ("Saturday", "星期六|周六|礼拜六|禮拜六|週六"),
        ("Sunday", "星期日|星期天|礼拜天|周日|禮拜天|週日|禮拜日"),
    ]));
    v.extend(mk_rule_months(&[
        ("January", "(一|1)月(份)?"),
        ("February", "(二|2)月(份)?"),
        ("March", "(三|3)月(份)?"),
        ("April", "(四|4)月(份)?"),
        ("May", "(五|5)月(份)?"),
        ("June", "(六|6)月(份)?"),
        ("July", "(七|7)月(份)?"),
        ("August", "(八|8)月(份)?"),
        ("September", "(九|9)月(份)?"),
        ("October", "(十|10)月(份)?"),
        ("November", "(十一|11)月(份)?"),
        ("December", "(十二|12)月(份)?"),
    ]));
    v.extend(computed_holidays());
    v.extend(computed_holidays_prime());
    v.extend(periodic_holidays());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
