#!/usr/bin/env python3
"""Read-only reminder. Require the actual client-local date; never roll forward."""
import argparse
import datetime as dt
import json
from pathlib import Path


def review_status(as_of, ledger):
    day = dt.date.fromisoformat(as_of)
    due = dt.date.fromisoformat(ledger["next_review"])
    if due.month not in {6, 12} or due.day != 1:
        raise ValueError("review ledger must use June/December 1")
    return {"as_of": day.isoformat(), "next_review": due.isoformat(),
            "status": "overdue" if day > due else "due" if day == due else "not-due",
            "schedule": "June/December", "writes": False,
            "notice": "Reminder only. Partial edits do not complete a full content audit."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--as-of", required=True, help="Actual client-local YYYY-MM-DD")
    args = parser.parse_args()
    try:
        ledger = json.loads((Path(__file__).resolve().parents[1] / "assets/content-review.json").read_text())
        print(json.dumps(review_status(args.as_of, ledger), indent=2))
    except (ValueError, KeyError) as exc:
        parser.error(str(exc))


if __name__ == "__main__":
    main()
