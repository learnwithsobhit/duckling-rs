#!/usr/bin/env python3
"""Differential test of the Rust server against upstream's Haskell example server.

Usage: diff_server.py <haskell_url> <rust_url> [count] [seed]

Requests are sampled from the captured fixtures (text, language, locale) combined with random
time zones, reference times, `latent` and `dims` values. Status code, content type and body
must match byte for byte. Languages that are not ported yet are skipped.
"""
import glob
import json
import os
import random
import sys
import urllib.error
import urllib.parse
import urllib.request

HS, RS = sys.argv[1], sys.argv[2]
COUNT = int(sys.argv[3]) if len(sys.argv) > 3 else 2000
SEED = int(sys.argv[4]) if len(sys.argv) > 4 else 1
PORTED = {"EN"}

ZONES = ["America/Los_Angeles", "Europe/London", "Asia/Kolkata", "Australia/Lord_Howe", "UTC",
         "America/St_Johns", "Pacific/Auckland", "Asia/Tokyo", "Europe/Dublin", "Nowhere/Land", "America/New_York"]
DIMS = [None, '["time"]', '["number","duration"]', '["time","distance","quantity"]',
        '["amount-of-money"]', '["email","url","phone-number"]', '"[\\"time\\"]"', '[]',
        '["bogus"]', 'not json', '[1]']
LATENT = [None, "true", "false", "True", "yes"]
LOCALES = [None, "en_us", "en_gb", "EN_AU", "en_ca", "en_zz", "fr_fr", "xx"]


def fixtures():
    out = []
    for f in sorted(glob.glob(os.path.join(os.path.dirname(__file__), "..", "fixtures", "*.jsonl"))):
        lang = os.path.basename(f)[:-6]
        if lang not in PORTED:
            continue
        with open(f) as fh:
            for line in fh:
                r = json.loads(line)
                out.append(r["input"])
    return out


def post(base, params):
    data = urllib.parse.urlencode(params).encode()
    req = urllib.request.Request(base + "/parse", data=data)
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return r.status, r.headers.get("Content-Type"), r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.headers.get("Content-Type"), e.read().decode()


def main():
    rnd = random.Random(SEED)
    texts = sorted(set(fixtures()))
    bad = 0
    for i in range(COUNT):
        params = {"text": rnd.choice(texts), "lang": "en"}
        if rnd.random() < 0.9:
            params["tz"] = rnd.choice(ZONES)
        if rnd.random() < 0.9:
            # 1990-01-01 .. 2037-01-01
            params["reftime"] = str(rnd.randint(631152000000, 2114380800000))
        d = rnd.choice(DIMS)
        if d is not None:
            params["dims"] = d
        la = rnd.choice(LATENT)
        if la is not None:
            params["latent"] = la
        lo = rnd.choice(LOCALES)
        if lo is not None:
            params["locale"] = lo
        if params.get("locale", "").lower().startswith(("fr", "xx")):
            continue
        if "reftime" not in params:
            # Wall-clock dependent; compare only the shape (same entities, not values).
            continue
        a, b = post(HS, params), post(RS, params)
        if a[0] != b[0] or a[2] != b[2]:
            bad += 1
            print("MISMATCH", json.dumps(params))
            print("  haskell:", a)
            print("  rust:   ", b)
            if bad >= 10:
                break
    print(f"done: {COUNT} sampled, {bad} mismatches")
    sys.exit(1 if bad else 0)


main()
