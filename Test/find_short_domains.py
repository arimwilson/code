#!/usr/bin/env python3
"""
Find short pronounceable 2‑3 letter domains that are still cheap.

All runtime options live in `.env`:
  GODADDY_API_KEY, GODADDY_API_SECRET  – registrar creds
  MAX_PRICE_USD                        – price ceiling (float)
  USE_WORDFREQ                         – 1/0 for English‑word ranking
  MIN_VOWELS, MAX_CONSONANT_STREAK     – phonetic heuristics
  TLDS                                 – comma‑sep short TLDs
  PATTERNS_2, PATTERNS_3               – comma‑sep label patterns
  CONCURRENCY_LIMIT, HTTP_TIMEOUT      – API tuning
"""
import asyncio, itertools, math, os
from dataclasses import dataclass
from typing import Iterable, List, Optional

import httpx
from dotenv import load_dotenv
from tqdm.asyncio import tqdm_asyncio

# ────────────────────────────────────────────────────────────────────────
# 1.  CONFIG  ─ picks up everything from .env (with sane fallbacks)
# ────────────────────────────────────────────────────────────────────────
load_dotenv()                                  # load .env from current dir

GODADDY_API_KEY    = os.getenv("GODADDY_API_KEY")
GODADDY_API_SECRET = os.getenv("GODADDY_API_SECRET")

MAX_PRICE_USD      = float(os.getenv("MAX_PRICE_USD", "10"))
USE_WORDFREQ       = os.getenv("USE_WORDFREQ", "1") != "0"

MIN_VOWELS         = int(os.getenv("MIN_VOWELS", "2"))
MAX_CONS_STREAK    = int(os.getenv("MAX_CONSONANT_STREAK", "2"))

TLDS               = os.getenv("TLDS", "ai,io,ly,sh").split(",")
PATTERNS_2         = os.getenv("PATTERNS_2", "CV,VC,VV").split(",")
PATTERNS_3         = os.getenv("PATTERNS_3", "CVC,VCV,CVV,VVC").split(",")

CONCURRENCY_LIMIT  = int(os.getenv("CONCURRENCY_LIMIT", "30"))
HTTP_TIMEOUT       = int(os.getenv("HTTP_TIMEOUT", "10"))

# static alphabets
VOWELS      = "aeiou"
CONSONANTS  = ''.join(c for c in "abcdefghijklmnopqrstuvwxyz" if c not in VOWELS)

# wordfreq (optional)
if USE_WORDFREQ:
    from wordfreq import zipf_frequency

def freq_score(word: str) -> float:
    """Lower is better; +inf means not ranked/disabled."""
    if not USE_WORDFREQ:
        return float("inf")
    # higher frequency → lower (better) score
    return -zipf_frequency(word, "en")

# ────────────────────────────────────────────────────────────────────────
# 2.  PRONUNCIATION HELPERS
# ────────────────────────────────────────────────────────────────────────
def has_ok_phonetics(s: str) -> bool:
    if sum(ch in VOWELS for ch in s) < MIN_VOWELS:
        return False
    streak = 0
    for ch in s:
        if ch in CONSONANTS:
            streak += 1
            if streak > MAX_CONS_STREAK:
                return False
        else:
            streak = 0
    if "q" in s and "qu" not in s:
        return False
    return True

def generate_labels() -> List[str]:
    pools = {"V": VOWELS, "C": CONSONANTS}
    labels: set[str] = set()

    def expand(patterns: Iterable[str]):
        for pat in patterns:
            letters = [pools.get(p, pools["C"]+pools["V"]) for p in pat]
            for tup in itertools.product(*letters):
                lbl = ''.join(tup)
                if has_ok_phonetics(lbl):
                    labels.add(lbl)

    expand(PATTERNS_2)
    expand(PATTERNS_3)
    return sorted(labels)

def label_tld_ok(label: str, tld: str) -> bool:
    combo = label + tld
    return has_ok_phonetics(combo)

# ────────────────────────────────────────────────────────────────────────
# 3.  API
# ────────────────────────────────────────────────────────────────────────
API_BASE = "https://api.godaddy.com/v1"

@dataclass
class Candidate:
    domain: str
    price: float
    score: float  # wordfreq score (lower = better)

async def godaddy_price(client: httpx.AsyncClient, domain: str) -> Optional[float]:
    try:
        r = await client.get(f"{API_BASE}/domains/available", params={"domain": domain})
        if r.status_code == 404:
            return None
        r.raise_for_status()
        data = r.json()
        if not data.get("available"):
            return None
        cents = data.get("price")
        return round(cents / 100.0, 2) if cents else None
    except (httpx.HTTPStatusError, ValueError):
        return None

async def main() -> None:
    if not GODADDY_API_KEY or not GODADDY_API_SECRET:
        raise SystemExit("🔑  Add GODADDY_API_KEY & GODADDY_API_SECRET to your .env")

    labels = generate_labels()
    combos = [(lbl, tld) for lbl, tld in itertools.product(labels, TLDS)
              if label_tld_ok(lbl, tld)]
    print(f"{len(combos):,} label/TLD combos after phonetic filter.")

    headers = {"Authorization": f"sso-key {GODADDY_API_KEY}:{GODADDY_API_SECRET}",
               "Accept": "application/json"}

    found: List[Candidate] = []
    async with httpx.AsyncClient(headers=headers, timeout=HTTP_TIMEOUT) as client:
        sem = asyncio.Semaphore(CONCURRENCY_LIMIT)

        async def check(label: str, tld: str):
            async with sem:
                dom = f"{label}.{tld}"
                price = await godaddy_price(client, dom)
            if price is not None and price <= MAX_PRICE_USD:
                found.append(Candidate(dom, price, freq_score(label+tld)))

        tasks = [check(l, t) for l, t in combos]
        await tqdm_asyncio.gather(*tasks, desc="Querying GoDaddy", total=len(tasks))

    if not found:
        print("😢  Nothing under your price cap.")
        return

    found.sort(key=lambda c: (c.score, c.price, len(c.domain)))

    print(f"\n=== Available under ${MAX_PRICE_USD:.2f} ===")
    for c in found:
        rank = f"{c.score:+.2f}" if USE_WORDFREQ else "-"
        print(f"{c.domain:15s}  ${c.price:<5}   score {rank}")

if __name__ == "__main__":
    asyncio.run(main())

