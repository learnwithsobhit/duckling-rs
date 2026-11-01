//! Holiday tables of `Duckling.Time.DE.Rules` (`ruleHolidays`, `ruleComputedHolidays`, `ruleComputedHolidays'`).
//! Generated from the Haskell tables with `tools/hs_holidays.py` helpers; rule order is identical to upstream.
#![allow(clippy::all, unused_imports)]

use crate::dims::time::computed::*;
use crate::dims::time::helpers::*;
use crate::dims::time::holiday_helpers::*;
use crate::dims::time::manual_holidays::*;
use crate::dims::time::types::TimeIntervalType::Open;
use crate::dims::time_grain::Grain as G;
use crate::types::Rule;

pub fn holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Neujahr", "neujahr(s?tag)?", month_day(1, 1)),
        ("Valentinstag", "valentin'?stag", month_day(2, 14)),
        ("Schweizer Bundesfeiertag", "schweiz(er)? (bundes)?feiertag|bundes feiertag", month_day(8, 1)),
        ("Tag der Deutschen Einheit", "tag (der)? deutsc?hen? einheit", month_day(10, 3)),
        ("Oesterreichischer Nationalfeiertag", "((ö)sterreichischer?)? nationalfeiertag|national feiertag", month_day(10, 26)),
        ("Halloween", "hall?owe?en?", month_day(10, 31)),
        ("Allerheiligen", "allerheiligen?|aller heiligen?", month_day(11, 1)),
        ("Nikolaus", "nikolaus(tag)?|nikolaus tag|nikolo", month_day(12, 6)),
        ("Heiligabend", "heilig(er)? abend", month_day(12, 24)),
        ("Weihnachten", "weih?nacht(en|stag)?", month_day(12, 25)),
        ("Silvester", "silvester", month_day(12, 31)),
        ("Muttertag", "mutt?ertag|mutt?er (tag)?", nth_dow_of_month(2, 7, 5)),
        ("Vatertag", "vatt?er( ?tag)?", nth_dow_of_month(3, 7, 6)),
    ])
}

pub fn computed_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Christi Himmelfahrt", "(christi\\s+)?himmelfahrt(stag)?", cycle_nth_after(false, G::Day, 39, &easter_sunday())),
        ("Aschermittwoch", "ascher?(tag|mittwoch)", cycle_nth_after(false, G::Day, -46, &easter_sunday())),
        ("Aschura", "asc?hura(\\-?tag)?", cycle_nth_after(false, G::Day, 9, &muharram())),
        ("Bhai Dooj", "bhai(ya)?\\s+d(u|oo)j|bhau\\-beej|bhai\\s+(tika|phonta)", cycle_nth_after(false, G::Day, 4, &dhanteras())),
        ("Chhath", "chhathi?|chhath (parv|puja)|dala (chhath|puja)|surya shashthi", cycle_nth_after(false, G::Day, 8, &dhanteras())),
        ("Boghi", "boghi|bogi\\s+pandigai", cycle_nth_after(false, G::Day, -1, &thai_pongal())),
        ("Chinesisches Neujahr", "chinesische(s|r)\\s+(neujahr(s(tag|fest))?|frühlingsfest)", chinese_new_year()),
        ("Aschermontag", "(orthodoxer?\\s+)?(ascher|reiner?\\s+|sauberer?\\s+)montag", cycle_nth_after(false, G::Day, -48, &orthodox_easter())),
        ("Corpus Christi", "corpus\\s+christi|fronleichnam", cycle_nth_after(false, G::Day, 60, &easter_sunday())),
        ("Dhanteras", "dhanatrayodashi|dhanteras|dhanvantari\\s+trayodashi", dhanteras()),
        ("Diwali", "deepavali|diwali|lakshmi\\s+puja", cycle_nth_after(false, G::Day, 2, &dhanteras())),
        ("Durga Ashtami", "(durga|maha)(\\s+a)?shtami", cycle_nth_after(false, G::Day, 7, &navaratri())),
        ("Ostermontag", "ostermontag", cycle_nth_after(false, G::Day, 1, &easter_sunday())),
        ("Ostersonntag", "ostersonntag", easter_sunday()),
        ("Eid al-Adha", "bakr[\\-\\s]e?id|e?id [au]l\\-adha|opferfest", eidal_adha()),
        ("Eid al-Fitr", "eid al\\-fitr", eidal_fitr()),
        ("Govardhan Puja", "govardhan\\s+puja|annak(u|oo)t", cycle_nth_after(false, G::Day, 3, &dhanteras())),
        ("Karfreitag", "(kar|stiller\\s+|hoher\\s+)freitag", cycle_nth_after(false, G::Day, -2, &easter_sunday())),
        ("Guru Gobind Singh Jayanti", "guru\\s+(gobind|govind)\\s+singh\\s+(Geburtstag|jayanti)", guru_gobind_singh_jayanti()),
        ("Holi", "(rangwali )?holi|dhuleti|dhulandi|phagwah", cycle_nth_after(false, G::Day, 39, &vasant_panchami())),
        ("Holika Dahan", "holika dahan|kamudu pyre|chhoti holi", cycle_nth_after(false, G::Day, 38, &vasant_panchami())),
        ("Karsamstag", "(kar|stiller\\s+)samstag|karsonnabend", cycle_nth_after(false, G::Day, -1, &easter_sunday())),
        ("Islamisches Neujahr", "(arabisches|hijri|islamisches) neujahr|amun jadid|muharram", muharram()),
        ("Isra and Mi'raj", "isra and mi'raj|aufstieg des propheten|(die\\s+)?nachtreise|aufstieg\\s+in\\s+den\\s+himmel", cycle_nth_after(false, G::Day, 26, &rajab())),
        ("Jumu'atul-Wida", "jumu'atul\\-widaa?'?|jamat[\\-\\s]ul[\\-\\s]vida", pred_nth_after(-1, &day_of_week(5), &eidal_fitr())),
        ("Kaanum Pongal", "(kaanum|kanni)\\s+pongal", cycle_nth_after(false, G::Day, 2, &thai_pongal())),
        ("Lag BaOmer", "lag (b|l)[a']omer", lag_ba_omer()),
        ("Vaisakhi", "mesadi|[bv]aisakhi|vaisakhadi|vasakhi|vaishakhi", vaisakhi()),
        ("Lailat al-Qadr", "la[iy]lat al[\\-\\s][qk]adr|(die)? nacht der (bestimmung|allmacht)", cycle_nth_after(false, G::Day, 26, &ramadan())),
        ("Lazarus-Samstag", "lazarus(\\-|\\s+)samstag", cycle_nth_after(false, G::Day, -8, &orthodox_easter())),
        ("Maha Navami", "maha\\s+navami", cycle_nth_after(false, G::Day, 8, &navaratri())),
        ("Maha Saptami", "maha\\s+saptami", cycle_nth_after(false, G::Day, 6, &navaratri())),
        ("Mattu Pongal", "maa?ttu\\s+pongal", cycle_nth_after(false, G::Day, 1, &thai_pongal())),
        ("Gründonnerstag", "(grün|hoher\\s+|heiliger\\s+|weißer\\s+|palm)donnerstag", cycle_nth_after(false, G::Day, -3, &easter_sunday())),
        ("Maulid an-Nabī", "Maulid\\s+an\\-Nabī|mawlid(\\s+al\\-nab(awi|i\\s+al\\-sharif))?|mevli[dt]|mulud|geburtstag des propheten( muhammad)?", mawlid()),
        ("Naraka Chaturdashi", "naraka? (nivaran )?chaturdashi|(kali|roop) chaudas|choti diwali", cycle_nth_after(false, G::Day, 1, &dhanteras())),
        ("Orthodoxer Ostermontag", "orthodoxer\\s+ostermontag", cycle_nth_after(false, G::Day, 1, &orthodox_easter())),
        ("Orthodoxer Ostersonntag", "orthodoxer\\s+ostersonntag", orthodox_easter()),
        ("Orthodoxer Karsamstag", "orthodoxer\\s+karsamstag", cycle_nth_after(false, G::Day, -1, &orthodox_easter())),
        ("Orthodoxer Karfreitag", "orthodoxer\\s+karfreitag", cycle_nth_after(false, G::Day, -2, &orthodox_easter())),
        ("Orthodoxer Palmsonntag", "orthodoxer\\s+palmsonntag", cycle_nth_after(false, G::Day, -7, &orthodox_easter())),
        ("Palmsonntag", "palmsonntag", cycle_nth_after(false, G::Day, -7, &easter_sunday())),
        ("Pfingsten", "pfingsten|pentecost", cycle_nth_after(false, G::Day, 49, &easter_sunday())),
        ("Purim", "purim", purim()),
        ("Raksha Bandhan", "raksha(\\s+)?bandhan|rakhi", raksha_bandhan()),
        ("Pargat Diwas", "pargat diwas|(maharishi )?valmiki jayanti", pargat_diwas()),
        ("Mahavir Jayanti", "(mahavir|mahaveer) (jayanti|janma kalyanak)", mahavir_jayanti()),
        ("Maha Shivaratri", "maha(\\s+)?shivaratri", maha_shiva_ratri()),
        ("Dayananda Saraswati Jayanti", "((maharishi|swami) )?(dayananda )?saraswati jayanti", saraswati_jayanti()),
        ("Karva Chauth", "karva\\s+chauth|karaka\\s+chaturthi", karva_chauth()),
        ("Krishna Janmashtami", "(krishna )?janmashtami|gokulashtami", krishna_janmashtami()),
        ("Schmini Azeret", "sc?he?mini\\s+at?zeret", cycle_nth_after(false, G::Day, 21, &rosh_hashana())),
        ("Fastnacht", "fastnacht(sdienstag)?|mardi gras", cycle_nth_after(false, G::Day, -47, &easter_sunday())),
        ("Shushan Purim", "shushan\\s+purim", cycle_nth_after(false, G::Day, 1, &purim())),
        ("Simchat Torah", "simc?hat\\s+torah", cycle_nth_after(false, G::Day, 22, &rosh_hashana())),
        ("Thai Pongal", "(thai )?pongal|pongal pandigai|(makara? |magha )?sankranth?i|maghi", thai_pongal()),
        ("Thiru Onam", "(thiru(v|\\s+))?onam", thiru_onam()),
        ("Tisha B'Av", "tisha b'av", tisha_b_av()),
        ("Dreifaltigkeitssonntag", "trinitatis(fest)?|(dreifaltigkeits|goldener\\s+)sonntag|drei(faltigkeit|einigkeit)(sfest)?", cycle_nth_after(false, G::Day, 56, &easter_sunday())),
        ("Vasant Panchami", "[bv]asant\\s+panchami", vasant_panchami()),
        ("Vijayadashami", "dasara|duss(eh|he)ra|vijayadashami", cycle_nth_after(false, G::Day, 9, &navaratri())),
        ("Tu biSchevat", "tu b[i']sc?he?vat", tu_bishvat()),
        ("Vesak", "v(e|ai)sak(ha)?|buddha(\\-?tag|\\s+purnima)|wesakfest", vesak()),
        ("Jom Ha'atzmaut", "[yj]om ha'?atzmaut", yom_haatzmaut()),
        ("Jom HaShoah", "[yj]om hashoah|[yj]om hazikaron lashoah ve-lag'vurah|holocaust\\-?gedenktag", cycle_nth_after(false, G::Day, 12, &passover())),
        ("Jom Kippur", "[yj]om\\s+kippur", cycle_nth_after(false, G::Day, 9, &rosh_hashana())),
        ("Pfingstmontag", "pfingstmontag|(pentecost|whit)\\s+montag", cycle_nth_after(false, G::Day, 50, &easter_sunday())),
        ("Rabindra Jayanti", "rabindra(nath)?\\s+jayanti", rabindra_jayanti()),
        ("Guru Ravidass Jayanti", "guru\\s+ravidass?\\s+(geburtstag|jayanti)", ravidass_jayanti()),
    ])
}

pub fn computed_holidays_prime() -> Vec<Rule> {
    mk_rule_holidays_maybe(&[
        ("Global Youth Service-Tag", "global youth service[\\-\\s]?tag|gysd", { let start = global_youth_service_day(); let end = cycle_nth_after(false, G::Day, 2, &global_youth_service_day()); interval(Open, &start, &end) }),
        ("Große Fastenzeit", "große\\s+fastenzeit", { let start = cycle_nth_after(false, G::Day, -48, &orthodox_easter()); let end = cycle_nth_after(false, G::Day, -9, &orthodox_easter()); interval(Open, &start, &end) }),
        ("Chanukka", "c?hann?ukk?ah?", { let start = chanukah(); let end = cycle_nth_after(false, G::Day, 7, &chanukah()); interval(Open, &start, &end) }),
        ("Fastenzeit", "fastenzeit", { let start = cycle_nth_after(false, G::Day, -46, &easter_sunday()); let end = cycle_nth_after(false, G::Day, -1, &easter_sunday()); interval(Open, &start, &end) }),
        ("Navaratri", "durga\\s+puja|durgotsava|nava?rath?ri", { let start = navaratri(); let end = cycle_nth_after(false, G::Day, 9, &navaratri()); interval(Open, &start, &end) }),
        ("Pessach", "passover|pess?a[ck]?h|pascha|Passah?", { let start = passover(); let end = cycle_nth_after(false, G::Day, 8, &passover()); interval(Open, &start, &end) }),
        ("Ramadan", "rama[dt]h?an|ramzaa?n", { let start = ramadan(); let end = cycle_nth_after(false, G::Day, -1, &eidal_fitr()); interval(Open, &start, &end) }),
        ("Rosch haSchana", "rosch ha\\-?schanah?", { let start = rosh_hashana(); let end = cycle_nth_after(false, G::Day, 2, &rosh_hashana()); interval(Open, &start, &end) }),
        ("Schawuot", "sc?ha[vw]u'?oth?|shovuos", { let start = cycle_nth_after(false, G::Day, 50, &passover()); let end = cycle_nth_after(false, G::Day, 52, &passover()); interval(Open, &start, &end) }),
        ("Sukkot", "Laubhüttenfest|su[ck]{2}o[st]", { let start = cycle_nth_after(false, G::Day, 14, &rosh_hashana()); let end = cycle_nth_after(false, G::Day, 22, &rosh_hashana()); interval(Open, &start, &end) }),
        ("Parsi Neujahr", "parsi neujahr|jamshedi navroz", pred_every_n_days_from(365, (2020, 8, 16))),
        ("Earth Hour", "earth hour|stunde der erde", compute_earth_hour()),
        ("Königstag", "königstag|koningsdag", compute_kings_day()),
    ])
}

