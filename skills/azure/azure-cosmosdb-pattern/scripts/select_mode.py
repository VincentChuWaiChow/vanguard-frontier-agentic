#!/usr/bin/env python3
"""Local path selector only: no reference reads, network, subprocess or writes."""
import argparse
import json
import sys

FEATURES = {
    "core": ["references/modeling.md", "references/runtime-contract.md"],
    "indexes": ["references/indexing-and-capacity.md"],
    "capacity": ["references/indexing-and-capacity.md"],
    "iam": ["references/access-control.md"],
    "encryption": ["references/encryption.md"],
    "ttl": ["references/runtime-contract.md"],
    "backup": ["references/recovery.md"],
    "replication": ["references/replication.md"],
    "change-feed": ["references/change-feed-and-integrations.md"],
    "vectors": ["references/data-types-and-vectors.md"],
    "embeddings": ["references/embedding-integrations.md"],
    "monitoring": ["references/monitoring.md"],
    "provisioning": ["references/provisioning.md"],
}
COMMANDS = ("help", "select", "design", "refactor", "audit", "performance-test",
            "feature-plan", "backup", "restore")
ACTIONS = {
    "ttl": {"enable", "disable", "configure"},
    "indexes": {"configure"}, "capacity": {"configure"},
    "iam": {"configure"}, "encryption": {"configure"},
    "backup": {"configure"}, "replication": {"configure"},
    "change-feed": {"configure"}, "vectors": {"enable", "configure"},
    "embeddings": {"configure"}, "monitoring": {"configure"},
}


def positive(value):
    try:
        number = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("positive integer required") from error
    if number <= 0:
        raise argparse.ArgumentTypeError("positive finite cap required")
    return number


def build_plan(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv[:1] == ["/azure-cosmosdb-pattern"]:
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
            p.add_argument("--mode", choices=["offline", "live"], default="offline")
        elif name == "backup":
            p.add_argument("--method", choices=["continuous", "periodic"], default="continuous")
        elif name == "restore":
            p.add_argument("--method", choices=["continuous-new-account", "continuous-deleted-resource",
                                               "periodic-support"], default="continuous-new-account")
        elif name == "performance-test":
            p.add_argument("--environment", choices=["local", "nonproduction"], default="local")
            for option, default in [("duration-seconds", 30), ("max-requests", 500),
                                    ("max-concurrency", 2), ("max-ru", 1000)]:
                p.add_argument("--" + option, type=positive, default=default)
        elif name == "feature-plan":
            p.add_argument("--action", choices=["enable", "disable", "configure"], required=True)
    args = parser.parse_args(argv or ["help"])
    if args.command == "help":
        return {"schema_version": 1, "commands": list(COMMANDS), "features": list(FEATURES),
                "read": ["references/commands/help.md"], "execution": "none"}
    features = list(dict.fromkeys(x.strip() for x in args.features.split(",")))
    if any(f not in FEATURES for f in features):
        parser.error("unknown/empty feature; use help; other Cosmos APIs and AWS features aren't supported")
    if args.command in {"backup", "restore"} and "backup" not in features:
        parser.error("recovery commands require backup in selected features")
    if args.command == "feature-plan":
        for f in features:
            if args.action not in ACTIONS.get(f, set()):
                parser.error(f"{f}: generic {args.action} unsupported; review native lifecycle")
    paths = [f"references/commands/{args.command}.md"]
    out = {"schema_version": 1, "mode": args.command, "selected_features": features,
           "execution": "none", "api": "NoSQL"}
    if args.command == "audit":
        paths.append(f"references/commands/audit/{args.mode}.md")
        out.update(audit_mode=args.mode, live_read_approval_required=args.mode == "live")
    elif args.command in {"backup", "restore"}:
        paths.extend(["references/commands/recovery-target.md",
                      f"references/commands/{args.command}/{args.method}.md"])
        out.update(method=args.method, change_approval_required=True)
        if args.command == "restore":
            out["target_policy"] = ("deleted-resource-only-no-overwrite"
                                    if args.method == "continuous-deleted-resource" else "new-account-only")
    elif args.command == "performance-test":
        out.update(environment=args.environment, run_approval_required=True,
                   limits={k: getattr(args, k) for k in ("duration_seconds", "max_requests", "max_concurrency", "max_ru")})
    elif args.command == "feature-plan":
        out.update(action=args.action, change_approval_required=True)
    for feature in features:
        paths.extend(FEATURES[feature])
    out["read"] = list(dict.fromkeys(paths))
    return out


if __name__ == "__main__":
    print(json.dumps(build_plan(), indent=2))
