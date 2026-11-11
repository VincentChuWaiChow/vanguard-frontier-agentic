#!/usr/bin/env python3
"""Gate: every NoSQL skill has a lazy-loaded audit with offline and live variants.

Covers each skill directory in ``validate-nosql-operations.py``'s ``SERVICES`` plus
the ported pattern skills. Each skill must ship:

* ``references/commands/audit.md`` (the router), linked from ``SKILL.md``;
* ``references/commands/audit/offline.md`` and ``references/commands/audit/live.md``,
  linked from the router and **not** from ``SKILL.md``, so an agent loads the
  router first and then exactly one variant;
* a router that says to load only one variant.

For the operations skills (not the pattern skills, whose audit is feature-based and
canonical upstream), both variants carry the ten operations headings in order, and
every live section either has a command block plus a citation, or a
``**No live command:**`` line giving the reason.

Command safety is an allowlist, checked on every fenced block in either variant
(only blocks tagged as data such as ``json`` or ``yaml`` are skipped; ``console``
blocks are rejected). Each block is lexed with shell rules, so commands after
``if``/``then``/``!``/``{``, after ``&&``/``||``/``|``/``;``, inside ``$(...)``,
backticks and process substitution are all checked, and ``"${name[@]}"`` heads are
expanded from their array definition:

* every command head must be on the mode's allowlist (``LIVE_HEADS`` or
  ``OFFLINE_HEADS``); wrappers such as ``env``, ``xargs``, ``bash -c``, ``eval``,
  absolute paths and heads taken from variables are rejected;
* live: a cloud CLI's action (the last word of its subcommand path) starts with a
  read verb; no path word is mutating, secret-reading (``keys``, ``secret``,
  ``connection strings``, ...) or a data-plane read (``query``, ``get-item``,
  ``read-rows``, ...); no confirmation-bypass or decryption flag; ``curl`` takes only
  allowlisted options and GET; ``python3`` runs only a quoted heredoc that imports
  ``json``/``sys`` and opens files read-only;
* offline: no cloud CLI, network tool or data shell; ``terraform``/``tofu`` only
  ``show``; ``rg --pre``, ``find -exec``/``-delete``, ``git -c`` and non-read git
  subcommands are rejected; ``python3`` only runs the skill's own ``scripts/*.py``;
* ``rg``/``grep`` searches for credential patterns list file names only (``-l``),
  so an audit never prints a secret it finds;
* heredoc delimiters are quoted (an unquoted body would expand ``$(...)``), and
  arithmetic expansion and ANSI-C quoting are rejected;
* every evidence ID cited in a live file is a VERIFIED row of the skill's ledger,
  and every ID cited in an offline file exists.

The file shapes live in ``templates/nosql-audit-template.md``. Never weaken a rule
to get green; reword the command or record ``**No live command:**`` instead.
"""

import ast
import importlib.util
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("nosql_ops", REPO / "tests/validate-nosql-operations.py")
_ops = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_ops)

PATTERN_SKILLS = [
    "skills/aws/aws-dynamodb-patterns",
    "skills/oci/oci-nosql-pattern",
    "skills/azure/azure-cosmosdb-pattern",
]
SKILLS = sorted({skill for skill, _ in _ops.SERVICES.values()} | set(PATTERN_SKILLS))
SECTIONS = _ops.SECTIONS
ROUTER = "references/commands/audit.md"
VARIANTS = {"offline": "references/commands/audit/offline.md", "live": "references/commands/audit/live.md"}
NO_LIVE = "**No live command:**"

CLOUD_CLIS = {"aws", "az", "gcloud", "oci", "aliyun", "hcloud", "scw", "ovhcloud", "ionosctl", "cbt", "bq", "firebase"}
DATA_SHELLS = {"mongosh", "mongo", "cqlsh", "mongodump", "mongoexport", "mongorestore", "mongoimport"}
NETWORK_TOOLS = {"curl", "wget"}
SHELL_UTILS = {":", "echo", "printf", "test", "[", "true", "false"}
LIVE_HEADS = CLOUD_CLIS | SHELL_UTILS | {"umask", "mktemp", "mkdir", "set", "exit", "grep", "jq", "curl", "python3"}
OFFLINE_HEADS = SHELL_UTILS | {"rg", "grep", "jq", "find", "git", "head", "tail", "sort", "wc", "cd", "terraform", "tofu",
                               "python3"}
KEYWORDS = {"if", "then", "else", "elif", "fi", "do", "done", "while", "until", "{", "}", "!"}

READ_VERBS = {"describe", "get", "list", "show", "ls", "view", "info", "status", "inspect", "lookup", "whoami"}
READ_EXCEPTIONS = {("gcloud", "logging", "read")}  # audit-log entries, not customer data
MUTATING_EXACT = {"set", "add", "untag", "run", "put", "stop", "exec", "move", "edit", "copy", "grant", "apply", "patch",
                  "deploy", "drop", "write", "cancel", "purge", "rotate", "revoke", "assign", "unassign", "promote", "rename",
                  "replace", "scale", "resize", "attach", "detach", "upload", "invoke", "insert", "reset", "clone", "change",
                  "start", "reboot", "migrate", "upgrade", "terminate", "kill", "truncate", "decrypt", "encrypt", "sign"}
MUTATING_PREFIX = ("create", "delete", "update", "modify", "restore", "remove", "enable", "disable", "import", "export",
                   "failover", "switchover", "register", "deregister", "execute")
SECRET_WORDS = {"keys", "secret", "secrets", "secretsmanager", "password", "passwords", "credential", "credentials",
                "token", "tokens", "decryption", "decrypt"}
SECRET_PAIRS = {("connection", "string"), ("connection", "strings"), ("access", "key"), ("master", "key")}
DATA_WORDS = {"item", "items", "row", "rows", "record", "records", "query", "scan", "statement", "statements",
              "document", "documents", "object", "objects", "range", "read"}
DATA_ACTIONS = {"lookup", "count", "select", "sample", "sample-row-keys"}  # whole action, e.g. cbt lookup
LISTABLE_DATA = {"object", "objects"}  # listing object names (backup inventory) is metadata
BYPASS_FLAGS = {"--force", "--yes", "-y", "--assume-yes", "--no-confirm", "--quiet-confirm"}

CURL_FLAGS = {"--silent", "--show-error", "--fail", "--fail-with-body", "--location", "--compressed", "--globoff"}
CURL_VALUE_FLAGS = {"--header", "--max-time", "--connect-timeout", "--output", "--write-out", "--retry", "--url",
                    "--request"}
CURL_SHORT = {"s": False, "S": False, "f": False, "L": False, "g": False, "H": True, "m": True, "o": True, "w": True,
              "X": True}
GIT_READ = {"log", "show", "grep", "ls-files", "diff", "blame", "rev-parse", "status"}
FIND_BANNED = {"-exec", "-execdir", "-ok", "-okdir", "-delete", "-fprint", "-fprint0", "-fprintf", "-fls"}
PY_IMPORTS = {"json", "sys"}
PY_BANNED_NAMES = {"__import__", "exec", "eval", "compile", "getattr", "setattr", "globals", "locals", "vars",
                   "breakpoint", "input", "__builtins__"}
CREDENTIAL_PATTERN = re.compile(r"accountkey|sharedaccesskey|passw|secret|private.?key|api.?key|token|"
                                r"://[^\s/]*:[^\s/]*@|mongodb(\+srv)?://", re.IGNORECASE)
NAMES_ONLY = {"-l", "--files-with-matches", "-c", "--count", "-q", "--quiet"}
DATA_LANGS = {"json", "jsonc", "yaml", "yml", "hcl", "toml", "xml", "csv", "diff", "markdown", "md"}
SESSION_LANGS = {"console", "shell-session", "sh-session", "terminal", "shellsession"}
OPERATORS = ["&&", "||", ";;", "|&", ";", "&", "|", "(", ")"]
REDIRECTS = ["&>>", "<<<", "<<-", "<<", ">>", "&>", ">&", "<&", ">|", "<>", "<", ">"]
NAME = r"[A-Za-z_]\w*"


@dataclass
class Word:
    value: str          # after quote removal; expansions kept as written
    raw: str            # source text
    dynamic: bool       # contains a parameter expansion or substitution
    quoted: bool


@dataclass
class Token:
    kind: str           # "word", "op", "redir"
    text: str = ""
    word: Word | None = None
    heredoc: dict | None = None


@dataclass
class Command:
    words: list[Word]
    heredocs: list[dict] = field(default_factory=list)


class Lexer:
    """A small shell lexer: quotes, escapes, comments, operators, redirections,
    heredocs, ``$(...)``, backticks and process substitution. Anything it cannot
    model safely is reported as a problem (fail closed)."""

    def __init__(self, text: str) -> None:
        self.s = text
        self.i = 0
        self.problems: list[str] = []
        self.nested: list[list[Token]] = []
        self.pending: list[dict] = []

    def run(self, close: bool = False) -> list[Token]:
        s, tokens, depth = self.s, [], 0
        chars: list[str] = []
        state = {"raw_start": None, "dynamic": False, "quoted": False}

        def flush() -> None:
            if state["raw_start"] is not None:
                word = Word("".join(chars), s[state["raw_start"]:self.i], state["dynamic"], state["quoted"])
                tokens.append(Token("word", word=word))
                if len(tokens) >= 2 and tokens[-2].kind == "redir" and tokens[-2].heredoc is not None:
                    doc = tokens[-2].heredoc
                    doc["delim"] = word.value
                    doc["quoted"] = word.quoted or "\\" in word.raw
                    self.pending.append(doc)
            chars.clear()
            state.update(raw_start=None, dynamic=False, quoted=False)

        def begin() -> None:
            if state["raw_start"] is None:
                state["raw_start"] = self.i

        while self.i < len(s):
            c = s[self.i]
            if c == "\\":
                if s[self.i + 1:self.i + 2] == "\n":
                    self.i += 2
                    continue
                begin()
                chars.append(s[self.i + 1:self.i + 2])
                self.i += 2
                continue
            if c in " \t":
                flush()
                self.i += 1
                continue
            if c == "\n":
                flush()
                tokens.append(Token("op", "\n"))
                self.i += 1
                self.read_heredocs()
                continue
            if c == "#" and state["raw_start"] is None:
                end = s.find("\n", self.i)
                self.i = len(s) if end < 0 else end
                continue
            if c == "'":
                begin()
                end = s.find("'", self.i + 1)
                if end < 0:
                    self.problems.append("unterminated single quote")
                    end = len(s)
                chars.append(s[self.i + 1:end])
                state["quoted"] = True
                self.i = end + 1
                continue
            if c == '"':
                begin()
                state["quoted"] = True
                self.i += 1
                while self.i < len(s) and s[self.i] != '"':
                    d = s[self.i]
                    if d == "\\" and s[self.i + 1:self.i + 2] in ('"', "\\", "$", "`", "\n"):
                        if s[self.i + 1] != "\n":
                            chars.append(s[self.i + 1])
                        self.i += 2
                    elif d in "$`":
                        state["dynamic"] |= self.expansion(chars)
                    else:
                        chars.append(d)
                        self.i += 1
                if self.i >= len(s):
                    self.problems.append("unterminated double quote")
                self.i += 1
                continue
            if c in "$`":
                begin()
                state["dynamic"] |= self.expansion(chars)
                continue
            if c in "<>" and s[self.i + 1:self.i + 2] == "(":
                begin()
                self.i += 2
                self.nested.append(self.run(close=True))
                chars.append("<(...)")
                state["dynamic"] = True
                continue
            if c in "|&;()<>":
                if c in "<>" and state["raw_start"] is not None and "".join(chars).isdigit():
                    chars.clear()
                    state.update(raw_start=None, dynamic=False, quoted=False)
                flush()
                if close and c == ")" and depth == 0:
                    self.i += 1
                    return tokens
                op = next((o for o in REDIRECTS if s.startswith(o, self.i)), None)
                if op and not (op == "&" or s.startswith("&&", self.i)):
                    token = Token("redir", op)
                    if op in ("<<", "<<-"):
                        token.heredoc = {"strip": op == "<<-", "body": None}
                    tokens.append(token)
                    self.i += len(op)
                    continue
                op = next(o for o in OPERATORS if s.startswith(o, self.i))
                depth += {"(": 1, ")": -1}.get(op, 0)
                tokens.append(Token("op", op))
                self.i += len(op)
                continue
            begin()
            chars.append(c)
            self.i += 1
        flush()
        if close:
            self.problems.append("unterminated $( or <( substitution")
        for doc in self.pending:
            self.problems.append(f"heredoc '{doc['delim']}' has no terminator")
        self.pending = []
        return tokens

    def expansion(self, chars: list[str]) -> bool:
        """Consume one $-expansion or backtick at self.i; return True (dynamic)."""
        s = self.s
        if s[self.i] == "`":
            end = self.i + 1
            while end < len(s) and s[end] != "`":
                end += 2 if s[end] == "\\" else 1
            inner = Lexer(s[self.i + 1:end])
            self.nested.append(inner.run())
            self.nested += inner.nested
            self.problems += inner.problems
            chars.append("`...`")
            self.i = end + 1
            return True
        nxt = s[self.i + 1:self.i + 2]
        if s.startswith("$((", self.i):
            self.problems.append("arithmetic expansion $((...)) is not allowed")
        if nxt == "'":
            self.problems.append("ANSI-C quoting $'...' is not allowed")
        if nxt == "(":
            self.i += 2
            self.nested.append(self.run(close=True))
            chars.append("$(...)")
            return True
        if nxt == "{":
            end = s.find("}", self.i)
            end = len(s) - 1 if end < 0 else end
            chars.append(s[self.i:end + 1])
            self.i = end + 1
            return True
        match = re.match(r"\$(" + NAME + r"|[0-9@*#?$!-])", s[self.i:])
        if match:
            chars.append(match.group(0))
            self.i += len(match.group(0))
            return True
        chars.append("$")
        self.i += 1
        return False

    def read_heredocs(self) -> None:
        for doc in self.pending:
            body = []
            while self.i < len(self.s):
                end = self.s.find("\n", self.i)
                end = len(self.s) if end < 0 else end
                line = self.s[self.i:end]
                self.i = end + 1
                if (line.lstrip("\t") if doc["strip"] else line) == doc["delim"]:
                    doc["body"] = "\n".join(body)
                    break
                body.append(line)
            else:
                self.problems.append(f"heredoc '{doc['delim']}' has no terminator")
                doc["body"] = "\n".join(body)
            if not doc["quoted"]:
                self.problems.append(f"heredoc '{doc['delim']}' must use a quoted delimiter (<<'{doc['delim']}')")
        self.pending = []


def split_commands(tokens: list[Token], aliases: dict[str, list[Word]], problems: list[str]) -> list[Command]:
    commands, current, heredocs = [], [], []
    skip_next, array_name, array_words = False, None, []
    for token in tokens:
        if array_name is not None:
            if token.kind == "op" and token.text == ")":
                aliases[array_name] = array_words
                array_name, array_words = None, []
            elif token.kind == "word":
                array_words.append(token.word)
            continue
        if token.kind == "op":
            if token.text == "(" and current and re.fullmatch(NAME + "=", current[-1].raw):
                array_name = current.pop().raw[:-1]
                continue
            commands.append(Command(current, heredocs))
            current, heredocs = [], []
            continue
        if token.kind == "redir":
            skip_next = True
            if token.heredoc is not None:
                heredocs.append(token.heredoc)
            continue
        if skip_next:
            skip_next = False
            continue
        current.append(token.word)
    commands.append(Command(current, heredocs))
    result = []
    for command in commands:
        words = command.words
        while True:
            while words and re.match(NAME + "=", words[0].raw):
                words = words[1:]
            if words and not words[0].quoted and words[0].raw in KEYWORDS:
                words = words[1:]
                continue
            break
        if words and words[0].raw in {"for", "select", "case"}:
            if words[0].raw == "case":
                problems.append("'case' is not allowed in an audit block")
            continue  # loop word lists hold no command; their substitutions are checked
        if words:
            alias = re.fullmatch(r'"?\$\{(' + NAME + r')\[@\]\}"?', words[0].raw)
            if alias:
                if alias.group(1) not in aliases:
                    problems.append(f"array '{alias.group(1)}' is used before it is defined")
                    continue
                words = aliases[alias.group(1)] + words[1:]
            result.append(Command(words, command.heredocs))
    return result


def parse_block(block: str, aliases: dict[str, list[Word]]) -> tuple[list[Command], list[str]]:
    lexer = Lexer(block)
    tokens = lexer.run()
    problems = list(lexer.problems)
    commands = []
    for stream in [tokens] + lexer.nested:
        commands += split_commands(stream, aliases, problems)
    return commands, problems


def commands(block: str) -> list[list[str]]:
    """Token values of every simple command in a shell block (array heads expanded)."""
    parsed, _ = parse_block(block, {})
    return [[word.value for word in command.words] for command in parsed]


def is_cli(tokens: list[str]) -> bool:
    return tokens[0] in CLOUD_CLIS


def pieces(word: str) -> list[str]:
    """Split a subcommand word on hyphens and CamelCase (aliyun/hcloud actions)."""
    found = []
    for part in word.split("-"):
        found += [w.lower() for w in re.findall(r"[A-Z]+(?![a-z])|[A-Z]?[a-z0-9]+", part)]
    return found


def is_path_word(word: Word) -> bool:
    return not word.dynamic and bool(re.fullmatch(r"[A-Za-z][A-Za-z0-9-]*", word.value))


def cli_path(words: list[Word]) -> list[str]:
    """Subcommand path: leading bare words after the CLI, skipping global options."""
    path, i = [], 1
    while i < len(words):
        word = words[i]
        if word.value.startswith("-") and not word.dynamic:
            if path:
                break
            if "=" not in word.value and i + 1 < len(words) and not is_path_word(words[i + 1]):
                i += 1
            i += 1
            continue
        if not is_path_word(word):
            break
        path.append(word.value)
        i += 1
    return path


def shown(words: list[Word]) -> str:
    return " ".join(word.value for word in words)[:120]


def cli_problems(words: list[Word]) -> list[str]:
    text = shown(words)
    path = cli_path(words)
    if not path:
        return [f"no subcommand found in '{text}'"]
    problems = []
    all_pieces = [p for word in path for p in pieces(word)]
    action = pieces(path[-1])
    for part in all_pieces:
        if part in MUTATING_EXACT or part.startswith(MUTATING_PREFIX):
            problems.append(f"mutating word '{part}' in '{text}'")
        if part in SECRET_WORDS:
            problems.append(f"secret-reading word '{part}' in '{text}'")
        if part in DATA_WORDS and not (part in LISTABLE_DATA and action[:1] in (["list"], ["ls"])) \
                and (words[0].value, *path) not in READ_EXCEPTIONS:
            problems.append(f"data-plane word '{part}' in '{text}'; audit control-plane metadata only")
    if path[-1].lower() in DATA_ACTIONS:
        problems.append(f"data-plane action '{path[-1]}' in '{text}'; audit control-plane metadata only")
    for pair in zip(all_pieces, all_pieces[1:]):
        if pair in SECRET_PAIRS:
            problems.append(f"secret-reading words '{' '.join(pair)}' in '{text}'")
    for word in words[1:]:
        if word.value.startswith("-"):
            flag = word.value.split("=", 1)[0]
            if flag in BYPASS_FLAGS:
                problems.append(f"confirmation-bypass flag '{flag}' in '{text}'")
            if any(p in SECRET_WORDS for p in pieces(flag.lstrip("-"))):
                problems.append(f"secret-reading flag '{flag}' in '{text}'")
    if (words[0].value, *path) not in READ_EXCEPTIONS and (not action or action[0] not in READ_VERBS):
        problems.append(f"action '{path[-1]}' is not a read ({', '.join(sorted(READ_VERBS))}) in '{text}'")
    return problems


def curl_problems(words: list[Word]) -> list[str]:
    text, problems, i = shown(words), [], 1
    while i < len(words):
        value = words[i].value
        method = None
        if words[i].dynamic and not words[i].quoted:
            problems.append(f"unquoted expansion '{words[i].raw}' can inject curl options in '{text}'")
        if value.startswith("--"):
            name, eq, arg = value.partition("=")
            if name in CURL_VALUE_FLAGS:
                if not eq:
                    i += 1
                    arg = words[i].value if i < len(words) else ""
                if name == "--request":
                    method = arg
            elif name not in CURL_FLAGS or eq:
                problems.append(f"curl option '{name}' is not allowed in '{text}'")
        elif value.startswith("-") and len(value) > 1 and not words[i].dynamic:
            j = 1
            while j < len(value):
                flag = value[j]
                if flag not in CURL_SHORT:
                    problems.append(f"curl option '-{flag}' is not allowed in '{text}'")
                    break
                if CURL_SHORT[flag]:
                    arg = value[j + 1:]
                    if not arg:
                        i += 1
                        arg = words[i].value if i < len(words) else ""
                    if flag == "X":
                        method = arg
                    break
                j += 1
        if method is not None and method.upper() != "GET":
            problems.append(f"curl uses method '{method}' in '{text}'; GET only")
        i += 1
    return problems


def python_problems(command: Command, mode: str, skill: Path) -> list[str]:
    words, text = command.words, shown(command.words)
    if mode == "offline":
        script = words[1].value if len(words) > 1 else ""
        if re.fullmatch(r"scripts/[\w-]+\.py", script) and (skill / script).is_file():
            return []
        return [f"offline python3 may only run the skill's own scripts/*.py: '{text}'"]
    if len(words) < 2 or words[1].value != "-" or len(command.heredocs) != 1:
        return [f"live python3 must read a quoted heredoc script ('python3 - <<'PY''): '{text}'"]
    try:
        tree = ast.parse(command.heredocs[0]["body"] or "")
    except SyntaxError as error:
        return [f"python3 heredoc does not parse ({error.msg}): '{text}'"]
    problems = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            names = [alias.name for alias in node.names] if isinstance(node, ast.Import) else [node.module or ""]
            problems += [f"python3 heredoc imports '{n}'; only {sorted(PY_IMPORTS)}" for n in names
                         if n.split(".")[0] not in PY_IMPORTS]
        elif isinstance(node, ast.Name) and node.id in PY_BANNED_NAMES:
            problems.append(f"python3 heredoc uses '{node.id}'")
        elif isinstance(node, ast.Attribute) and node.attr.startswith("__"):
            problems.append(f"python3 heredoc uses dunder attribute '{node.attr}'")
        elif isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "open" \
                and (len(node.args) != 1 or node.keywords):
            problems.append("python3 heredoc may only call open(path) read-only")
    return problems


def command_problems(command: Command, mode: str, skill: Path) -> list[str]:
    words = command.words
    head, text = words[0].value, shown(words)
    if words[0].dynamic:
        return [f"command name comes from an expansion in '{text}'; name the program literally"]
    allowed = LIVE_HEADS if mode == "live" else OFFLINE_HEADS
    if head in DATA_SHELLS:
        return [f"data-plane shell '{head}' in '{text}'; audit control-plane metadata only"]
    if head not in allowed:
        if mode == "offline" and (head in CLOUD_CLIS or head in NETWORK_TOOLS):
            return [f"'{head}' in offline audit ('{text}'); offline means no cloud, CLI or network call"]
        return [f"'{head}' is not an allowed {mode} audit command ('{text}'); allowed: {' '.join(sorted(allowed))}"]
    if head in CLOUD_CLIS:
        return cli_problems(words)
    if head == "curl":
        return curl_problems(words)
    if head == "python3":
        return python_problems(command, mode, skill)
    values = [word.value for word in words]
    if head in {"terraform", "tofu"} and (len(values) < 2 or values[1] != "show"):
        return [f"'{text}' can reach a provider; offline allows only '{head} show' on a saved plan"]
    if head in {"rg", "grep"} and any(CREDENTIAL_PATTERN.search(v) for v in values[1:]) \
            and not any(v in NAMES_ONLY or re.fullmatch(r"-[A-Za-z]*[lcq][A-Za-z]*", v) for v in values[1:]):
        return [f"'{text}' searches for credentials and would print them; list file names only (-l)"]
    if head == "rg" and any(v == "--pre" or v.startswith("--pre=") for v in values):
        return [f"'rg --pre' runs a program: '{text}'"]
    if head == "find" and FIND_BANNED & set(values):
        return [f"find action {sorted(FIND_BANNED & set(values))} runs or deletes: '{text}'"]
    if head == "sort" and any(v.startswith("--compress-program") for v in values):
        return [f"'sort --compress-program' runs a program: '{text}'"]
    if head == "git":
        rest = values[1:]
        while rest and rest[0] in {"-C", "--git-dir", "--work-tree", "--no-pager"}:
            rest = rest[1:] if rest[0] == "--no-pager" else rest[2:]
        sub = next((v for v in rest if not v.startswith("-")), "")
        if any(v in {"-c", "--config-env"} or v.startswith("--config-env=") for v in values[1:]) \
                or sub not in GIT_READ:
            return [f"git must be a read subcommand ({', '.join(sorted(GIT_READ))}) without -c: '{text}'"]
    return []


def fenced_blocks(text: str) -> list[tuple[str, str]]:
    """Every fenced code block as (language, body), at any indent and with ``` or ~~~."""
    blocks, lines, i = [], text.splitlines(), 0
    while i < len(lines):
        match = re.match(r"^(\s*)(`{3,}|~{3,})\s*([^`\s]*)", lines[i])
        if not match:
            i += 1
            continue
        indent, fence, lang = len(match.group(1)), match.group(2), match.group(3).lower()
        body, i = [], i + 1
        while i < len(lines) and not re.fullmatch(r"\s*" + re.escape(fence[0]) + "{" + str(len(fence)) + r",}\s*",
                                                    lines[i]):
            line = lines[i]
            body.append(line[min(indent, len(line) - len(line.lstrip(" "))):])
            i += 1
        blocks.append((lang, "\n".join(body) + "\n"))
        i += 1
    return blocks


def shell_blocks(text: str, where: str, errors: list[str]) -> list[str]:
    found = []
    for lang, body in fenced_blocks(text):
        if lang in SESSION_LANGS:
            errors.append(f"{where}: a ```{lang} block mixes commands and output; use ```bash for commands")
        elif lang not in DATA_LANGS:
            found.append(body)
    return found


def sections(text: str) -> dict[str, str]:
    starts = [m.start() for m in re.finditer(r"^## ", text, re.MULTILINE)] + [len(text)]
    found = {}
    for begin, end in zip(starts, starts[1:]):
        block = text[begin:end]
        found[block.splitlines()[0][3:].strip()] = block
    return found


def check(skill_rel: str, errors: list[str]) -> None:
    skill = REPO / skill_rel
    pattern = skill_rel in PATTERN_SKILLS
    skill_md = (skill / "SKILL.md").read_text(encoding="utf-8")
    router = skill / ROUTER
    if not router.exists():
        errors.append(f"{skill_rel}: missing {ROUTER}")
        return
    if f"]({ROUTER})" not in skill_md:
        errors.append(f"{skill_rel}: SKILL.md does not link {ROUTER}; no agent would load the audit")
    router_text = router.read_text(encoding="utf-8")
    if not re.search(r"\b(only|Do not read both)\b", router_text):
        errors.append(f"{skill_rel}/{ROUTER}: must say to load only the selected variant")
    rows = _ops.ledger_status(skill)
    for mode, rel in VARIANTS.items():
        where = f"{skill_rel}/{rel}"
        path = skill / rel
        if not path.exists():
            errors.append(f"{where}: missing")
            continue
        if f"]({rel.removeprefix('references/commands/')})" not in router_text:
            errors.append(f"{skill_rel}/{ROUTER}: does not link {rel.removeprefix('references/commands/')}")
        if f"]({rel})" in skill_md:
            errors.append(f"{skill_rel}: SKILL.md links {rel} directly; link only the router so variants load lazily")
        text = path.read_text(encoding="utf-8")
        blocks = shell_blocks(text, where, errors)
        aliases: dict[str, list[Word]] = {}
        for block in blocks:
            parsed, problems = parse_block(block, aliases)
            errors += [f"{where}: {p}" for p in problems]
            for command in parsed:
                errors += [f"{where}: {p}" for p in command_problems(command, mode, skill)]
        for evidence in _ops.MARKER_RE.findall(text):
            if evidence not in rows:
                errors.append(f"{where}: {evidence} has no row in the skill's evidence ledger")
            elif mode == "live" and rows[evidence] != "VERIFIED":
                errors.append(f"{where}: cites {evidence}, whose ledger status is {rows[evidence]}; live commands need VERIFIED sources")
        if pattern:
            continue
        if not blocks:
            errors.append(f"{where}: needs at least one bash command block")
        headings = [h for h in re.findall(r"^## (.+?)\s*$", text, re.MULTILINE) if h in SECTIONS]
        if headings != SECTIONS:
            errors.append(f"{where}: sections must be exactly {SECTIONS} in order; found {headings}")
            continue
        if mode == "live":
            for title, block in sections(text).items():
                if title not in SECTIONS:
                    continue
                has_command = bool(shell_blocks(block, where, [])) and bool(_ops.MARKER_RE.search(block))
                if not has_command and NO_LIVE not in block:
                    errors.append(f"{where} [{title}]: needs a cited command block or a '{NO_LIVE}' line with the reason")


def main() -> int:
    errors: list[str] = []
    for skill_rel in SKILLS:
        check(skill_rel, errors)
    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        print(f"validate-nosql-audit: {len(errors)} problem(s)")
        return 1
    print(f"validate-nosql-audit: OK ({len(SKILLS)} skills, offline and live audit variants)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
