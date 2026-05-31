#!/usr/bin/env python3
"""Download Wordle word lists and dated past-solution data from NYT.

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
from argparse import ArgumentParser
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import date, datetime, timedelta
from html import unescape
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import urljoin

try:
    from zoneinfo import ZoneInfo
except ImportError:  # pragma: no cover - Python < 3.9 fallback.
    ZoneInfo = None  # type: ignore[assignment]


ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "wordle-data"

WORDLE_URL = "https://www.nytimes.com/games/wordle/index.html"
WORDLE_API_URL = "https://www.nytimes.com/svc/wordle/v2/{date}.json"
FIRST_WORDLE_DATE = date(2021, 6, 19)
FIRST_ANSWERS = ["cigar", "rebut", "sissy", "humph", "awake"]
NYT_SOURCE = "nytimes.com/svc/wordle/v2"

WORD_RE = re.compile(r"^[a-z]{5}$")
JS_ASSET_RE = re.compile(r"""["'](?P<url>(?:https://www\.nytimes\.com)?/games-assets/v2/[^"']+?\.js)["']""")
WORD_ARRAY_RE = re.compile(r'\[(?:"[a-z]{5}",){1000,}"[a-z]{5}"\]')
QUOTED_WORD_RE = re.compile(r'"([a-z]{5})"')


def fetch_text(url: str) -> str:
    request = urllib.request.Request(url, headers={"User-Agent": "wordle-api-data-updater/1.0"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read().decode("utf-8")


def fetch_json(url: str) -> dict[str, object]:
    return json.loads(fetch_text(url))


def current_wordle_date() -> date:
    if ZoneInfo is None:
        return date.today()
    return datetime.now(ZoneInfo("America/New_York")).date()


def parse_date(value: str) -> date:
    return datetime.strptime(value, "%Y-%m-%d").date()


def date_range(start: date, end: date) -> list[date]:
    if end < start:
        raise ValueError(f"end date {end.isoformat()} is before start date {start.isoformat()}")
    days = (end - start).days
    return [start + timedelta(days=offset) for offset in range(days + 1)]


def fetch_nyt_word_lists() -> tuple[list[str], list[str]]:
    html = unescape(fetch_text(WORDLE_URL))
    asset_urls = sorted(
        {
            urljoin(WORDLE_URL, match.group("url"))
            for match in JS_ASSET_RE.finditer(html)
            if "datadog" not in match.group("url")
        }
    )
    if not asset_urls:
        raise ValueError("no NYT Wordle JavaScript assets found")

    for asset_url in asset_urls:
        script = fetch_text(asset_url)
        for match in WORD_ARRAY_RE.finditer(script):
            words = QUOTED_WORD_RE.findall(match.group(0))
            if len(words) < 10_000:
                continue
            try:
                first_answer_idx = words.index(FIRST_ANSWERS[0])
            except ValueError:
                continue
            if words[first_answer_idx : first_answer_idx + len(FIRST_ANSWERS)] != FIRST_ANSWERS:
                continue
            validate_words(words, f"word list in {asset_url}")
            if len(set(words)) != len(words):
                raise ValueError(f"duplicate words found in {asset_url}")
            return sorted(words), sorted(words[first_answer_idx:])

    raise ValueError("no NYT Wordle word list found in JavaScript assets")


def validate_words(words: list[str], source: str) -> None:
    for word in words:
        if not WORD_RE.match(word):
            raise ValueError(f"invalid word from {source}: {word!r}")


def fetch_solution(day: date) -> dict[str, object]:
    url = WORDLE_API_URL.format(date=day.isoformat())
    try:
        data = fetch_json(url)
    except HTTPError as err:
        raise ValueError(f"failed to fetch {url}: HTTP {err.code}") from err

    answer = str(data.get("solution", "")).lower()
    if not WORD_RE.match(answer):
        raise ValueError(f"invalid solution from {url}: {answer!r}")

    print_date = str(data.get("print_date") or day.isoformat())
    puzzle_number = data.get("days_since_launch") or data.get("id")
    if not isinstance(puzzle_number, int):
        raise ValueError(f"missing puzzle number from {url}")

    return {
        "date": print_date,
        "puzzle_number": puzzle_number,
        "solution": answer,
        "source": NYT_SOURCE,
        "is_repeat": False,
    }


def fetch_all_past_solutions(through: date) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    days = date_range(FIRST_WORDLE_DATE, through)
    with ThreadPoolExecutor(max_workers=8) as executor:
        futures = {executor.submit(fetch_solution, day): day for day in days}
        for future in as_completed(futures):
            rows.append(future.result())

    rows.sort(key=lambda row: str(row["date"]))
    if not rows:
        raise ValueError("no past solutions fetched from NYT")
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


def parse_args() -> tuple[date, bool]:
    parser = ArgumentParser()
    parser.add_argument(
        "--through",
        default=current_wordle_date().isoformat(),
        help="last NYT Wordle print date to fetch, in YYYY-MM-DD format",
    )
    parser.add_argument(
        "--skip-past-solutions",
        action="store_true",
        help="refresh word lists only, leaving past_solutions.json unchanged",
    )
    args = parser.parse_args()
    return parse_date(args.through), args.skip_past_solutions


def main() -> int:
    through, skip_past_solutions = parse_args()
    guesses, answers = fetch_nyt_word_lists()
    past_solutions = (
        json.loads((DATA_DIR / "past_solutions.json").read_text(encoding="utf-8"))
        if skip_past_solutions
        else fetch_all_past_solutions(through)
    )

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
