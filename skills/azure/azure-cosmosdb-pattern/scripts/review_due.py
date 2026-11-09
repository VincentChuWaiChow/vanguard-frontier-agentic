#!/usr/bin/env python3
"""Read-only content reminder. Explicit client-local date; never rewrites ledger."""
import argparse
from datetime import date
import json
from pathlib import Path


def due_status(ledger, as_of):
    due = date.fromisoformat(ledger["next_full_review_due"])
    return {"as_of": as_of.isoformat(), "due": due.isoformat(),
            "status": "overdue" if as_of > due else "due" if as_of == due else "not-due",
            "cadence": "June/December", "next_due_advances_automatically": False,
            "notification": "in-session-only", "last_full_review": ledger["last_full_review"]}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--as-of", required=True, type=date.fromisoformat)
    args = p.parse_args()
    ledger = json.loads((Path(__file__).resolve().parents[1] / "assets/content-review.json").read_text())
    print(json.dumps(due_status(ledger, args.as_of), indent=2))


if __name__ == "__main__":
    main()
