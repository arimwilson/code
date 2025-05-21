#!/usr/bin/env python3
import os
import sys
import json
import argparse
import requests
from datetime import datetime, timedelta

GITHUB_API = "https://api.github.com"

def iso8601(dt_str, end=False):
    """
    Convert YYYY-MM-DD or full ISO8601 to ISO8601 timestamp.
    If only date is given, appends T00:00:00Z (or T23:59:59Z for end).
    """
    if 'T' in dt_str:
        return dt_str if dt_str.endswith('Z') else dt_str + 'Z'
    base = dt_str.strip()
    return f"{base}T{'23:59:59' if end else '00:00:00'}Z"


def get_commits(owner, repo, token, since, until, path=None):
    """
    List commits in [since, until], optionally filtered by path.
    """
    url = f"{GITHUB_API}/repos/{owner}/{repo}/commits"
    headers = {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json"
    }
    params = {"since": since, "until": until}
    if path:
        params["path"] = path

    commits = []
    page = 1
    while True:
        params["page"] = page
        resp = requests.get(url, headers=headers, params=params)
        # Handle common errors
        if resp.status_code == 404:
            sys.exit(f"Error: repo {owner}/{repo} not found (404). Check the spelling.")
        if resp.status_code == 403:
            sys.exit(f"Error: access forbidden (403). Verify your token’s scopes and org access.")
        resp.raise_for_status()
        data = resp.json()
        if not data:
            break
        commits.extend(data)
        page += 1
    return commits


def get_diff(owner, repo, sha, token):
    """
    Fetch full diff for a given commit SHA.
    """
    url = f"{GITHUB_API}/repos/{owner}/{repo}/commits/{sha}"
    headers = {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3.diff"
    }
    resp = requests.get(url, headers=headers)
    resp.raise_for_status()
    return resp.text


def main():
    parser = argparse.ArgumentParser(
        description="Fetch commits and diffs from GitHub in a date range."
    )
    parser.add_argument("--owner",            required=True, help="GitHub repo owner/org")
    parser.add_argument("--repo",             required=True, help="Repository name")
    parser.add_argument("--start-date",       required=True, help="Start date YYYY-MM-DD or ISO8601")
    parser.add_argument("--end-date",         required=True, help="End date   YYYY-MM-DD or ISO8601 (inclusive)")
    parser.add_argument("--path",                         help="Sub-directory or file path to filter")
    parser.add_argument("--exclude-authors", default="",  help="Comma-separated list of author logins or names to exclude")
    parser.add_argument(
        "--include-diffs", action=argparse.BooleanOptionalAction,
        default=True,
        help="Include diffs in the output (default: include diffs)."
    )
    args = parser.parse_args()

    token = os.getenv("GITHUB_TOKEN")
    if not token:
        sys.exit("Error: set GITHUB_TOKEN env var to your PAT (with repo scope)")

    since = iso8601(args.start_date, end=False)
    until = iso8601(args.end_date,   end=True)

    # Prepare exclusion set
    exclude_list = [a.strip() for a in args.exclude_authors.split(',') if a.strip()]

    commits = get_commits(args.owner, args.repo, token, since, until, args.path)
    if not commits:
        print("[]")
        return

    output = []
    for c in commits:
        sha          = c["sha"]
        info         = c["commit"]
        author_name  = info["author"]["name"]
        author_info = c.get("author")
        author_login = None
        if author_info is not None:
            author_login = author_info.get("login")
        date         = info["author"]["date"]
        message      = info["message"].strip()
        diff = None
        if args.include_diffs:
            diff = get_diff(args.owner, args.repo, sha, token)

        # Skip excluded authors
        if author_login in exclude_list or author_name in exclude_list:
            continue

        output.append({
            "sha":     sha,
            "author":  author_name,
            "login":   author_login,
            "date":    date,
            "message": message,
            "diff":    diff
        })

    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
