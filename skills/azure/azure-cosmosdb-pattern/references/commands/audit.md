# Audit dispatcher

Default **offline**. Load exactly one [offline](audit/offline.md) or
[live](audit/live.md) procedure according to the selected mode. Do not open both.
Scope only selected features. Unknown or unselected settings are UNVERIFIED, not
compliant/disabled. Selection itself grants no permission to access Azure.
Report severity, evidence location/time, expected vs observed behavior, narrow
recommendation, missing evidence and PASS/MISS/UNVERIFIED; never auto-remediate.
