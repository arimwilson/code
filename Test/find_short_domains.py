#!/usr/bin/env python3
"""
Find short pronounceable 2‑3 letter domain names with cheap prices.
Heuristics:
  • label length 2‑3, short TLDs (≤3 letters)
  • combined “word” (label+tld) must look pronounceable and, if enabled,
    be a reasonably common English‑looking word via wordfreq.
"""
import asyncio, itertools, os, math
from dataclasses import dataclass
from typing import Iterable, List, Optional

import httpx
from dotenv import load_dotenv
from tqdm.asyncio import tqdm_asyncio

# -------------- CONFIG -------------------------------------------------
SHORT_CHEAPISH_TLDS = [
    # mostly 2‑letter ccTLDs + a few 3‑letter bargains
    "ai", "am", "fm", "gg", "gl", "gs", "io", "is", "it",
    "la", "ly", "me", "sh", "so", "to", "tv", "vc", "ws",
    "zip", "lol", "app", "run"  # 3‑letter, still “shortish”
]

VOWELS = "aeiou"
CONSONANTS = ''.join(ch for ch in "abcdefghijklmnopqrstuvwxyz" if ch not in VOWELS)
PATTERNS_2 = ["CV", "VC", "VV"]          # restrict to smoother sounding 2‑letter combos
PATTERNS_3 = ["CVC", "VCV", "CVV", "VVC"]

API_BASE = "https://api.godaddy.com/v1"
WORD_FREQ_LIMIT = 5e4  # only keep words in the 50k most common bucket
# ----------------------------------------------------------------------

# -------- optional wordfreq scoring -----------------------------------
USE_WORDFREQ = os.getenv("USE_WORDFREQ", "1") != "0"
if USE_WORDFREQ:
    from wordfreq import zipf_frequency

def is_common_word(word: str) -> bool:
    """Returns True if word is among ~50k most common English word‑forms."""
    if not USE_WORDFREQ:
        return True
    # zipf_frequency ≈ log10(freq per billion). “4” => ~1/10 000, “3” => 1/100 000
    return zipf_frequency(word, "en") >= 3.0
# ----------------------------------------------------------------------

def has_ok_phonetics(s: str) -> bool:
    """Lo‑fi pronounceability heuristic."""
    if sum(ch in VOWELS for ch in s) < 2:          # need at least 2 vowels overall
        return False
    streak = 0
    for ch in s:
        if ch in CONSONANTS:
            streak += 1
            if streak > 2:
                return False
        else:
            streak = 0
    if "q" in s and "qu" not in s:
        return False
    return True

def generate_labels() -> List[str]:
    pools = {"V": VOWELS, "C": CONSONANTS}
    labels = set()
    for pat in PATTERNS_2 + PATTERNS_3:
        letters = [pools[p] for p in pat]
        for tup in itertools.product(*letters):
            lbl = ''.join(tup)
            if has_ok_phonetics(lbl):
                labels.add(lbl)
    return sorted(labels)

@dataclass
class Candidate:
    domain: str
    price: Optional[float]  # USD
    word_rank: float        # lower is better (∞ means not found)

def combined_ok(label: str, tld: str) -> bool:
    combo = label + tld   # “goo” + “gl” → “googl”
    return has_ok_phonetics(combo) and is_common_word(combo)

async def godaddy_check(client: httpx.AsyncClient, domain: str) -> Optional[float]:
    try:
        r = await client.get(f"{API_BASE}/domains/available", params={"domain": domain})
        r.raise_for_status()
        data = r.json()
        if not data.get("available"):
            return None
        cents = data.get("price")
        return round(cents / 100.0, 2) if cents else None
    except httpx.HTTPStatusError:
        return None

async def main():
    load_dotenv()
    key, secret = os.getenv("GODADDY_API_KEY"), os.getenv("GODADDY_API_SECRET")
    max_price = float(os.getenv("MAX_PRICE_USD", "10"))
    tlds = os.getenv("TLDS", ",".join(SHORT_CHEAPISH_TLDS)).split(",")
    labels = generate_labels()

    # Early phonetic filter on label+TLD to cut API traffic
    combos = [(lbl, tld) for lbl, tld in itertools.product(labels, tlds)
              if combined_ok(lbl, tld)]
    print(f"{len(combos):,} label/TLD combos pass pronunciation screen.")

    headers = {"Authorization": f"sso-key {key}:{secret}", "Accept": "application/json"}
    async with httpx.AsyncClient(headers=headers, timeout=10) as client:
        sem = asyncio.Semaphore(30)
        async def check(lbl, tld):
            dom = f"{lbl}.{tld}"
            async with sem:
                price = await godaddy_check(client, dom)
            if price is None or price > max_price:
                return None
            word_rank = -math.inf if not USE_WORDFREQ else zipf_frequency(lbl+tld, "en") * -1
            return Candidate(dom, price, word_rank)

        tasks = [check(l, t) for l, t in combos]
        found: List[Candidate] = []
        for coro in tqdm_asyncio.as_completed(tasks, total=len(tasks), desc="Querying"):
            res = await coro
            if res:
                found.append(res)

    found.sort(key=lambda c: (c.word_rank, c.price, len(c.domain)))
    print(f"\n=== Under ${max_price:.2f} & ranked ===")
    for c in found:
        print(f"{c.domain:15s}  ${c.price:<5}   (score {c.word_rank:+.2f})")

if __name__ == "__main__":
    asyncio.run(main())

