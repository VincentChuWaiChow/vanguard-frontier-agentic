# Kyverno runtime semantic acceptance specifications

These scenarios have not been executed. They describe behavior that only a running API server and Kyverno controller can establish (webhook failure handling, controller-side exception expiry, real UPDATE old/new pairs, reconciliation after a revert, registry dependency errors, and where denied writes are recorded). Offline `kyverno test` results cannot settle any of them.

Run them only in a separately authorized, disposable environment with pinned versions. Record the evidence listed under `capture`, change `status` only with the recorded results, and keep failures and unresolved results visible.
