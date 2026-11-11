#!/usr/bin/env python3
"""Return selected paths only; never read references or execute OCI operations."""
import argparse
import json
import sys

FEATURES = {
    "core": ["references/modeling.md", "references/runtime-contract.md"],
    "indexes": ["references/indexes.md"],
    "iam": ["references/access.md"],
    "encryption": ["references/encryption.md"],
    "ttl": ["references/ttl.md"],
    "replicas": ["references/replicas.md"],
    "monitoring": ["references/monitoring.md"],
    "vectors": ["references/types-and-vectors.md"],
    "embeddings": ["references/embedding-integrations.md"],
    "backup": ["references/recovery-boundary.md"],
}
COMMANDS = ["help", "select", "design", "refactor", "audit", "performance-test", "feature-plan", "backup", "restore"]
ACTIONS = {
    "indexes": {"enable", "configure"}, "iam": {"configure"},
    "encryption": {"configure"}, "ttl": {"enable", "disable", "configure"},
    "replicas": {"enable", "configure"}, "monitoring": {"configure"},
    "embeddings": {"configure"}, "backup": {"configure"},
}


def positive(value):
    try:
        number = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("positive integer required") from exc
    if number <= 0:
        raise argparse.ArgumentTypeError("limits must be positive")
    return number


def build_plan(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv[:1] == ["/oci-nosql-pattern"]:
        argv = argv[1:]
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    sub = parser.add_subparsers(dest="command", required=True)
    for name in COMMANDS:
        p = sub.add_parser(name, allow_abbrev=False)
        if name == "help":
            continue
        p.add_argument("--features", default="backup" if name in {"backup", "restore"} else "core",
                       required=name == "feature-plan")
        if name == "audit":
            p.add_argument("--mode", dest="audit_mode", choices=["offline", "live"], default="offline")
        if name in {"backup", "restore"}:
            p.add_argument("--method", choices=["migrator", "sdk"], default="migrator")
        if name == "feature-plan":
            p.add_argument("--action", choices=["enable", "disable", "configure"], required=True)
        if name == "performance-test":
            p.add_argument("--environment", choices=["local", "nonproduction"], default="local")
            p.add_argument("--duration-seconds", type=positive, default=30)
            p.add_argument("--max-requests", type=positive, default=500)
            p.add_argument("--max-concurrency", type=positive, default=2)
    args = parser.parse_args(argv or ["help"])
    if args.command == "help":
        return {"schema_version": 1, "mode": "help", "commands": COMMANDS,
                "features": list(FEATURES), "read": ["references/commands/help.md"], "execution": "none"}
    features = list(dict.fromkeys(x.strip() for x in args.features.split(",")))
    if any(x not in FEATURES for x in features):
        parser.error("unknown/empty feature; PITR, Streams, DAX and native vector activation are not NDCS equivalents")
    if args.command == "feature-plan":
        for feature in features:
            if args.action not in ACTIONS.get(feature, set()):
                parser.error(f"{feature}: unsupported {args.action}; vectors are boundary guidance, not native ANN activation")
    paths = [f"references/commands/{args.command}.md"]
    result = {"schema_version": 1, "mode": args.command, "selected_features": features,
              "execution": "none"}
    if args.command == "audit":
        paths += [f"references/commands/audit/{args.audit_mode}.md"]
        result.update(audit_mode=args.audit_mode, live_read_approval_required=args.audit_mode == "live")
    if args.command in {"backup", "restore"}:
        paths += [f"references/commands/{args.command}/{args.method}.md"]
        result.update(method=args.method, change_approval_required=True)
        if args.command == "restore":
            result["target_policy"] = "new-table-only"
    for feature in features:
        paths.extend(FEATURES[feature])
    if args.command == "feature-plan":
        result.update(action=args.action, change_approval_required=True)
    if args.command == "performance-test":
        result.update(environment=args.environment, run_approval_required=True,
                      limits={"duration_seconds": args.duration_seconds, "max_requests": args.max_requests,
                              "max_concurrency": args.max_concurrency})
    result["read"] = list(dict.fromkeys(paths))
    return result


if __name__ == "__main__":
    print(json.dumps(build_plan(), indent=2))
