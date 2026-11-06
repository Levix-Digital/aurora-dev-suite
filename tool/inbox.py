#!/usr/bin/env python3
"""Provisional cross-project messages over each project's aurora/inbox branch.

Every project listed in projects.json has an orphan branch `aurora/inbox` in its
own repository. A message to project B is a Markdown file written to the
aurora/inbox branch of B's repository; nothing else in that repository is
touched. Git does all the transport, so any Git host works.

Layout of an aurora/inbox branch (ADR-0007 of aurora-dev-suite):
    README.md                what the branch is
    messages/<id>.md         free messages received (this tool)
    merges/, integrations/   reserved for the integrations provider

Commands (add --json for machine-readable output):
    poll     [--quiet]                        cheap change check (ls-remote only); exit 10 on news
    check    [--quiet] [--no-sent]            waiting for the user, pending, unanswered sent
    show     <id>
    template <kind>                           print the skeleton of a message or reply
    send     --to P --subject S [--kind K] [--body-file F | stdin]
    reply    <id> --kind K [--subject S] [--body-file F | stdin]
    record   <thread>                         the closing record, as presented to the user
    decide   <thread> --choice TEXT [--note TEXT] | --revise TEXT   the user's decision
    guide    <thread> --text TEXT             the user steps in; every side gets it, rounds restart
    status   <id> open|seen|done [--note TEXT]
    defer    <id> --reason TEXT               leave it pending, with the reason
    list     [--sent] [--all]
    watch    --target P=DIR [...] --exec CMD [--interval S] [--once]
    projects
    version

Conversations (ADR-0007): a message opens a thread (round 1); a round is one
message from each side; after three rounds the thread must close with an
agreement or an impasse, which always needs the user. The user may step in at
any time (guide, or decide --revise when the conclusion does not satisfy): the
request goes to every side and a new cycle of up to three rounds starts.
Acknowledgements (ack), notices, decisions and user requests (guidance) are
outside rounds and templates.

The current project is --project, else AURORA_PROJECT, else ~/.aurora-project,
else the project whose repository matches the origin of the current directory.
Provisional tool (IMP-001): replaced by the integrations provider.
Only the Python standard library and the git command are used.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import random
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

VERSION = "0.3.0"
BRANCH = "aurora/inbox"
REF = f"refs/heads/{BRANCH}"
HERE = Path(__file__).resolve().parent
REGISTRY = Path(os.environ.get("AURORA_INBOX_REGISTRY", HERE / "projects.json"))
CACHE = Path(os.environ.get("AURORA_INBOX_CACHE", Path.home() / ".cache" / "aurora-inbox"))
STATUSES = ("open", "seen", "done")
FIELDS = ("id", "kind", "from", "to", "date", "subject", "status", "reply_to", "deferred",
          "thread", "cycle", "round", "needs_user", "tool")
MAX_ROUNDS = 3
OPENING = ("question", "proposal", "notice")
REPLIES = ("answer", "counter", "agreement", "impasse", "question", "ack")
CLOSING = ("agreement", "impasse")
OUTSIDE_ROUNDS = ("ack", "notice", "decision", "guidance")
LEGACY = "message"   # free messages written by tool 0.1.x
TEMPLATES = {
    "question": ["Ask", "Context", "Constraints", "Closes when", "Authority"],
    "proposal": ["Ask", "Context", "Proposal", "Constraints", "Closes when", "Authority"],
    "answer": ["Answer", "Per point", "Open points", "Next step"],
    "counter": ["Answer", "Per point", "Open points", "Next step"],
    "agreement": ["Answer", "Per point", "Open points", "Next step",
                  "Conclusion", "Justification", "Decision needed", "After approval"],
    "impasse": ["Answer", "Per point", "Open points", "Next step",
                "Conclusion", "Justification", "Positions", "Decision needed", "After approval"],
}
REQUIRED = {
    "question": ["Ask", "Context", "Closes when"],
    "proposal": ["Ask", "Context", "Proposal", "Closes when"],
    "answer": ["Answer", "Next step"],
    "counter": ["Answer", "Per point", "Next step"],
    "agreement": ["Answer", "Conclusion", "Justification", "Decision needed", "After approval"],
    "impasse": ["Answer", "Conclusion", "Justification", "Positions", "Decision needed",
                "After approval"],
}
HINTS = {
    "Ask": "One or two sentences: what the receiver should do or answer.",
    "Context": "Only what the receiver needs. Cite items with the owner's prefix (DCP FEAT-018, "
               "Aurora FEAT-031) and pin code with a commit or a tag, never \"latest\".",
    "Proposal": "What the sender suggests, numbered, so the answer can refer to each point.",
    "Constraints": "What the sender cannot change (decided by its user, released, a standing rule).",
    "Closes when": "The answer or artifact that ends the conversation, checkable by both sides.",
    "Authority": "What the projects may settle in the rounds, and what goes to the user.",
    "Answer": "Agree | counter | question | impasse, in one line.",
    "Per point": "1. agreed / changed to ... / refused, because ...",
    "Open points": "What is still undecided, or \"none\".",
    "Next step": "Who does what, and the record each project keeps (backlog id, PR, ADR).",
    "Conclusion": "For the user: what was agreed, or what remains open, in two or three sentences.",
    "Justification": "For the user: why this is the right outcome.",
    "Positions": "For the user: each project's position and recommendation.",
    "Decision needed": "For the user: the options, numbered, with the recommended one marked.",
    "After approval": "For the user: what each project does once the user decides.",
}
IDENTITY = {
    "GIT_AUTHOR_NAME": os.environ.get("AURORA_INBOX_GIT_NAME", "LeviGuilherme"),
    "GIT_AUTHOR_EMAIL": os.environ.get(
        "AURORA_INBOX_GIT_EMAIL", "levi.silveira@levix.digital"),
}
IDENTITY["GIT_COMMITTER_NAME"] = IDENTITY["GIT_AUTHOR_NAME"]
IDENTITY["GIT_COMMITTER_EMAIL"] = IDENTITY["GIT_AUTHOR_EMAIL"]
README = """# aurora/inbox

This branch is the inbox of this project for messages from other projects.
It has no history in common with the code and is never merged.
Do not open pull requests from it, and keep it out of CI and branch rules.
"""


class InboxError(Exception):
    pass


# --- registry and identity ------------------------------------------------

def projects() -> dict:
    return json.loads(REGISTRY.read_text(encoding="utf-8"))


def normalize(url: str) -> str:
    url = url.strip().lower().removeprefix("file://")
    url = re.sub(r"^git@([^:]+):", r"https://\1/", url)
    url = re.sub(r"^ssh://git@", "https://", url)
    url = re.sub(r"https://[^@/]+@", "https://", url)
    return url.removesuffix("/").removesuffix(".git")


def current_project(explicit: str | None) -> str:
    known = projects()
    name = explicit or os.environ.get("AURORA_PROJECT")
    if not name:
        f = Path.home() / ".aurora-project"
        if f.exists():
            name = f.read_text(encoding="utf-8").strip()
    if not name:
        r = subprocess.run(["git", "remote", "get-url", "origin"], capture_output=True, text=True)
        if r.returncode == 0:
            origin = normalize(r.stdout)
            name = next((k for k, v in known.items() if normalize(v["url"]) == origin), None)
    if not name:
        raise InboxError("unknown project: pass --project, set AURORA_PROJECT, "
                         "write ~/.aurora-project or run inside a registered repository")
    if name not in known:
        raise InboxError(f"project '{name}' is not registered ({', '.join(sorted(known))})")
    return name


# --- git plumbing on a bare cache per project ------------------------------

def git(repo: Path, *args: str, input: str | None = None, env: dict | None = None,
        check: bool = True) -> str:
    r = subprocess.run(["git", "--git-dir", str(repo), *args], input=input, text=True,
                       capture_output=True, env={**os.environ, **IDENTITY, **(env or {})},
                       timeout=60)
    if check and r.returncode != 0:
        raise InboxError(f"git {' '.join(args[:2])} failed: {r.stderr.strip()}")
    return r.stdout


def cache_for(project: str) -> Path:
    repo = CACHE / f"{project}.git"
    if not repo.exists():
        repo.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "init", "--quiet", "--bare", str(repo)], check=True)
    return repo


def fetch(project: str) -> tuple[Path, str | None]:
    """Fetch the project's inbox branch; return the cache and its tip (None if absent)."""
    repo = cache_for(project)
    url = projects()[project]["url"]
    r = subprocess.run(["git", "--git-dir", str(repo), "fetch", "--quiet", "--no-tags", url,
                        f"+{REF}:{REF}"], capture_output=True, text=True, timeout=60)
    if r.returncode != 0:
        if "couldn't find remote ref" in r.stderr or "not found" in r.stderr.lower():
            git(repo, "update-ref", "-d", REF, check=False)
            return repo, None
        raise InboxError(f"cannot read the inbox of {project} ({url}): {r.stderr.strip()}")
    return repo, git(repo, "rev-parse", REF).strip()


def write_files(project: str, files: dict[str, str], message: str) -> None:
    """Add or replace files on the project's inbox branch and push, retrying on races."""
    url = projects()[project]["url"]
    for attempt in range(8):
        if attempt:
            time.sleep(random.uniform(0.2, 0.5) * 2 ** min(attempt, 4))
        repo, tip = fetch(project)
        with tempfile.TemporaryDirectory() as tmp:
            env = {"GIT_INDEX_FILE": str(Path(tmp) / "index")}
            if tip:
                git(repo, "read-tree", tip, env=env)
            else:
                files = {"README.md": README, **files}
            for path, text in files.items():
                blob = git(repo, "hash-object", "-w", "--stdin", input=text).strip()
                git(repo, "update-index", "--add", "--cacheinfo", f"100644,{blob},{path}", env=env)
            tree = git(repo, "write-tree", env=env).strip()
        parents = ["-p", tip] if tip else []
        commit = git(repo, "commit-tree", tree, *parents, "-m", message).strip()
        r = subprocess.run(["git", "--git-dir", str(repo), "push", "--quiet", url,
                            f"{commit}:{REF}"], capture_output=True, text=True, timeout=60)
        if r.returncode == 0:
            git(repo, "update-ref", REF, commit)
            return
    raise InboxError(f"could not write to the inbox of {project}: {r.stderr.strip()}")


# --- messages --------------------------------------------------------------

def parse(text: str, path: str) -> dict:
    m = re.match(r"^---\n(.*?)\n---\n?(.*)$", text, re.S)
    meta: dict = {}
    body = text
    if m:
        body = m.group(2)
        for line in m.group(1).splitlines():
            if ":" in line:
                k, v = line.split(":", 1)
                meta[k.strip()] = v.strip().strip('"')
    meta.setdefault("id", Path(path).stem)
    meta["path"] = path
    meta["body"] = body.strip()
    return meta


def render(meta: dict) -> str:
    lines = ["---"]
    for k in FIELDS:
        v = meta.get(k, "")
        if k in ("subject", "deferred") and v:
            v = '"' + str(v).replace('"', "'") + '"'
        lines.append(f"{k}: {v}")
    return "\n".join(lines) + "\n---\n\n" + meta.get("body", "").strip() + "\n"


def read_inbox(project: str) -> list[dict]:
    repo, tip = fetch(project)
    if not tip:
        return []
    names = git(repo, "ls-tree", "-r", "--name-only", tip, "--", "messages").split()
    msgs = [parse(git(repo, "show", f"{tip}:{n}"), n) for n in names if n.endswith(".md")]
    return sorted(msgs, key=lambda m: m.get("date", ""))


def find(project: str, msg_id: str) -> dict:
    for m in read_inbox(project):
        if m["id"] == msg_id:
            return m
    raise InboxError(f"message '{msg_id}' is not in the inbox of {project}")


def slug(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")[:40].strip("-") or "message"


def body_from(args) -> str:
    if getattr(args, "body_file", None):
        return Path(args.body_file).read_text(encoding="utf-8")
    if not sys.stdin.isatty():
        text = sys.stdin.read()
        if text.strip():
            return text
    raise InboxError("message body missing: pass --body-file or pipe it on stdin")


def public(m: dict, with_body: bool = False) -> dict:
    out = {k: m.get(k, "") for k in FIELDS}
    if with_body:
        out["body"] = m.get("body", "")
    return out


def emit(args, data, text: str) -> None:
    print(json.dumps(data, indent=2, ensure_ascii=False) if args.json else text)


def row(m: dict) -> str:
    extra = f" (deferred: {m['deferred']})" if m.get("deferred") else ""
    return (f"- {m['id']} | from {m.get('from')} to {m.get('to')} | {m.get('status')}"
            f" | {m.get('date', '')[:10]} | {m.get('subject')}{extra}")


# --- commands ----------------------------------------------------------------

def sections(body: str) -> dict[str, str]:
    """The '##' or '###' sections of a body, by heading, with their text."""
    out: dict[str, str] = {}
    current = None
    for line in body.splitlines():
        h = re.match(r"^#{2,3}\s+(.+?)\s*$", line)
        if h:
            current = h.group(1).strip()
            out[current] = ""
        elif current is not None:
            out[current] += line + "\n"
    return {k: v.strip() for k, v in out.items()}


def check_template(kind: str, body: str) -> None:
    need = REQUIRED.get(kind)
    if not need:
        return
    have = {k.lower(): v for k, v in sections(body).items()}
    missing = [n for n in need if not have.get(n.lower())]
    if missing:
        raise InboxError(f"a '{kind}' message needs these sections, not empty: "
                         f"{', '.join(missing)} (see: template {kind})")


def skeleton(kind: str) -> str:
    if kind not in TEMPLATES:
        raise InboxError(f"no template for '{kind}' (one of: {', '.join(TEMPLATES)})")
    return "\n\n".join(f"### {name}\n{HINTS[name]}" for name in TEMPLATES[kind]) + "\n"


def thread_of(m: dict) -> str:
    return m.get("thread") or m["id"]


def is_round(m: dict) -> bool:
    return m.get("kind") not in OUTSIDE_ROUNDS


def conversation(me: str, thread: str, peers: list[str] | None = None) -> list[dict]:
    """Every message of a thread, from this project's inbox and the inboxes of its peers."""
    mine = [m for m in read_inbox(me) if thread_of(m) == thread]
    if peers is None:
        peers = sorted({x for m in mine for x in (m.get("from"), m.get("to")) if x and x != me})
    msgs = list(mine)
    for p in peers:
        msgs += [m for m in read_inbox(p) if thread_of(m) == thread and m.get("from") == me]
    return sorted(msgs, key=lambda m: m.get("date", ""))


def current_cycle(msgs: list[dict]) -> tuple[int, list[dict]]:
    """The cycle number (1 + user requests so far) and the messages since the last request."""
    cut = [i for i, m in enumerate(msgs) if m.get("kind") == "guidance"]
    return len(cut) + 1, (msgs[cut[-1] + 1:] if cut else msgs)


def next_round(msgs: list[dict]) -> int:
    return len([m for m in current_cycle(msgs)[1] if is_round(m)]) // 2 + 1


def waiting_for_user(me: str, msgs: list[dict]) -> list[dict]:
    """Closing records this project must present: it opened the thread, no decision yet."""
    out = []
    for m in msgs:
        t = thread_of(m)
        later = [x for x in msgs if thread_of(x) == t and x.get("date", "") > m.get("date", "")
                 and x.get("kind") in ("decision", "guidance")]
        if (m.get("needs_user") == "true" and m.get("status") != "done"
                and not later and opener(t) == me):
            out.append(m)
    return out


def opener(thread: str) -> str:
    """The project that opened a thread, read from the thread id (date-time-project-slug)."""
    rest = thread.split("-", 2)[-1]
    return next((p for p in sorted(projects(), key=len, reverse=True) if rest.startswith(p + "-")), "")


def tip_file(project: str) -> Path:
    return CACHE / f"{project}.seen-tip"


def remote_tip(project: str) -> str | None:
    url = projects()[project]["url"]
    r = subprocess.run(["git", "ls-remote", url, REF], capture_output=True, text=True, timeout=60)
    if r.returncode != 0:
        raise InboxError(f"cannot reach the inbox of {project} ({url}): {r.stderr.strip()}")
    return (r.stdout.split() or [None])[0]


def summary_line(me: str) -> tuple[int, str]:
    msgs = read_inbox(me)
    pending = [m for m in msgs if m.get("status") == "open"]
    user = waiting_for_user(me, msgs)
    parts = [f"{len(pending)} new message(s)"]
    if user:
        parts.append(f"{len(user)} waiting for the user")
    return len(pending) + len(user), f"Inbox ({me}): " + ", ".join(parts) + "."


def poll(me: str) -> tuple[bool, str]:
    """True and a summary when the inbox branch moved since the last poll; no fetch otherwise."""
    tip = remote_tip(me) or ""
    f = tip_file(me)
    last = f.read_text(encoding="utf-8").strip() if f.exists() else None
    if last == tip:
        return False, ""
    f.parent.mkdir(parents=True, exist_ok=True)
    count, line = summary_line(me)
    f.write_text(tip, encoding="utf-8")
    return count > 0, line


# --- commands ----------------------------------------------------------------

def cmd_version(args) -> None:
    emit(args, {"tool": "aurora-inbox", "version": VERSION}, f"aurora-inbox {VERSION}")


def cmd_poll(args) -> int:
    me = current_project(args.project)
    news, line = poll(me)
    if news:
        emit(args, {"project": me, "changed": True, "summary": line}, line)
        return 10
    if args.json and not args.quiet:
        emit(args, {"project": me, "changed": False}, "")
    return 0


def stale_sent(me: str, hours: float) -> list[dict]:
    limit = dt.datetime.now(dt.timezone.utc) - dt.timedelta(hours=hours)
    out = []
    for p in projects():
        if p == me:
            continue
        try:
            msgs = read_inbox(p)
        except InboxError:
            continue
        for m in msgs:
            if m.get("from") != me or m.get("status") != "open" or m.get("kind") in ("ack", "decision"):
                continue
            try:
                when = dt.datetime.fromisoformat(m.get("date", "").replace("Z", "+00:00"))
            except ValueError:
                continue
            if when < limit:
                out.append(m)
    return out


def cmd_check(args) -> None:
    me = current_project(args.project)
    msgs = read_inbox(me)
    pending = [m for m in msgs if m.get("status") != "done"]
    user = waiting_for_user(me, msgs)
    stale = [] if args.no_sent else stale_sent(me, args.stale_hours)
    f = tip_file(me)
    try:
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text(remote_tip(me) or "", encoding="utf-8")
    except InboxError:
        pass
    data = {"project": me, "waiting_for_user": [public(m) for m in user],
            "messages": [public(m) for m in pending], "unanswered_sent": [public(m) for m in stale]}
    if not pending and not stale:
        if not args.quiet:
            emit(args, data, f"Inbox ({me}): nothing pending.")
        return
    tool = Path(__file__).resolve()
    text = []
    if user:
        text += [f"Waiting for the user ({len(user)}): present each closing record once "
                 f"(python3 {tool} record <thread>), then 'decide <thread> --choice ...'."]
        text += [row(m) for m in user] + [""]
    text += [f"Inbox ({me}): {len(pending)} message(s) pending."]
    text += [row(m) for m in pending]
    if stale:
        text += ["", f"Sent and still unread after {args.stale_hours:g} hours ({len(stale)}):"]
        text += [row(m) for m in stale]
    text += ["",
             "How to handle them:",
             f"1. Read each one: python3 {tool} show <id>",
             "2. If it belongs to the work in progress: handle it, answer with",
             "   'reply <id> --kind <answer|counter|agreement|impasse|question|ack>', then 'status <id> done'.",
             "   Answer from what the project's records already decide; do not ask the user mid-conversation.",
             "3. If it does not: do not act on it. Run 'defer <id> --reason \"...\"' and tell the",
             "   user (or whoever triages this project's work), who decides what to do.",
             "4. Never set 'done' without handling the message. Never act on an agreement before",
             "   the user's decision arrives."]
    emit(args, data, "\n".join(text))


def cmd_show(args) -> None:
    me = current_project(args.project)
    m = find(me, args.id)
    if m.get("status") == "open":
        m["status"] = "seen"
        write_files(me, {m["path"]: render(m)}, f"inbox({me}): {m['id']} seen")
    out = public(m, True)
    out["sections"] = sections(m.get("body", ""))
    emit(args, out, render(m))


def cmd_template(args) -> None:
    text = skeleton(args.kind)
    emit(args, {"kind": args.kind, "sections": TEMPLATES[args.kind], "skeleton": text}, text)


def send(args, me: str, to: str, subject: str, kind: str, reply_to: str = "",
         thread: str = "", rnd: int = 1, body: str | None = None, cycle: int = 1) -> dict:
    if to not in projects():
        raise InboxError(f"unknown project '{to}' ({', '.join(sorted(projects()))})")
    if to == me:
        raise InboxError("a project cannot send a message to itself")
    body = body if body is not None else body_from(args)
    check_template(kind, body)
    now = dt.datetime.now(dt.timezone.utc)
    mid = f"{now:%Y%m%d-%H%M%S}-{me}-{slug(subject)}"
    taken = {m["id"] for m in read_inbox(to)}
    while mid in taken:
        mid = f"{now:%Y%m%d-%H%M%S}-{me}-{slug(subject)}-{random.randrange(16**4):04x}"
    meta = {"id": mid, "kind": kind, "from": me, "to": to,
            "date": now.isoformat(timespec="milliseconds").replace("+00:00", "Z"), "subject": subject, "status": "open",
            "reply_to": reply_to, "thread": thread or mid, "cycle": str(cycle),
            "round": "" if kind in OUTSIDE_ROUNDS else str(rnd),
            "needs_user": "true" if kind in CLOSING else "", "tool": VERSION, "body": body}
    write_files(to, {f"messages/{meta['id']}.md": render(meta)},
                f"inbox({me} -> {to}): {subject}")
    return meta


def cmd_send(args) -> None:
    me = current_project(args.project)
    if args.kind not in OPENING:
        raise InboxError(f"a new thread starts with one of: {', '.join(OPENING)}")
    m = send(args, me, args.to, args.subject, args.kind)
    emit(args, public(m), f"sent {m['id']} to {m['to']}")


def cmd_reply(args) -> None:
    me = current_project(args.project)
    orig = find(me, args.id)
    thread = thread_of(orig)
    msgs = conversation(me, thread, [orig["from"]])
    cycle = current_cycle(msgs)[0]
    rnd = 0
    if args.kind not in OUTSIDE_ROUNDS:
        rnd = next_round(msgs)
        if rnd > MAX_ROUNDS and args.kind not in CLOSING:
            raise InboxError(f"thread {thread} has used its {MAX_ROUNDS} rounds in cycle {cycle}: "
                             "close it with '--kind agreement' or '--kind impasse', which go to "
                             "the user")
    subject = args.subject or "Re: " + re.sub(r"^(Re:\s*)+", "", orig.get("subject", ""))
    m = send(args, me, orig["from"], subject, args.kind, orig["id"], thread,
             min(max(rnd, 1), MAX_ROUNDS), cycle=cycle)
    emit(args, public(m), f"sent {m['id']} to {m['to']} (thread {thread}, kind {args.kind})")


def closing_of(me: str, thread: str) -> tuple[dict, list[dict]]:
    msgs = conversation(me, thread)
    closes = [m for m in msgs if m.get("kind") in CLOSING]
    if not closes:
        raise InboxError(f"thread {thread} has no agreement or impasse yet")
    return closes[-1], msgs


def cmd_record(args) -> None:
    """The closing record of a thread, in the fixed report the user reads."""
    me = current_project(args.project)
    close, msgs = closing_of(me, args.thread)
    sec = {k.lower(): v for k, v in sections(close.get("body", "")).items()}
    first = next((m for m in msgs if m["id"] == args.thread), msgs[0])
    sides = sorted({x for m in msgs for x in (m.get("from"), m.get("to")) if x})
    cycle, last = current_cycle([m for m in msgs if m.get("date", "") <= close.get("date", "")])
    rounds = max([int(m["round"]) for m in last if m.get("round", "").isdigit()] or [1])
    requests = [sections(m.get("body", "")).get("User request", m.get("body", ""))
                for m in msgs if m.get("kind") == "guidance"]
    record = {
        "thread": args.thread, "subject": first.get("subject", "").removeprefix("Re: "),
        "projects": sides, "rounds": rounds, "cycle": cycle, "user_requests": requests,
        "outcome": close.get("kind"),
        "conclusion": sec.get("conclusion", ""), "justification": sec.get("justification", ""),
        "positions": sec.get("positions", ""), "decision_needed": sec.get("decision needed", ""),
        "after_approval": sec.get("after approval", ""), "open_points": sec.get("open points", ""),
        "messages": [m["id"] for m in msgs],
    }
    lines = [f"Discussed with: {', '.join(sides)} ({rounds} round(s)"
             + (f", cycle {cycle} after {cycle - 1} request(s) from the user)" if cycle > 1 else ")"),
             f"Subject: {record['subject']}",
             f"Outcome: {record['outcome']}", "",
             "Conclusion:", record["conclusion"], "",
             "Justification:", record["justification"]]
    if record["positions"]:
        lines += ["", "Positions:", record["positions"]]
    if record["open_points"] and record["open_points"].lower() != "none":
        lines += ["", "Open points:", record["open_points"]]
    lines += ["", "Decision needed:", record["decision_needed"], "",
              "After approval:", record["after_approval"]]
    emit(args, record, "\n".join(lines))


def guide(args, me: str, thread: str, text: str) -> list[dict]:
    """The user steps into a conversation: every side gets the request and the rounds restart."""
    msgs = conversation(me, thread)
    if not msgs:
        raise InboxError(f"thread {thread} is not known to {me}")
    cycle = current_cycle(msgs)[0] + 1
    sides = sorted({x for m in msgs for x in (m.get("from"), m.get("to")) if x and x != me})
    first = next((m for m in msgs if m["id"] == thread), msgs[0])
    subject = "User request: " + re.sub(r"^(Re:\s*)+", "", first.get("subject", ""))
    body = (f"### User request\n{text.strip()}\n\nThe user stepped into this conversation. "
            f"Cycle {cycle} starts: up to {MAX_ROUNDS} new rounds that take this request into "
            "account, closed by a new agreement or impasse for the user.\n")
    sent = [send(args, me, p, subject, "guidance", msgs[-1]["id"], thread, body=body, cycle=cycle)
            for p in sides]
    for m in msgs:
        if m.get("to") == me and m.get("kind") in CLOSING and m.get("status") != "done":
            update(me, m["id"], "done", f"user asked for changes: {text.strip()}")
    return sent


def cmd_guide(args) -> None:
    me = current_project(args.project)
    text = args.text or body_from(args)
    sent = guide(args, me, args.thread, text)
    emit(args, [public(m) for m in sent],
         "\n".join(f"sent {m['id']} to {m['to']} (cycle {m['cycle']})" for m in sent))


def cmd_decide(args) -> None:
    """Send the user's decision to every other side of the thread and close the record."""
    me = current_project(args.project)
    if args.revise:
        sent = guide(args, me, args.thread, args.revise)
        emit(args, [public(m) for m in sent],
             "\n".join(f"sent {m['id']} to {m['to']} (cycle {m['cycle']})" for m in sent))
        return
    close, msgs = closing_of(me, args.thread)
    sides = sorted({x for m in msgs for x in (m.get("from"), m.get("to")) if x and x != me})
    body = f"### Decision\n{args.choice}\n"
    if args.note:
        body += f"\n### Note\n{args.note}\n"
    body += "\nEach project records this decision in its own backlog.\n"
    first = next((m for m in msgs if m["id"] == args.thread), msgs[0])
    subject = "Decision: " + first.get("subject", "").removeprefix("Re: ")
    cycle = current_cycle(msgs)[0]
    sent = [send(args, me, p, subject, "decision", close["id"], args.thread, body=body,
                 cycle=cycle)
            for p in sides]
    if close.get("to") == me and close.get("status") != "done":
        update(me, close["id"], "done", f"user decided: {args.choice}")
    emit(args, [public(m) for m in sent],
         "\n".join(f"sent {m['id']} to {m['to']}" for m in sent))


def update(me: str, msg_id: str, status: str, note: str | None, deferred: str = "") -> dict:
    m = find(me, msg_id)
    m["status"] = status
    m["deferred"] = deferred
    if note:
        stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
        m["body"] = m["body"] + f"\n\n> {status} ({stamp}): {note}"
    write_files(me, {m["path"]: render(m)}, f"inbox({me}): {m['id']} {status}")
    return m


def cmd_status(args) -> None:
    me = current_project(args.project)
    m = update(me, args.id, args.status, args.note)
    emit(args, public(m), f"{m['id']}: {m['status']}")


def cmd_defer(args) -> None:
    me = current_project(args.project)
    m = update(me, args.id, "seen", f"deferred: {args.reason}", deferred=args.reason)
    emit(args, public(m), f"{m['id']}: deferred ({args.reason})")


def cmd_list(args) -> None:
    me = current_project(args.project)
    if args.sent or args.all:
        msgs = [m for p in projects() for m in read_inbox(p)]
        if args.sent:
            msgs = [m for m in msgs if m.get("from") == me]
    else:
        msgs = read_inbox(me)
    msgs.sort(key=lambda m: m.get("date", ""))
    emit(args, [public(m) for m in msgs], "\n".join(row(m) for m in msgs) or "no messages")


def cmd_watch(args) -> int:
    """Poll the listed projects through Git; on news, run the user's command in its folder."""
    targets = []
    for t in args.target:
        name, _, folder = t.partition("=")
        if name not in projects() or not folder:
            raise InboxError(f"--target must be <registered project>=<folder>, got '{t}'")
        targets.append((name, Path(folder).expanduser()))
    while True:
        for name, folder in targets:
            try:
                news, line = poll(name)
            except InboxError as exc:
                print(f"inbox: {exc}", file=sys.stderr)
                continue
            if news:
                print(line, flush=True)
                subprocess.run(args.exec, shell=True, cwd=folder,
                               env={**os.environ, "AURORA_PROJECT": name,
                                    "AURORA_INBOX_SUMMARY": line})
        if args.once:
            return 0
        time.sleep(args.interval)


def cmd_projects(args) -> None:
    known = projects()
    emit(args, known, "\n".join(f"- {k}: {v['name']} ({v['url']})" for k, v in known.items()))


def main() -> int:
    p = argparse.ArgumentParser(description="Provisional cross-project messages (aurora/inbox).")
    p.add_argument("--project")
    p.add_argument("--json", action="store_true")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("version").set_defaults(func=cmd_version)
    c = sub.add_parser("poll", help="exit 0 and silent when nothing changed, 10 on news")
    c.add_argument("--quiet", action="store_true")
    c.set_defaults(func=cmd_poll)
    c = sub.add_parser("check")
    c.add_argument("--quiet", action="store_true", help="print nothing when nothing is pending")
    c.add_argument("--no-sent", action="store_true", help="skip the unanswered sent messages")
    c.add_argument("--stale-hours", type=float, default=48.0)
    c.set_defaults(func=cmd_check)
    s = sub.add_parser("show")
    s.add_argument("id")
    s.set_defaults(func=cmd_show)
    s = sub.add_parser("template")
    s.add_argument("kind", choices=list(TEMPLATES))
    s.set_defaults(func=cmd_template)
    s = sub.add_parser("send")
    s.add_argument("--to", required=True)
    s.add_argument("--subject", required=True)
    s.add_argument("--kind", default="question", choices=OPENING)
    s.add_argument("--body-file")
    s.set_defaults(func=cmd_send)
    s = sub.add_parser("reply")
    s.add_argument("id")
    s.add_argument("--kind", required=True, choices=REPLIES)
    s.add_argument("--subject")
    s.add_argument("--body-file")
    s.set_defaults(func=cmd_reply)
    s = sub.add_parser("record")
    s.add_argument("thread")
    s.set_defaults(func=cmd_record)
    s = sub.add_parser("decide")
    s.add_argument("thread")
    g = s.add_mutually_exclusive_group(required=True)
    g.add_argument("--choice")
    g.add_argument("--revise", help="the user is not satisfied: what to change; rounds restart")
    s.add_argument("--note")
    s.set_defaults(func=cmd_decide)
    s = sub.add_parser("guide", help="the user steps into a conversation at any time")
    s.add_argument("thread")
    s.add_argument("--text")
    s.add_argument("--body-file")
    s.set_defaults(func=cmd_guide)
    s = sub.add_parser("status")
    s.add_argument("id")
    s.add_argument("status", choices=STATUSES)
    s.add_argument("--note")
    s.set_defaults(func=cmd_status)
    s = sub.add_parser("defer")
    s.add_argument("id")
    s.add_argument("--reason", required=True)
    s.set_defaults(func=cmd_defer)
    s = sub.add_parser("list")
    s.add_argument("--sent", action="store_true")
    s.add_argument("--all", action="store_true")
    s.set_defaults(func=cmd_list)
    s = sub.add_parser("watch")
    s.add_argument("--target", action="append", required=True, help="<project>=<folder>")
    s.add_argument("--exec", required=True, help="local command run in the folder on news")
    s.add_argument("--interval", type=float, default=300.0)
    s.add_argument("--once", action="store_true")
    s.set_defaults(func=cmd_watch)
    sub.add_parser("projects").set_defaults(func=cmd_projects)
    args = p.parse_args()
    try:
        return args.func(args) or 0
    except InboxError as exc:
        print(f"inbox: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
