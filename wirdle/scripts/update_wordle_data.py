#!/usr/bin/env python3
"""Download Wordle word lists and dated past-solution data.

The output matches the files consumed by this crate:

  wordle-data/allowed_guesses.txt
  wordle-data/candidate_solutions.txt
  wordle-data/past_solutions.json
  wordle-data/editorial_overrides.json
"""

from __future__ import annotations

import json
import re
import sys
import tempfile
import urllib.request
from datetime import datetime
from html import unescape
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "wordle-data"

ANSWERS_URL = "https://raw.githubusercontent.com/Roy-Orbison/wordle-guesses-answers/main/answers.txt"
GUESSES_URL = "https://raw.githubusercontent.com/Roy-Orbison/wordle-guesses-answers/main/guesses.txt"
PAST_SOLUTIONS_URL = "https://wordle.today/answers"

WORD_RE = re.compile(r"^[a-z]{5}$")


def fetch_text(url: str) -> str:
    request = urllib.request.Request(url, headers={"User-Agent": "wordle-api-data-updater/1.0"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read().decode("utf-8")


def clean_words(text: str) -> list[str]:
    words: set[str] = set()
    for line in text.splitlines():
        word = line.strip().lower()
        if not word or word.startswith("#"):
            continue
        if not WORD_RE.match(word):
            raise ValueError(f"invalid word from source: {word!r}")
        words.add(word)
    return sorted(words)


def fetch_all_past_solutions() -> list[dict[str, object]]:
    html = fetch_text(PAST_SOLUTIONS_URL)
    text = unescape(re.sub(r"<[^>]+>", " ", html))
    row_re = re.compile(
        r"([A-Z][a-z]+,\s+[A-Z][a-z]+\s+\d{1,2},\s+\d{4})\s+"
        r"Wordle\s+([\d,]+)\s+([A-Z]{5})"
    )
    rows: list[dict[str, object]] = []
    for date_text, puzzle_text, answer_text in row_re.findall(text):
        answer = answer_text.lower()
        if not WORD_RE.match(answer):
            raise ValueError(f"invalid solution from source: {answer!r}")
        rows.append(
            {
                "date": datetime.strptime(date_text, "%A, %B %d, %Y").date().isoformat(),
                "puzzle_number": int(puzzle_text.replace(",", "")),
                "solution": answer,
                "source": "wordle.today",
                "is_repeat": False,
            }
        )

    rows.sort(key=lambda row: int(row["puzzle_number"]))
    if not rows:
        raise ValueError("no past solutions parsed from wordle.today")
    seen: set[str] = set()
    for row in rows:
        solution = str(row["solution"])
        row["is_repeat"] = solution in seen
        seen.add(solution)
    return rows


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=path.parent, delete=False) as tmp:
        tmp.write(content)
        tmp_path = Path(tmp.name)
    tmp_path.replace(path)


def main() -> int:
    answers = clean_words(fetch_text(ANSWERS_URL))
    guesses = clean_words(fetch_text(GUESSES_URL))
    past_solutions = fetch_all_past_solutions()

    past_words = {str(row["solution"]) for row in past_solutions}
    allowed_guesses = sorted(set(guesses) | set(answers) | past_words)
    candidate_solutions = sorted(set(answers) | past_words)

    write_text(DATA_DIR / "allowed_guesses.txt", "\n".join(allowed_guesses) + "\n")
    write_text(DATA_DIR / "candidate_solutions.txt", "\n".join(candidate_solutions) + "\n")
    write_text(
        DATA_DIR / "past_solutions.json",
        json.dumps(past_solutions, indent=2, sort_keys=False) + "\n",
    )
    write_text(
        DATA_DIR / "editorial_overrides.json",
        json.dumps(
            {
                "add_candidate_solutions": sorted(past_words - set(answers)),
                "remove_candidate_solutions": [],
                "word_priors": {},
            },
            indent=2,
        )
        + "\n",
    )

    print(f"allowed_guesses={len(allowed_guesses)}")
    print(f"candidate_solutions={len(candidate_solutions)}")
    print(f"past_solutions={len(past_solutions)}")
    if past_solutions:
        print(f"past_solution_range={past_solutions[0]['date']}..{past_solutions[-1]['date']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
