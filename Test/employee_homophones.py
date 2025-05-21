#!/usr/bin/env python3
"""
employee_homophones.py
======================

Read a CSV file that contains *one column of employee names*, group people
whose **first names sound alike** (Aaron ≈ Erin, Jon ≈ John, etc.) using
**Double Metaphone**, and print either

    • a tidy, human-readable report  (default), or
    • the raw Python list-of-tuples structure (`--raw`).

-----------
Installation
-----------
This version depends on the **jellyfish** library:

    pip install jellyfish

------------
Usage examples
------------
Pretty report (names in column 0)::

    python employee_homophones.py employees.csv

Pretty report, names in column 2::

    python employee_homophones.py employees.csv --col 2

Raw Python structure instead of formatted output::

    python employee_homophones.py employees.csv --raw
"""
from __future__ import annotations

import argparse
import csv
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

# ──────────────────────────────────────────────────────────────────────────
#  Phonetic helper  –  Double Metaphone
# ──────────────────────────────────────────────────────────────────────────
try:
    from metaphone import doublemetaphone          # pypi package: metaphone
except ImportError:                                # pragma: no cover
    raise SystemExit(
        "This script now needs the 'metaphone' package.\n"
        "Install it with:\n\n    pip install metaphone\n"
    )

def phonetic_key(word: str) -> str:
    """
    Return the primary Double-Metaphone code for *word*.
    Falls back to the alternate code if the primary is empty.
    Returns '' when the name has no letters.
    """
    word = re.sub(r"[^A-Za-z]", "", word)
    if not word:
        return ""
    primary, alternate = doublemetaphone(word)
    return primary or alternate

# ──────────────────────────────────────────────────────────────────────────
#  CSV ingestion
# ──────────────────────────────────────────────────────────────────────────
def read_names(csv_path: Path, col: int = 0) -> list[str]:
    """
    Read *csv_path*, return a list[str] of the cells in column *col*.

    • Skips completely blank rows.
    • Automatically skips the first row if it looks like a header
      (case-insensitive match for 'name').
    """
    names: list[str] = []
    with csv_path.open(newline="", encoding="utf-8") as f:
        rdr = csv.reader(f)
        for row in rdr:
            if not row or col >= len(row):
                continue
            cell = row[col].strip()
            if not cell:
                continue
            if not names and re.search(r"name", cell, re.I):
                # First non-blank cell of first row looks like header → skip.
                continue
            names.append(cell)
    return names


# ──────────────────────────────────────────────────────────────────────────
#  Grouping logic
# ──────────────────────────────────────────────────────────────────────────
def name_categories(full_names: list[str]) -> list[tuple[str, int, list[str]]]:
    """
    Group employees by homophonic first names.

    Returns
    -------
    list[ tuple[ representative-spelling:str,
                 total_count:int,
                 list_of_full_names:list[str] ] ]
    sorted by *total_count* descending.
    """
    firsts = [n.split()[0] for n in full_names]  # “Aaron Girard” → “Aaron”
    first_counts: Counter[str] = Counter(firsts)

    buckets: defaultdict[str, list[str]] = defaultdict(list)
    for full, first in zip(full_names, firsts):
        key = phonetic_key(first)
        if not key:                # skip rows that fail to yield a code
            continue
        buckets[key].append(full)

    results: list[tuple[str, int, list[str]]] = []
    for full_list in buckets.values():
        variants = {n.split()[0] for n in full_list}
        # Representative spelling: most frequent variant, tie → alphabetical
        rep = max(variants, key=lambda v: (first_counts[v], v))
        total = sum(first_counts[v] for v in variants)
        # Sort full names by popularity of *first name*, then alphabetically
        full_list.sort(key=lambda n: (-first_counts[n.split()[0]], n))
        results.append((rep, total, full_list))

    # Descending by employee count
    results.sort(key=lambda t: -t[1])
    return results


# ──────────────────────────────────────────────────────────────────────────
#  Pretty printer
# ──────────────────────────────────────────────────────────────────────────
def pretty_print(groups: list[tuple[str, int, list[str]]]) -> None:
    """
    Produce a tidy, indented, human-readable report, e.g.:

        Aaron (17)
            • Aaron Girard
            • Erin Ndrio
            • Aron Smith

        John (12)
            • John Smith
            • Jon Smyth
            • Johnny Doe
    """
    for rep, total, names in groups:
        print(f"{rep} ({total})")
        for n in names:
            print(f"    • {n}")
        print()  # blank line between buckets


# ──────────────────────────────────────────────────────────────────────────
#  CLI entry point
# ──────────────────────────────────────────────────────────────────────────
def main() -> None:
    ap = argparse.ArgumentParser(description="Group employees by phonetic first names.")
    ap.add_argument("csv", type=Path, help="CSV file containing employee names")
    ap.add_argument(
        "--col",
        type=int,
        default=0,
        help="Zero-based column index that holds the names (default 0)",
    )
    ap.add_argument(
        "--raw",
        action="store_true",
        help="Print raw Python list instead of a pretty report",
    )
    args = ap.parse_args()

    names = read_names(args.csv, args.col)
    if not names:
        sys.exit("No names found — check file content and column index.")

    groups = name_categories(names)
    if args.raw:
        print(groups)
    else:
        pretty_print(groups)


if __name__ == "__main__":
    main()
