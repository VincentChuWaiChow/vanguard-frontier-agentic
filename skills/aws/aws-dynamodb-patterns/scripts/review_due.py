#!/usr/bin/env python3
"""Read-only June/December content reminder. No scheduler, network or file writes."""
import argparse
from datetime import date
import json
from pathlib import Path

SCHEDULE = Path(__file__).resolve().parents[1] / "assets/content-review.json"


def next_window_after(completed):
    """First June/December window strictly after a genuinely completed review."""
    for year in (completed.year, completed.year + 1):
        for month in (6, 12):
            candidate = date(year, month, 1)
            if candidate > completed:
                return candidate
    raise ValueError("Cannot calculate next review window")


def review_status(schedule, as_of):
    if schedule["review_months"] != [6, 12]:
        raise ValueError("Review months must be June and December")
    due = date.fromisoformat(schedule["next_review_due"])
    if due.month not in (6, 12) or due.day != 1:
        raise ValueError("Review due date must start a June/December window")
    completed = schedule["last_full_review"]
    if completed is not None:
        completed = date.fromisoformat(completed)
        if next_window_after(completed) != due:
            raise ValueError("Next due date must match the completed full review")
    targeted = schedule["last_targeted_review"]
    if targeted is not None:
        date.fromisoformat(targeted)
    if as_of < due:
        status = "not_due"
    elif (as_of.year, as_of.month) == (due.year, due.month):
        status = "due"
    else:
        status = "overdue"
    return {"as_of": as_of.isoformat(), "status": status,
            "review_months": schedule["review_months"], "next_review_due": due.isoformat(),
            "last_full_review": schedule["last_full_review"],
            "last_targeted_review": targeted, "background_notification": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--as-of", required=True, type=date.fromisoformat,
                        help="User/client-local date, YYYY-MM-DD; do not infer host timezone")
    args = parser.parse_args()
    try:
        result = review_status(json.loads(SCHEDULE.read_text()), args.as_of)
    except (ValueError, KeyError, OSError) as error:
        parser.error(str(error))
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
