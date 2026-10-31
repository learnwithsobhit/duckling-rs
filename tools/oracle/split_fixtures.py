#!/usr/bin/env python3
"""Dedupe the raw Haskell dump (fixtures/raw.jsonl) and split it into one file per language:
fixtures/<LANG>.jsonl. Lines are kept byte-for-byte (the parity test compares raw JSON text)."""
import json, os, sys, collections

root = os.path.join(os.path.dirname(__file__), "..", "..", "fixtures")
raw = os.path.join(root, "raw.jsonl")
seen = set()
by_lang = collections.defaultdict(list)
n = 0
with open(raw, encoding="utf-8") as f:
    for line in f:
        line = line.rstrip("\n")
        if not line or line in seen:
            continue
        seen.add(line)
        n += 1
        by_lang[json.loads(line)["lang"]].append(line)
for lang, lines in by_lang.items():
    lines.sort()
    with open(os.path.join(root, f"{lang}.jsonl"), "w", encoding="utf-8") as out:
        out.write("\n".join(lines) + "\n")
print(f"{n} unique records in {len(by_lang)} languages")
for lang in sorted(by_lang):
    print(f"  {lang}: {len(by_lang[lang])}")
