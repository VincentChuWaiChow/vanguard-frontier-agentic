#!/usr/bin/env python3
"""Select lazy skill instructions. Never read their contents or execute AWS work."""
import argparse
import json
import sys

FEATURES = {
    "core": ["references/modeling.md", "references/runtime-contract.md"],
    "indexes": ["references/lsi-vs-gsi.md"],
    "encryption": ["references/encryption.md"],
    "iam": ["references/iam-and-access.md"],
    "tags": ["references/iam-and-access.md"],
    "resource-policy": ["references/iam-and-access.md"],
    "ttl": ["references/runtime-contract.md", "references/commands/aws-cli.md"],
    "pitr": ["references/recovery-and-replicas.md"],
    "streams": ["references/streams-and-dax.md"],
    "dax": ["references/streams-and-dax.md"],
    "replicas": ["references/recovery-and-replicas.md"],
    "vectors": ["references/data-types-and-vectors.md"],
    "embeddings": ["references/embedding-integrations.md"],
    "monitoring": ["references/monitoring.md"],
}
MODE_GUIDES = {
    "select": ["references/commands/select.md"],
    "design": ["references/commands/design.md"],
    "refactor": ["references/commands/refactor.md"],
    "backup": ["references/commands/backup.md", "references/commands/recovery-target.md"],
    "restore": ["references/commands/restore.md", "references/commands/recovery-target.md"],
    "audit": ["references/commands/audit.md"],
    "performance-test": ["references/commands/performance-test.md"],
    "feature-plan": ["references/commands/feature-plan.md"],
}
TOGGLES = {"ttl", "pitr", "streams"}
ENABLE_OR_CONFIGURE = {"indexes", "vectors", "dax", "replicas", "monitoring"}
CONFIGURE_ONLY = {"encryption", "iam", "tags", "resource-policy", "embeddings"}


def positive_int(value):
    try:
        number = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("Use a positive integer") from error
    if number <= 0:
        raise argparse.ArgumentTypeError("Limits must be positive, never unbounded")
    return number


def build_plan(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv[:1] == ["/aws-dynamodb-patterns"]:
        argv = argv[1:]
    if not argv:
        argv = ["help"]
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="mode", required=True)
    commands.add_parser("help", help="List commands and selectable features as JSON")
    for name in MODE_GUIDES:
        command = commands.add_parser(name)
        command.add_argument("--features", default="pitr" if name in {"backup", "restore"} else "core",
                             required=name == "feature-plan",
                             help="Comma-separated feature IDs; selection lasts one invocation")
        if name in {"backup", "restore"}:
            command.add_argument("--method", choices=["pitr", "on-demand"], default="pitr")
        elif name == "audit":
            source = command.add_mutually_exclusive_group()
            source.add_argument("--mode", dest="audit_mode", choices=["offline", "live"])
            source.add_argument("--evidence", choices=["repo", "live-metadata"],
                                help="Compatibility spelling; use --mode for new invocations")
        elif name == "performance-test":
            command.add_argument("--environment", choices=["local", "nonproduction"], default="local")
            command.add_argument("--duration-seconds", type=positive_int, default=30)
            command.add_argument("--max-requests", type=positive_int, default=1000)
            command.add_argument("--max-concurrency", type=positive_int, default=4)
        elif name == "feature-plan":
            command.add_argument("--action", choices=["enable", "disable", "configure"], required=True)
    args = parser.parse_args(argv)
    if args.mode == "help":
        return {"commands": list(MODE_GUIDES), "features": list(FEATURES),
                "read": ["references/commands/help.md"], "execution": "none"}

    features = list(dict.fromkeys(value.strip() for value in args.features.split(",")))
    unknown = [value for value in features if value not in FEATURES]
    if unknown:
        parser.error(f"Unknown or empty features: {unknown}; use the help command")
    if args.mode == "feature-plan":
        for feature in features:
            allowed = set()
            if feature in TOGGLES:
                allowed = {"enable", "disable", "configure"}
            elif feature in ENABLE_OR_CONFIGURE:
                allowed = {"enable", "configure"}
            elif feature in CONFIGURE_ONLY:
                allowed = {"configure"}
            if args.action not in allowed:
                parser.error(f"{feature} does not support generic {args.action}; review its specific lifecycle")

    selected_paths = []
    for feature in features:
        if args.mode in {"backup", "restore"} and feature == "pitr":
            continue  # Selected recovery method already supplies this scope; no replica procedures.
        selected_paths.extend(FEATURES[feature])
    references = MODE_GUIDES[args.mode] + selected_paths
    if args.mode in {"backup", "restore"}:
        references.insert(2, f"references/commands/{args.mode}/{args.method}.md")
    elif args.mode == "audit":
        audit_mode = args.audit_mode or ("live" if args.evidence == "live-metadata" else "offline")
        references.insert(1, f"references/commands/audit/{audit_mode}.md")
    result = {"schema_version": 1, "mode": args.mode, "selected_features": features,
              "read": list(dict.fromkeys(references)), "execution": "none"}
    if args.mode in {"backup", "restore"}:
        result.update(method=args.method, change_approval_required=True)
        if args.mode == "restore":
            result["target_policy"] = "new-table-only"
    elif args.mode == "audit":
        result.update(audit_mode=audit_mode, evidence="repo" if audit_mode == "offline" else "live-metadata",
                      live_read_approval_required=audit_mode == "live")
    elif args.mode == "performance-test":
        result.update(environment=args.environment, run_approval_required=True,
                      limits={"duration_seconds": args.duration_seconds,
                              "max_requests": args.max_requests, "max_concurrency": args.max_concurrency})
    elif args.mode == "feature-plan":
        result.update(action=args.action, change_approval_required=True)
    return result


if __name__ == "__main__":
    print(json.dumps(build_plan(), indent=2))
