#!/usr/bin/env python3
"""Transpile Duckling.Time.<LANG>.Rules.hs into a Rust time.rs.

Usage: hs_time_rules.py <Rules.hs> <Lang> <out.rs>
This is a best-effort mechanical port. Review compile errors and parity diffs.
"""
import re, sys

PRED = {
    "isNotLatent": "is_not_latent",
    "isADayOfWeek": "is_a_day_of_week",
    "isATimeOfDay": "is_a_time_of_day",
    "isAPartOfDay": "is_a_part_of_day",
    "isAMonth": "is_a_month",
    "isAnHourOfDay": "is_an_hour_of_day",
    "isMidnightOrNoon": "is_midnight_or_noon",
    "isDOMInteger": "is_dom_integer",
    "isDOMOrdinal": "is_dom_ordinal",
    "isDOMValue": "is_dom_value",
    "isOkWithThisNext": "is_ok_with_this_next",
}

FUN = {
    "cycleNth": "cycle_nth",
    "cycleN": "cycle_n",
    "cycleNthAfter": "cycle_nth_after",
    "cycleLastOf": "cycle_last_of",
    "durationAgo": "duration_ago",
    "durationAfter": "duration_after",
    "durationBefore": "duration_before",
    "inDuration": "in_duration",
    "predNth": "pred_nth",
    "predNthAfter": "pred_nth_after",
    "predLastOf": "pred_last_of",
    "predNthClosest": "pred_nth_closest",
    "hourMinute": "hour_minute",
    "hourMinuteSecond": "hour_minute_second",
    "dayOfMonth": "day_of_month",
    "dayOfWeek": "day_of_week",
    "monthDay": "month_day",
    "yearMonthDay": "year_month_day",
    "yearMonth": "year_month",
    "mkLatent": "mk_latent",
    "notLatent": "not_latent",
    "partOfDay": "part_of_day",
    "timeOfDay": "time_of_day",
    "minutesBefore": "minutes_before",
    "minutesAfter": "minutes_after",
    "intersectDOM": "intersect_dom",
    "inTimezone": "in_timezone",
    "withDirection": "with_direction",
    "withHoliday": "with_holiday",
    "mkOkForThisNext": "mk_ok_for_this_next",
    "getIntValue": "get_int_value",
    "parseInt": "parse_int",
    "isIntegerBetween": "is_integer_between_tok",
    "isOrdinalBetween": "is_ordinal_between",
    "isDurationGreaterThan": "is_duration_greater_than",
    "isGrainFinerThan": "is_grain_finer_than",
    "isGrainCoarserThan": "is_grain_coarser_than",
    "isGrainOfTime": "is_grain_of_time",
}

ZERO_ARG = {"today", "now", "weekend", "weekday", "season"}


def snake(n):
    n = n.replace("nthDOWOfMonth", "nthDowOfMonth").replace("longWEBefore", "longWeBefore")
    n = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", n)
    return re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", n).lower()


def rust_str(s):
    return s.replace("\\", "\\\\").replace('"', '\\"')


def tr_ident(name):
    if name in PRED:
        return PRED[name]
    if name in FUN:
        return FUN[name]
    if name.startswith("TG."):
        return "G::" + name[3:]
    if name.startswith("TTime."):
        t = name[6:]
        return {"Open": "Open", "Closed": "Closed", "After": "IntervalDirection::After",
                "Before": "IntervalDirection::Before"}.get(t, t)
    if name in ("True", "False"):
        return name.lower()
    if name in ("Nothing",):
        return "None"
    return snake(name)


def strip_comments(src):
    src = re.sub(r"--[^\n]*", "", src)
    src = re.sub(r"{-[\\s\\S]*?-}", "", src)
    return src


def split_top_rules(src):
    """Return (name, body) for each `ruleX = Rule { ... }` and list rules."""
    out = []
    i = 0
    while True:
        m = re.search(r"^(rule\w+)\s*::\s*", src[i:], re.M)
        if not m:
            break
        start = i + m.start()
        name = m.group(1)
        rest = src[start:]
        # find the '=' of the definition after the type sig
        eq = re.search(r"^" + re.escape(name) + r"\s*=\s*", rest, re.M)
        if not eq:
            i = start + 1
            continue
        body_start = eq.end()
        # next top-level binding
        nxt = re.search(r"^(rule\w+|rules)\s*::\s*", rest[body_start:], re.M)
        body = rest[body_start: body_start + nxt.start() if nxt else None].rstrip()
        out.append((name, body.strip()))
        i = start + (nxt.start() + body_start if nxt else len(rest))
    return out


def parse_string(s, i):
    assert s[i] == '"'
    i += 1
    out = []
    while i < len(s):
        if s[i] == "\\":
            out.append(s[i:i+2]); i += 2; continue
        if s[i] == '"':
            return "".join(out), i + 1
        out.append(s[i]); i += 1
    raise ValueError("unterminated string")


def parse_pattern_items(block):
    m = re.search(r"pattern\s*=\s*\[", block)
    if not m:
        return []
    i = m.end()
    items = []
    depth = 1
    start = i
    while i < len(block) and depth:
        c = block[i]
        if c == '"':
            _, i = parse_string(block, i)
            continue
        if c == "[":
            depth += 1
        elif c == "]":
            depth -= 1
            if depth == 0:
                raw = block[start:i]
                break
        i += 1
    else:
        raw = block[start:]
    # split top-level commas
    parts, buf, d, j = [], [], 0, 0
    while j < len(raw):
        if raw[j] == '"':
            s, j2 = parse_string(raw, j)
            buf.append(raw[j:j2]); j = j2; continue
        if raw[j] in "[(":
            d += 1
        elif raw[j] in "])":
            d -= 1
        if raw[j] == "," and d == 0:
            parts.append("".join(buf).strip()); buf = []; j += 1; continue
        buf.append(raw[j]); j += 1
    if "".join(buf).strip():
        parts.append("".join(buf).strip())
    return [p for p in parts if p]


def tr_pattern(item):
    item = " ".join(item.split())
    m = re.match(r'regex\s+"(.*)"\s*$', item, re.S)
    if m:
        return f'regex("{m.group(1)}")'
    m = re.match(r"dimension\s+(\w+)\s*$", item)
    if m:
        return f"dimension(Dim::{m.group(1)})"
    m = re.match(r"Predicate\s+\$\s+isIntegerBetween\s+(-?\d+)\s+(-?\d+)\s*$", item)
    if m:
        return f"pred(is_integer_between_tok({m.group(1)}, {m.group(2)}))"
    m = re.match(r"Predicate\s+\$\s+isOrdinalBetween\s+(-?\d+)\s+(-?\d+)\s*$", item)
    if m:
        return f"pred(is_ordinal_between({m.group(1)}, {m.group(2)}))"
    m = re.match(r"Predicate\s+\$\s+isDurationGreaterThan\s+TG\.(\w+)\s*$", item)
    if m:
        return f"pred(is_duration_greater_than(G::{m.group(1)}))"
    m = re.match(r"Predicate\s+\$\s+isGrain(?:OfTime|FinerThan|CoarserThan)?\s+TG\.(\w+)\s*$", item)
    if m:
        fn = "is_grain_of_time"
        if "Finer" in item:
            fn = "is_grain_finer_than"
        elif "Coarser" in item:
            fn = "is_grain_coarser_than"
        return f"pred({fn}(G::{m.group(1)}))"
    m = re.match(r"Predicate\s+\$\s+isGrain\s+TG\.(\w+)\s*$", item)
    if m:
        return f"pred(is_grain(G::{m.group(1)}))"
    m = re.match(r"Predicate\s+\$\s+and\s+\.\s+sequence\s+\[(.*)\]\s*$", item)
    if m:
        names = [PRED.get(x.strip(), snake(x.strip())) for x in m.group(1).split(",") if x.strip()]
        body = " && ".join(f"{n}(t)" for n in names)
        return f"pred(|t| {body})"
    m = re.match(r"Predicate\s+(\w+)\s*$", item)
    if m:
        return f"pred({PRED.get(m.group(1), snake(m.group(1)))})"
    return f"/* TODO pattern: {item} */ pred(|_| false)"


def extract_name(block):
    m = re.search(r'name\s*=\s*"(.*?)"', block, re.S)
    return m.group(1) if m else "unnamed"


def extract_prod(block):
    m = re.search(r",\s*prod\s*=\s*", block)
    if not m:
        return ""
    return block[m.end():].strip().rstrip("}").strip().rstrip(",").strip()


def tokenize_expr(s):
    s = re.sub(r"\(\s*-\s*", "(-", s)
    s = s.replace("<$>", " <$> ")
    return re.findall(r"<\|>|\(|\)|\$|[A-Za-z_][A-Za-z0-9_.']*|-?\d+|,", s)


class HP:
    def __init__(self, toks):
        self.t = toks
        self.i = 0

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else None

    def next(self):
        v = self.t[self.i]
        self.i += 1
        return v

    def expr(self):
        a = self.app()
        if self.peek() == "$":
            self.next()
            b = self.expr()
            return ("app", a[1], a[2] + [b]) if a[0] == "app" else ("app", a, [b])
        if self.peek() == "<$>":
            self.next()
            b = self.expr()
            return ("fmap", a, b)
        return a

    def app(self):
        head = self.atom()
        args = []
        while self.peek() not in (None, ")", "$", ",", "<$>"):
            args.append(self.atom())
        return ("app", head, args) if args else head

    def atom(self):
        t = self.next()
        if t == "(":
            e = self.expr()
            if self.peek() == ",":
                items = [e]
                while self.peek() == ",":
                    self.next()
                    items.append(self.expr())
                assert self.next() == ")"
                return ("tuple", items)
            assert self.next() == ")"
            return e
        return ("atom", t)


# names that are TimeData values already in scope (not nullary constructors)
BOUND = set()

# functions whose result is Option<TimeData> (need ttd / ?)
OPT_FNS = {
    "interval",
    "intersect",
    "minutes_before",
    "minutes_after",
    "intersect_dom",
    "in_timezone",
}


def emit_hs(e, as_ref=False):
    if e[0] == "fmap":
        return "ttd(" + emit_hs(e[2]) + ")"
    if e[0] == "tuple":
        return "(" + ", ".join(emit_hs(x) for x in e[1]) + ")"
    if e[0] == "atom":
        t = e[1]
        if re.fullmatch(r"-?\d+", t):
            return t
        if t in ("False", "True"):
            return t.lower()
        if t.startswith("TG."):
            return "G::" + t[3:]
        if t.startswith("TTime."):
            return {
                "Open": "Open",
                "Closed": "Closed",
                "After": "IntervalDirection::After",
                "Before": "IntervalDirection::Before",
            }.get(t[6:], t[6:])
        if t in BOUND:
            return ("&" if as_ref else "") + t
        if t in ZERO_ARG:
            return ("&" if as_ref else "") + t + "()"
        # bare identifier used as TimeData (td, td1, grain, n, ...)
        if re.match(r"^[a-z_]", t):
            return ("&" if as_ref else "") + t
        return tr_ident(t) + ("()" if t[0].isupper() else "")
    head, args = e[1], e[2]
    if head[0] == "atom" and head[1] == "tt":
        inner = emit_hs(args[0]) if args else ""
        return f"tt({inner})"
    if head[0] == "atom" and head[1] in ("Token",) and args:
        # Token Time <$> expr  parsed as app Token [Time, expr] if <$> was dropped
        pass
    hname = head[1] if head[0] == "atom" else emit_hs(head)
    hname = tr_ident(hname) if re.match(r"^[A-Za-z]", hname) else hname
    rust_args = []
    for a in args:
        need_ref = hname in {
            "intersect",
            "interval",
            "pred_nth",
            "pred_last_of",
            "pred_nth_after",
            "duration_after",
            "duration_before",
            "cycle_nth_after",
            "cycle_last_of",
            "minutes_before",
            "minutes_after",
            "in_timezone",
            "intersect_dom",
            "not_latent",
            "mk_latent",
            "part_of_day",
            "with_direction",
            "duration_ago",
            "in_duration",
        }
        s = emit_hs(a, as_ref=need_ref and a[0] == "atom")
        if need_ref and a[0] == "app" and not s.startswith("&"):
            s = "&" + s
        rust_args.append(s)
    if hname == "NEGATE":
        return "-" + rust_args[0]
    call = f"{hname}({', '.join(rust_args)})"
    if hname in OPT_FNS:
        return call
    return call


def rewrite_expr(s):
    s = s.strip().rstrip(";").strip()
    s = s.replace("Token Time .", "")
    s = re.sub(r"tt\s+\.", "tt $ ", s)
    s = re.sub(r"\(\s*-\s*(\w+)\s*\)", r"(NEGATE \1)", s)
    try:
        toks = tokenize_expr(s)
        if not toks:
            return s
        p = HP(toks)
        e = p.expr()
        out = emit_hs(e)
        if out.startswith("ttd_opt("):
            out = "ttd(" + out[len("ttd_opt("):]
        return out
    except Exception:
        return s


def simple_prod(prod, npat):
    """Try to emit a Rust closure. Return None if too hard."""
    p = " ".join(prod.split())
    # \_ -> tt $ expr   or  \_ -> tt expr
    m = re.match(r"\\_?\s*->\s*(.*)$", p)
    if m and "case " not in p and "\\case" not in p and "let " not in p:
        body = m.group(1).strip()
        # Token Time . mkLatent . partOfDay <$> interval TTime.Open from to
        # handled if we expand lets below
        if body.startswith("tt"):
            expr = rewrite_expr(body)
            if expr.startswith("tt(") and not expr.endswith(")"):
                expr += ")"
            return f"|_| {expr}"
        if "interval" in body and "partOfDay" in body:
            return None
        expr = rewrite_expr(body)
        return f"|_| {expr}"

    # no case, just do-block of simple binds — skip
    if "case tokens of" not in p and "\\case" not in p:
        return None

    # Very common: (_:Token Duration dd:_) -> tt $ durationAgo dd
    # We translate by positional Token X bindings.
    # Extract the first successful pattern arm.
    arm = None
    for pat in re.split(r"\s+(?:_->\s*Nothing|_ -> Nothing)", p)[:1]:
        arm = pat
    # Find the first `->` after the case header that is the production.
    # Simpler heuristic: if only one interesting arm, parse Token positions.
    tokens_pat = re.search(
        r"(?:case tokens of|\\\\case)\s*(.*?)(?:->\s*)(.*?)$",
        p,
    )
    # Fall back: generate a positional matcher from the first arm
    first = re.search(
        r"\(([^)]*)\)\s*->\s*(do\s+)?(.*)",
        p,
    )
    if not first:
        return None
    pat_s, body_s = first.group(1), first.group(3)
    # If body still has another case / Text.toLower match on alternatives, bail if many
    if body_s.count("->") > 3 and "Text.toLower" in body_s:
        return None

    slots = [x.strip() for x in pat_s.split(":")]
    binds = []  # (index, kind, name)
    idx = 0
    for sl in slots:
        if sl in ("_", "") or sl == "...":
            if sl == "...":
                break
            idx += 1
            continue
        if sl.startswith("Token Duration"):
            name = sl.split()[-1]
            if name in ("Duration", "DurationData"):
                name = "dd"
            binds.append((idx, "dur", name))
        elif "TimeGrain" in sl:
            name = sl.split()[-1]
            if name in ("TimeGrain",):
                name = "grain"
            binds.append((idx, "grain", name))
        elif "GroupMatch" in sl:
            # GroupMatch (m1:m2:_) or (match:_)
            gm = re.findall(r"\b([a-z][A-Za-z0-9]*)\b", sl)
            gm = [g for g in gm if g not in ("Token", "RegexMatch", "GroupMatch")]
            binds.append((idx, "rxs", gm or ["m"]))
        elif sl.startswith("Token Time") or "TimeData" in sl:
            name = sl.split()[-1]
            if name in ("Time", "TimeData", "{", "_"):
                # Token Time td   — last ident before optional {
                mm = re.search(r"Token Time (\w+)", sl)
                name = mm.group(1) if mm else "td"
            binds.append((idx, "time", name))
        elif sl.startswith("Token Numeral") or sl.startswith("Token Ordinal"):
            name = sl.split()[-1]
            if name in ("Numeral", "Ordinal", "NumeralData"):
                name = "token"
            binds.append((idx, "tok", name))
        elif re.match(r"^[a-zA-Z_][\w']*$", sl):
            binds.append((idx, "tok", sl.rstrip("'")))
        idx += 1

    # body cleanup
    body = body_s
    body = re.sub(r"^do\s+", "", body)
    # v <- getIntValue token; tt $ year v  =>  let v = get_int_value(...)?; tt(year(v))
    lets = []
    for bm in re.finditer(r"(\w+)\s*<-\s*getIntValue\s+(\w+)", body):
        lets.append((bm.group(1), f"get_int_value({bm.group(2)})?"))
    for bm in re.finditer(r"(\w+)\s*<-\s*parseInt\s+(\w+)", body):
        lets.append((bm.group(1), f"parse_int(&{bm.group(2)})?"))
    body = re.sub(r"\w+\s*<-\s*getIntValue\s+\w+\s*", "", body)
    body = re.sub(r"\w+\s*<-\s*parseInt\s+\w+\s*", "", body)
    # from <- intersect a b
    for bm in re.finditer(r"(\w+)\s*<-\s*intersect\s+(\([^)]+\)|\S+)\s+(\([^)]+\)|\S+)", body):
        lets.append((bm.group(1), f"intersect(&{rewrite_expr(bm.group(2))}, &{rewrite_expr(bm.group(3))})?"))
    body = re.sub(r"\w+\s*<-\s*intersect\s+\S+\s+\S+\s*", "", body)
    body = body.replace("_ -> Nothing", "").replace("_ ->Nothing", "")
    body = rewrite_expr(body)
    # cleanup leftover arrows / junk
    if "case " in body or "\\case" in body or "Text.to" in body:
        return None

    # bind prelude
    prelude = []
    bound = set()
    for i, kind, name in binds:
        if kind == "dur":
            prelude.append(f"let {name} = dur(ts[{i}])?;")
            bound.add(name)
        elif kind == "grain":
            prelude.append(f"let {name} = grain_of(ts[{i}])?;")
            bound.add(name)
        elif kind == "time":
            prelude.append(f"let {name} = td(ts[{i}])?;")
            bound.add(name)
        elif kind == "rxs":
            for gi, gname in enumerate(name):
                prelude.append(f"let {gname} = rx(ts[{i}], {gi})?;")
                bound.add(gname)
        elif kind == "rx":
            prelude.append(f"let {name} = rx(ts[{i}], 0)?;")
            bound.add(name)
        elif kind == "tok":
            prelude.append(f"let {name} = &ts[{i}];")
            bound.add(name)
    for n, e in lets:
        prelude.append(f"let {n} = {e};")
        bound.add(n)
    global BOUND
    BOUND = bound | {n for n, _ in lets}

    # hourMinute is12H hours n after TimeData form bind — need special case
    if "TimeOfDay (Just hours)" in prod or "TimeOfDay (Just hours)" in p:
        # find the time token index
        tidx = next((i for i, k, _ in binds if k == "time"), 0)
        prelude = [f"let (hours, is12h) = match &td(ts[{tidx}])?.form {{ Some(Form::TimeOfDay {{ hours: Some(h), is12h }}) => (*h, *is12h), _ => return None }};"]
        # re-add other binds
        for i, kind, name in binds:
            if kind == "time":
                continue
            if kind == "dur":
                prelude.append(f"let {name} = dur(ts[{i}])?;")
            elif kind == "grain":
                prelude.append(f"let {name} = grain_of(ts[{i}])?;")
            elif kind == "tok":
                prelude.append(f"let {name} = &ts[{i}];")
        for n, e in lets:
            prelude.append(f"let {n} = {e};")
        body = body.replace("is12H", "is12h")

    body = body.strip().rstrip(";").strip()
    if body.startswith("tt ") and not body.startswith("tt("):
        body = "tt(" + body[3:] + ")"
    if not body:
        return None
    inner = " ".join(prelude + [body])
    return f"|ts| {{ {inner} }}"


def emit_rule_fn(hs_name, block):
    name = extract_name(block)
    pats = parse_pattern_items(block)
    rust_pats = ", ".join(tr_pattern(p) for p in pats)
    prod = extract_prod(block)
    clos = simple_prod(prod, len(pats))
    fn = snake(hs_name)
    if clos is None:
        clos = f'|ts| {{ let _ = ts; /* TODO prod for {hs_name} */ None }}'
    return fn, name, f'''fn {fn}() -> Rule {{
    rule(
        "{rust_str(name)}",
        vec![{rust_pats}],
        {clos},
    )
}}
'''


def parse_tuple_list(body):
    """Parse Haskell list of tuples of strings / exprs."""
    # very loose: find ("a", "b"[, extras])
    items = []
    for m in re.finditer(
        r'\(\s*"([^"]*)"\s*,\s*"([^"]*)"\s*(?:,\s*([^)]+))?\s*\)',
        body,
    ):
        extra = m.group(3).strip() if m.group(3) else None
        items.append((m.group(1), m.group(2), extra))
    return items


def emit_mk_list(hs_name, body):
    fn = snake(hs_name)
    if body.startswith("mkRuleDaysOfWeek"):
        items = parse_tuple_list(body)
        rows = ",\n        ".join(f'("{a}", "{b}")' for a, b, _ in items)
        return fn, f"fn {fn}() -> Vec<Rule> {{\n    mk_rule_days_of_week(&[\n        {rows},\n    ])\n}}\n"
    if body.startswith("mkRuleMonths"):
        items = parse_tuple_list(body)
        rows = ",\n        ".join(f'("{a}", "{b}")' for a, b, _ in items)
        return fn, f"fn {fn}() -> Vec<Rule> {{\n    mk_rule_months(&[\n        {rows},\n    ])\n}}\n"
    if body.startswith("mkRuleSeasons"):
        items = []
        for m in re.finditer(
            r'\(\s*"([^"]*)"\s*,\s*"([^"]*)"\s*,\s*monthDay\s+(\d+)\s+(\d+)\s*,\s*monthDay\s+(\d+)\s+(\d+)\s*\)',
            body,
        ):
            items.append((m.group(1), m.group(2), m.group(3), m.group(4), m.group(5), m.group(6)))
        rows = ",\n        ".join(
            f'("{a}", "{b}", month_day({c}, {d}), month_day({e}, {f}))' for a, b, c, d, e, f in items
        )
        return fn, f"fn {fn}() -> Vec<Rule> {{\n    mk_rule_seasons(&[\n        {rows},\n    ])\n}}\n"
    if body.startswith("mkRuleHolidays"):
        # ("Name", "pat", expr)
        items = []
        for m in re.finditer(
            r'\(\s*"([^"]*)"\s*,\s*"([^"]*)"\s*,\s*([^)]+?)\)',
            body,
        ):
            expr = " ".join(m.group(3).split())
            expr = rewrite_expr(expr)
            # monthDay already rewritten
            items.append((m.group(1), m.group(2), expr))
        rows = ",\n        ".join(f'("{a}", "{b}", {c})' for a, b, c in items)
        return fn, f"fn {fn}() -> Vec<Rule> {{\n    mk_rule_holidays(&[\n        {rows},\n    ])\n}}\n"
    return fn, f"// TODO list {hs_name}\nfn {fn}() -> Vec<Rule> {{ Vec::new() }}\n"


HEADER = '''//! `Duckling.Time.{lang}.Rules` (generated by tools/hs_time_rules.py; then edited for parity).

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{{
    Form, IntervalDirection, TimeData,
    TimeIntervalType::{{Closed, Open}},
}};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::parse_int;
use crate::types::{{dimension, pred, regex, rule, Dim, Rule, Token}};

fn td(t: &Token) -> Option<&TimeData> {{
    match t {{
        Token::Time(d) => Some(d),
        _ => None,
    }}
}}

fn rx(t: &Token, i: usize) -> Option<String> {{
    match t {{
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }}
}}

fn ttd(o: Option<TimeData>) -> Option<Token> {{
    o.map(Token::Time)
}}

fn grain_of(t: &Token) -> Option<G> {{
    match t {{
        Token::TimeGrain(g) => Some(*g),
        _ => None,
    }}
}}

fn dur(t: &Token) -> Option<&DurationData> {{
    match t {{
        Token::Duration(d) => Some(d),
        _ => None,
    }}
}}

fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {{
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}}

'''


def main():
    src = open(sys.argv[1], encoding="utf-8").read()
    lang = sys.argv[2]
    outp = sys.argv[3]
    src = strip_comments(src)
    defs = split_top_rules(src)
    fns = []
    rust_fns = []
    list_fns = []
    for name, body in defs:
        if name == "rules":
            continue
        if body.startswith("Rule") or body.startswith("{"):
            fn, rname, code = emit_rule_fn(name, body)
            fns.append((name, fn, False))
            rust_fns.append(code)
        else:
            fn, code = emit_mk_list(name, body)
            fns.append((name, fn, True))
            list_fns.append(code)

    # rules list from Haskell
    rm = re.search(r"^rules\s*::\s*\[Rule\]\s*rules\s*=\s*(.*)", src, re.S | re.M)
    order = []
    extend = []
    if rm:
        tail = rm.group(1)
        for m in re.finditer(r"\b(rule\w+)\b", tail):
            n = m.group(1)
            if n == "rules":
                continue
            # in the concatenation part
            order.append(n)
        # unique preserving, but ++ lists appear after singles
    # Map hs name -> rust fn
    h2r = {h: r for h, r, _ in fns}
    is_list = {h: lst for h, _, lst in fns}

    single = []
    lists = []
    seen = set()
    if rm:
        # parse [ a, b, ...] ++ x ++ y
        head = rm.group(1)
        bracket = re.search(r"\[(.*?)\]", head, re.S)
        if bracket:
            for m in re.finditer(r"\b(rule\w+)\b", bracket.group(1)):
                n = m.group(1)
                if n not in seen:
                    seen.add(n); single.append(n)
        for m in re.finditer(r"\+\+\s*(rule\w+)", head):
            lists.append(m.group(1))

    pieces = []
    pieces.append(HEADER.format(lang=lang))
    pieces.extend(rust_fns)
    pieces.extend(list_fns)
    vec = ",\n        ".join(h2r.get(n, snake(n)) + "()" for n in single)
    ext = "\n    ".join(f"v.extend({h2r.get(n, snake(n))}());" for n in lists)
    pieces.append(f"""
fn build_rules() -> Vec<Rule> {{
    let mut v = vec![
        {vec}
    ];
    {ext}
    v
}}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {{
    &RULES
}}
""")
    open(outp, "w").write("\n".join(pieces))
    todos = sum(1 for c in rust_fns if "TODO prod" in c)
    print(f"wrote {outp}: {len(single)} rules, {len(lists)} lists, {todos} TODO prods")


if __name__ == "__main__":
    main()
