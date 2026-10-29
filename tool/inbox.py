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
    check   [--quiet]                         open and seen messages for this project
    show    <id>
    send    --to P --subject S [--body-file F | stdin]
    reply   <id> [--subject S] [--body-file F | stdin]
    status  <id> open|seen|done [--note TEXT]
    defer   <id> --reason TEXT                leave it pending, with the reason
    list    [--sent] [--all]
    projects

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

BRANCH = "aurora/inbox"
REF = f"refs/heads/{BRANCH}"
HERE = Path(__file__).resolve().parent
REGISTRY = HERE / "projects.json"
CACHE = Path(os.environ.get("AURORA_INBOX_CACHE", Path.home() / ".cache" / "aurora-inbox"))
STATUSES = ("open", "seen", "done")
FIELDS = ("id", "kind", "from", "to", "date", "subject", "status", "reply_to", "deferred")
IDENTITY = {
    "GIT_AUTHOR_NAME": os.environ.get("AURORA_INBOX_GIT_NAME", "LeviGuilherme"),
    "GIT_AUTHOR_EMAIL": os.environ.get(
        "AURORA_INBOX_GIT_EMAIL", "56275425+LeviGuilherme@users.noreply.github.com"),
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

def cmd_check(args) -> None:
    me = current_project(args.project)
    pending = [m for m in read_inbox(me) if m.get("status") != "done"]
    if not pending:
        if not args.quiet:
            emit(args, {"project": me, "messages": []}, f"Inbox ({me}): nothing pending.")
        return
    tool = Path(__file__).resolve()
    text = [f"Inbox ({me}): {len(pending)} message(s) pending."]
    text += [row(m) for m in pending]
    text += ["",
             "How to handle them:",
             f"1. Read each one: python3 {tool} show <id>",
             "2. If it belongs to the work in progress: handle it, answer with 'reply <id>',",
             "   then 'status <id> done'.",
             "3. If it does not: do not act on it. Run 'defer <id> --reason \"...\"' and tell the",
             "   user (or whoever triages this project's work), who decides what to do.",
             "4. Never set 'done' without handling the message."]
    emit(args, {"project": me, "messages": [public(m) for m in pending]}, "\n".join(text))


def cmd_show(args) -> None:
    me = current_project(args.project)
    m = find(me, args.id)
    if m.get("status") == "open":
        m["status"] = "seen"
        write_files(me, {m["path"]: render(m)}, f"inbox({me}): {m['id']} seen")
    emit(args, public(m, True), render(m))


def send(args, me: str, to: str, subject: str, reply_to: str = "") -> dict:
    if to not in projects():
        raise InboxError(f"unknown project '{to}' ({', '.join(sorted(projects()))})")
    if to == me:
        raise InboxError("a project cannot send a message to itself")
    now = dt.datetime.now(dt.timezone.utc).replace(microsecond=0)
    meta = {"id": f"{now:%Y%m%d-%H%M%S}-{me}-{slug(subject)}", "kind": "message",
            "from": me, "to": to, "date": now.isoformat().replace("+00:00", "Z"),
            "subject": subject, "status": "open", "reply_to": reply_to,
            "body": body_from(args)}
    write_files(to, {f"messages/{meta['id']}.md": render(meta)},
                f"inbox({me} -> {to}): {subject}")
    return meta


def cmd_send(args) -> None:
    me = current_project(args.project)
    m = send(args, me, args.to, args.subject)
    emit(args, public(m), f"sent {m['id']} to {m['to']}")


def cmd_reply(args) -> None:
    me = current_project(args.project)
    orig = find(me, args.id)
    m = send(args, me, orig["from"], args.subject or f"Re: {orig.get('subject')}", orig["id"])
    emit(args, public(m), f"sent {m['id']} to {m['to']}")


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


def cmd_projects(args) -> None:
    known = projects()
    emit(args, known, "\n".join(f"- {k}: {v['name']} ({v['url']})" for k, v in known.items()))


def main() -> int:
    p = argparse.ArgumentParser(description="Provisional cross-project messages (aurora/inbox).")
    p.add_argument("--project")
    p.add_argument("--json", action="store_true")
    sub = p.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check")
    c.add_argument("--quiet", action="store_true", help="print nothing when nothing is pending")
    c.set_defaults(func=cmd_check)
    s = sub.add_parser("show")
    s.add_argument("id")
    s.set_defaults(func=cmd_show)
    s = sub.add_parser("send")
    s.add_argument("--to", required=True)
    s.add_argument("--subject", required=True)
    s.add_argument("--body-file")
    s.set_defaults(func=cmd_send)
    s = sub.add_parser("reply")
    s.add_argument("id")
    s.add_argument("--subject")
    s.add_argument("--body-file")
    s.set_defaults(func=cmd_reply)
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
    sub.add_parser("projects").set_defaults(func=cmd_projects)
    args = p.parse_args()
    try:
        args.func(args)
    except InboxError as exc:
        print(f"inbox: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
