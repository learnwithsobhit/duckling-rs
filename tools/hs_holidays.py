#!/usr/bin/env python3
"""Convert the Haskell holiday rule tables (mkRuleHolidays lists) of every
Duckling.Time.EN.*.Rules module into Rust source.

Usage: hs_holidays.py <upstream Duckling/Time/EN dir> <out.rs>
The EN module is read from a comment-stripped copy at <dir>/Rules.hs.
"""
import os, re, sys

LISTS = [
    ('rulePeriodicHolidays', 'periodic_holidays', False),
    ("rulePeriodicHolidays'", 'periodic_holidays_prime', True),
    ('ruleComputedHolidays', 'computed_holidays', False),
    ("ruleComputedHolidays'", 'computed_holidays_prime', True),
    ('ruleBackwardCompatibleHolidays', 'backward_compatible_holidays', False),
]

# Items whose Haskell uses `do` blocks; hand-written in manual_holidays.rs.
MANUAL = {'Royal Queensland Show Day': 'royal_queensland_show_day',
          'Royal Queensland Show': 'royal_queensland_show'}

def snake(n):
    n = n.replace('nthDOWOfMonth', 'nthDowOfMonth').replace('longWEBefore', 'longWeBefore')
    return re.sub(r'([A-Z])', lambda m: '_' + m.group(1).lower(), n)

def tokenize(s):
    s = re.sub(r'\(\s*-\s*(\d)', r'(-\1', s)
    return re.findall(r'\(|\)|\$|[A-Za-z_][A-Za-z0-9_.\']*|-?\d+|,', s)

class P:
    def __init__(self, toks): self.t = toks; self.i = 0
    def peek(self): return self.t[self.i] if self.i < len(self.t) else None
    def next(self): v = self.t[self.i]; self.i += 1; return v
    def expr(self):
        a = self.app()
        if self.peek() == '$':
            self.next(); b = self.expr()
            return ('app', a[1], a[2] + [b]) if a[0] == 'app' else ('app', a, [b])
        return a
    def app(self):
        head = self.atom(); args = []
        while self.peek() not in (None, ')', '$', ','):
            args.append(self.atom())
        return ('app', head, args) if args else head
    def atom(self):
        t = self.next()
        if t == '(':
            e = self.expr()
            if self.peek() == ',':
                items = [e]
                while self.peek() == ',':
                    self.next(); items.append(self.expr())
                assert self.next() == ')'
                return ('tuple', items)
            assert self.next() == ')'
            return e
        return ('atom', t)

BOUND = set()

def emit(e):
    if e[0] == 'tuple':
        return '(' + ', '.join(emit(x) for x in e[1]) + ')'
    if e[0] == 'atom':
        t = e[1]
        if re.fullmatch(r'-?\d+', t): return t
        if t == 'False': return 'false'
        if t == 'True': return 'true'
        if t.startswith('TG.'): return 'G::' + t[3:]
        if t.startswith('TTime.'): return t[6:]
        if t in BOUND: return t
        return snake(t) + '()'
    head, args = e[1], e[2]
    out = []
    for a in args:
        s = emit(a)
        if a[0] == 'app' or (a[0] == 'atom' and (s.endswith('()') or a[1] in BOUND)):
            s = '&' + s
        out.append(s)
    return f"{snake(head[1])}({', '.join(out)})"

def balanced_items(body):
    items = []; depth = 0; cur = ''; instr = False; esc = False
    for ch in body:
        if instr:
            cur += ch
            if esc: esc = False
            elif ch == '\\': esc = True
            elif ch == '"': instr = False
            continue
        if ch == '"': instr = True; cur += ch; continue
        if ch == '(':
            depth += 1
            if depth == 1: cur = ''; continue
        if ch == ')':
            depth -= 1
            if depth == 0: items.append(cur); cur = ''; continue
        if depth >= 1: cur += ch
    return items

STR = r'"(?:[^"\\]|\\.)*"'

def split_item(it, consts):
    m = re.match(r'\s*(' + STR + r')\s*,\s*((?:' + STR + r'|\w+)(?:\s*\+\+\s*(?:' + STR + r'|\w+))*)\s*,\s*(.*)\Z', it, re.S)
    if not m:
        raise Exception('bad item: ' + it)
    pat = m.group(2)
    parts = re.findall(STR + r'|\w+', pat)
    lits = []
    for p in parts:
        lits.append(p[1:-1] if p.startswith('"') else consts[p][1:-1])
    return m.group(1), '"' + ''.join(lits) + '"', m.group(3).strip()

def conv_expr(txt):
    txt = ' '.join(txt.split())
    BOUND.clear()
    m = re.fullmatch(r'let (.*?) in (.*)', txt)
    if m and not m.group(2).startswith('do'):
        parts = re.split(r'(?:^|\s)(\w+) = ', m.group(1))
        lets = []
        names = parts[1::2]; exprs = parts[2::2]
        for n, ex in zip(names, exprs):
            lets.append(f"let {n} = {emit(P(tokenize(ex)).expr())};")
            BOUND.add(n)
        body = emit(P(tokenize(m.group(2))).expr())
        return '{ ' + ' '.join(lets) + ' ' + body + ' }'
    return emit(P(tokenize(txt)).expr())

def section(src, name):
    i = src.index('\n' + name + ' = ')
    j = src.index('\n  [', i)
    k = src.index('\n  ]', j)
    return src[j:k + 4]

def gen_module(mod, src):
    consts = dict(re.findall(r'^(\w+) :: String\n\1 = (".*")$', src, re.M))
    out = [f"pub mod {mod} {{", "    use super::super::computed::*;", "    use super::super::helpers::*;",
           "    use super::super::holiday_helpers::*;", "    use super::super::manual_holidays::*;",
           "    use super::super::types::TimeIntervalType::Open;",
           "    use crate::dims::time_grain::Grain as G;", "    use crate::types::Rule;", ""]
    for hs, rs, maybe in LISTS:
        if '\n' + hs + ' = ' not in src:
            continue
        body = section(src, hs)
        items = balanced_items(body[body.index('['):])
        out.append(f"    pub fn {rs}() -> Vec<Rule> {{")
        out.append("        mk_rule_holidays_maybe(&[" if maybe else "        mk_rule_holidays(&[")
        for it in items:
            n, r, e = split_item(it, consts)
            if re.search(r'\bdo\b', e):
                nm = n.strip('"')
                ce = f"{MANUAL[nm]}()" if nm in MANUAL else None
                assert ce, 'unhandled do-block: ' + nm
            else:
                ce = conv_expr(e)
            out.append(f"            ({n}, {r}, {ce}),")
        out.append("        ])")
        out.append("    }\n")
    out.append("}\n")
    return '\n'.join(out)

if __name__ == '__main__':
    d, outp = sys.argv[1], sys.argv[2]
    mods = [('en', os.path.join(d, 'Rules.hs'))]
    for loc in ['AU', 'BZ', 'CA', 'GB', 'IE', 'IN', 'JM', 'NZ', 'PH', 'TT', 'US', 'ZA']:
        mods.append((loc.lower() + '_', os.path.join(d, loc, 'Rules.hs')))
    hdr = "// @generated by tools/hs_holidays.py from Duckling.Time.EN.*.Rules; do not edit.\n#![allow(clippy::all, unused_imports)]\n\n"
    res = hdr
    for mod, path in mods:
        src = open(path).read()
        src = '\n'.join(l for l in src.split('\n') if not re.match(r'^\s*--', l) and l.strip())
        res += gen_module(mod.rstrip('_') if mod != 'in_' else 'in_', src)
    open(outp, 'w').write(res)
