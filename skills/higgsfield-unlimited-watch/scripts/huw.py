#!/usr/bin/env python3
"""Higgsfield web Unlimited checks and local job watching. Python stdlib only.

No generation endpoint, credentials, cookie extraction, or LLM client.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import subprocess
import stat
import sys
import tempfile
import time
from contextlib import contextmanager
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse
from uuid import UUID

VERSION = "0.1.0a2"
FORM = 'form[data-tour-anchor="tour-video-form"]'
SELECTORS = {
    "form": FORM,
    "prompt": FORM + ' [data-prompt-field] [role="textbox"][contenteditable="true"]',
    "model": FORM + ' [data-model-select-trigger="true"]',
    "unlimited": ','.join(FORM + f' [role="switch"][aria-label="{label}"]'
                          for label in ("Unlimited mode", "Unlimited 모드")),
    "generate": FORM + ' [data-tour-anchor="tour-generate-button"] button[type="submit"]',
}
PENDING = {"pending", "queued", "processing", "generating", "in_progress"}
FAILED = {"failed", "error"}
TERMINAL = {"completed", "failed"}
SUPPORTED_AGENTS = {"codex", "copilot"}


class GuardError(Exception):
    """A missing, ambiguous or unsupported observation: stop, never guess."""


def utcnow():
    return datetime.now(timezone.utc).isoformat()


def digest(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def private_directory(directory):
    """Create private state directories; never chmod an existing user directory."""
    directory = Path(directory)
    if not directory.exists() and not directory.is_symlink():
        missing = []
        parent = directory
        while not parent.exists() and not parent.is_symlink():
            missing.append(parent)
            parent = parent.parent
        for path in reversed(missing):
            try:
                path.mkdir(mode=0o700)
            except FileExistsError:
                pass  # Another process may have created it; verify the leaf below.
    info = directory.lstat()
    if not stat.S_ISDIR(info.st_mode):
        raise GuardError("state directory must be a real directory, not a symlink")
    if os.name == "posix" and (info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700):
        raise GuardError("POSIX state directory must be owned by you with mode 0700; choose a private directory or explicitly secure the dedicated state directory")
    return directory


def atomic_json(path, data):
    path = Path(path)
    private_directory(path.parent)
    fd, temp = tempfile.mkstemp(prefix="." + path.name + ".", suffix=".tmp", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            if os.name == "posix":
                os.fchmod(f.fileno(), 0o600)
            json.dump(data, f, ensure_ascii=False, indent=2)
            f.write("\n")
            f.flush()
            os.fsync(f.fileno())
        os.replace(temp, path)
    finally:
        Path(temp).unlink(missing_ok=True)


def append_transition(path, data):
    path = Path(path)
    private_directory(path.parent)
    if path.is_symlink():
        raise GuardError("transition journal must not be a symlink")
    flags = os.O_WRONLY | os.O_APPEND | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0)
    fd = os.open(path, flags, 0o600)
    with os.fdopen(fd, "a", encoding="utf-8") as f:
        if not stat.S_ISREG(os.fstat(f.fileno()).st_mode):
            raise GuardError("transition journal must be a regular file")
        if os.name == "posix":
            os.fchmod(f.fileno(), 0o600)
        f.write(json.dumps(data, ensure_ascii=False) + "\n")


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def asset_uuid(value):
    try:
        if str(UUID(value)) != value.lower():
            raise ValueError()
    except (ValueError, AttributeError):
        raise GuardError("asset_id must be a canonical UUID") from None
    return value.lower()


class Node:
    def __init__(self, tag, attrs=(), parent=None):
        self.tag, self.attrs, self.parent = tag, dict(attrs), parent
        self.children, self.parts = [], []

    def text(self):
        # Strip once at the boundary; preserve spaces across inline elements.
        return "".join(p.raw_text() if isinstance(p, Node) else p for p in self.parts).strip()

    def raw_text(self):
        return "".join(p.raw_text() if isinstance(p, Node) else p for p in self.parts)

    def walk(self):
        yield self
        for child in self.children:
            yield from child.walk()


class DOM(HTMLParser):
    VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link",
            "meta", "param", "source", "track", "wbr"}

    def __init__(self, html):
        super().__init__(convert_charrefs=True)
        self.root = Node("root")
        self.stack = [self.root]
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        node = Node(tag, attrs, self.stack[-1])
        self.stack[-1].children.append(node)
        self.stack[-1].parts.append(node)
        if tag not in self.VOID:
            self.stack.append(node)

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in self.VOID:
            self.handle_endtag(tag)

    def handle_endtag(self, tag):
        for i in range(len(self.stack) - 1, 0, -1):
            if self.stack[i].tag == tag:
                self.stack = self.stack[:i]
                break

    def handle_data(self, data):
        self.stack[-1].parts.append(data)


def normalize_newlines(text):
    return text.replace("\r\n", "\n").replace("\r", "\n")


def prompt_text(node):
    """Lexical plain text: preserve spaces/tabs and logical line boundaries.

    An otherwise empty editor/paragraph's sole br is an editor placeholder.
    Adjacent blocks each contribute a boundary, including empty paragraphs.
    """
    blocks = {"p", "div", "li", "pre", "blockquote", "h1", "h2", "h3", "h4", "h5", "h6"}
    if node.tag == "br":
        return "\n"
    if (node.tag in blocks or node.attrs.get("role") == "textbox") and len(node.parts) == 1:
        part = node.parts[0]
        if isinstance(part, Node) and part.tag == "br":
            return ""
    result = []
    previous_block = False
    for index, part in enumerate(node.parts):
        is_block = isinstance(part, Node) and part.tag in blocks
        if index and (previous_block or is_block):
            result.append("\n")
        result.append(prompt_text(part) if isinstance(part, Node) else part)
        previous_block = is_block
    return normalize_newlines("".join(result))


def one(nodes, predicate, label):
    matches = [n for n in nodes if predicate(n)]
    if len(matches) != 1:
        raise GuardError(f"selector drift: {label}: expected 1, found {len(matches)}")
    return matches[0]


def disabled(node):
    return "disabled" in node.attrs or node.attrs.get("aria-disabled") == "true"


def form_state(html):
    """Parse only the creation form's inner HTML, never history textboxes."""
    nodes = list(DOM(html).root.walk())
    prompt = one(nodes, lambda n: n.attrs.get("role") == "textbox" and
                 n.attrs.get("data-lexical-editor") == "true", "prompt")
    switch = one(nodes, lambda n: n.attrs.get("role") == "switch" and
                 n.attrs.get("aria-label") in ("Unlimited mode", "Unlimited 모드"), "Unlimited")
    submit = one(nodes, lambda n: n.tag == "button" and n.attrs.get("type") == "submit", "submit")
    model = one(nodes, lambda n: n.attrs.get("data-model-select-trigger") == "true", "model")
    settings = {n.attrs.get("aria-label", "unknown"): n.text() for n in nodes
                if "data-form-kit-setting-button" in n.attrs}
    refs = [x for n in nodes if n.attrs.get("data-tour-anchor") == "tour-video-reference"
            for x in n.walk() if x.tag == "img"]
    label = re.sub(r"\s+", "", submit.text())
    ready = (switch.attrs.get("aria-checked") == "true" and not disabled(switch)
             and label in {"GenerateUnlimited", "생성하기무제한"} and not disabled(submit))
    return {
        "unlimited": switch.attrs.get("aria-checked"), "switch_disabled": disabled(switch),
        "submit_label": submit.text(), "submit_disabled": disabled(submit),
        "unlimited_verified": ready, "model": model.text(), "settings": settings,
        "prompt_sha256": digest(prompt_text(prompt)), "prompt_length": len(prompt_text(prompt)),
        "reference_count": len(refs),
    }


def history_state(html, asset_id):
    nodes = [n for n in DOM(html).root.walk() if n.attrs.get("data-asset-id") == asset_id]
    if len(nodes) != 1:
        return {"state": "unknown", "reason": "asset_missing_or_ambiguous", "matches": len(nodes)}
    node = nodes[0]
    raw = node.attrs.get("data-job-status")
    video = any(n.tag == "video" for n in node.walk())
    state = ("pending" if raw in PENDING else "failed" if raw in FAILED else
             "completed" if raw == "completed" and video else "unknown")
    return {"state": state, "asset_id": asset_id, "dom_status": raw,
            "video_present": video, "reason": "dom_status"}


class Browser:
    """Shell-free transport to an existing terminal-browser session."""
    def __init__(self, config):
        prefix = config.get("command")
        if not isinstance(prefix, list) or not prefix or not all(isinstance(s, str) and s for s in prefix):
            raise GuardError("config.command must be a nonempty executable/argument array")
        self.prefix = prefix
        self.browser = str(config["browser"])
        self.tab = str(config.get("tab", "1"))
        self.timeout = float(config.get("command_timeout_seconds", 30))
        if not 1 <= self.timeout <= 120:
            raise GuardError("command_timeout_seconds must be 1..120")

    def action(self, *args):
        try:
            result = subprocess.run(self.prefix + ["action", "--browser", self.browser,
                                    "--tab", self.tab, "--", *args],
                                    capture_output=True, text=True, encoding="utf-8",
                                    timeout=self.timeout, shell=False,
                                    creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        except (OSError, subprocess.TimeoutExpired) as exc:
            # Never echo raw page text, command prompts, or private resource URLs.
            raise GuardError("browser_transport_" + type(exc).__name__) from None
        if result.returncode:
            raise GuardError(f"browser command failed (exit {result.returncode}); inspect the session locally")
        return result.stdout.strip()

    def check_origin(self):
        value = self.action("get", "url")
        url = urlparse(value)
        if url.scheme != "https" or url.hostname not in {"higgsfield.ai", "www.higgsfield.ai"}:
            raise GuardError("expected an HTTPS Higgsfield tab")

    def unique(self, key):
        try:
            count = int(self.action("get", "count", SELECTORS[key]))
        except ValueError:
            raise GuardError("invalid selector count") from None
        if count != 1:
            raise GuardError(f"selector drift: {key}: expected 1, found {count}")

    def inspect(self):
        self.check_origin()
        for key in SELECTORS:
            self.unique(key)
        return form_state(self.action("get", "html", FORM))

    def ensure_unlimited(self):
        state = self.inspect()
        if state["switch_disabled"]:
            raise GuardError("Unlimited switch is disabled")
        if state["unlimited"] == "false":
            self.action("click", SELECTORS["unlimited"])
        state = self.inspect()
        if not state["unlimited_verified"]:
            raise GuardError("Unlimited switch and submit label do not both confirm availability")
        return state

    def replace_prompt(self, prompt):
        prompt = normalize_newlines(prompt)
        self.inspect()
        self.action("click", SELECTORS["prompt"])
        self.action("press", "Control+a")
        self.action("press", "Backspace")
        if self.inspect()["prompt_length"] != 0:
            raise GuardError("prompt did not clear; no text was appended")
        self.action("fill", SELECTORS["prompt"], prompt)
        state = self.inspect()
        if state["prompt_sha256"] != digest(prompt):
            raise GuardError("prompt readback differs; inspect editor before generation")
        return state

    def observe(self, asset_id):
        self.check_origin()
        return history_state(self.action("get", "html", "main"), asset_id)


@contextmanager
def lock(directory, name="monitor.lock"):
    directory = private_directory(directory)
    path = directory / name
    try:
        fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError:
        raise GuardError(f"{name} exists: inspect its PID before recovery") from None
    try:
        with os.fdopen(fd, "w") as f:
            if os.name == "posix":
                os.fchmod(f.fileno(), 0o600)
            json.dump({"pid": os.getpid(), "created_at": utcnow()}, f)
        yield
    finally:
        path.unlink(missing_ok=True)


def watch(observe, asset_id, directory, *, interval=20, timeout=3600,
          confirmations=2, resume=False, clock=time.monotonic, sleep=time.sleep):
    """Persist an exact-asset event. Repeated invocation returns the original event.

    The caller supplies a local observer. No model calls occur inside this loop.
    """
    asset_id = asset_uuid(asset_id)
    if not math.isfinite(interval) or not math.isfinite(timeout) or interval < 5 or timeout <= 0 or confirmations < 2:
        raise GuardError("interval >= 5, timeout > 0, confirmations >= 2 required")
    directory = Path(directory)
    with lock(directory):
        target = directory / "target.json"
        if target.exists() and read_json(target).get("asset_id") != asset_id:
            raise GuardError("state directory belongs to a different asset")
        atomic_json(target, {"asset_id": asset_id})
        event_path = directory / "event.json"
        previous_event = read_json(event_path) if event_path.exists() else None
        if previous_event and (previous_event["state"] in TERMINAL or not resume):
            return previous_event
        attempt = (previous_event or {}).get("attempt", 0) + 1
        started = clock()
        polls, consecutive, last, previous = 0, 0, None, None
        event = None
        try:
            while True:
                polls += 1
                try:
                    observation = observe(asset_id)
                except GuardError:
                    observation = {"state": "unknown", "reason": "browser_unavailable_or_changed"}
                signature = (observation["state"], observation.get("dom_status"), observation.get("reason"))
                consecutive = consecutive + 1 if signature == last else 1
                last = signature
                current = dict(observation, asset_id=asset_id, observed_at=utcnow(), polls=polls,
                               consecutive_observations=consecutive, attempt=attempt,
                               elapsed_seconds=round(clock() - started, 3))
                atomic_json(directory / "status.json", current)
                if signature != previous:
                    append_transition(directory / "transitions.jsonl", current)
                    previous = signature
                if observation["state"] in TERMINAL and consecutive >= confirmations:
                    event = current
                    break
                if clock() - started >= timeout:
                    event = dict(current, state="timeout", reason="deadline_not_job_failure")
                    break
                sleep(min(interval, max(0, timeout - (clock() - started))))
        except KeyboardInterrupt:
            event = {"state": "cancelled", "reason": "monitor_stopped_job_not_cancelled",
                     "asset_id": asset_id, "attempt": attempt, "polls": polls, "observed_at": utcnow()}
        event["schema_version"] = 1
        event["event_id"] = digest(f"{asset_id}:{attempt}:{event['state']}")[:24]
        event["monitor_llm_calls"] = 0
        atomic_json(event_path, event)
        return event


def relay(directory, hc, terminal, *, expected_app, runner=subprocess.run):
    """One send attempt per event/target. Crash-ambiguous attempts are not retried."""
    if not re.fullmatch(r"t[0-9a-z]+", terminal):
        raise GuardError("notify target must be an observed local hc terminal ID")
    if expected_app not in SUPPORTED_AGENTS:
        raise GuardError("notify app must be an explicitly supported agent: codex or copilot")
    directory = private_directory(directory)
    event = read_json(directory / "event.json")
    if event.get("state") not in TERMINAL | {"timeout", "cancelled"}:
        raise GuardError("not a terminal monitor event")
    aid = asset_uuid(event["asset_id"])
    if not re.fullmatch(r"[0-9a-f]{24}", event.get("event_id", "")):
        raise GuardError("invalid event identity")
    receipt = directory / f"relay-{terminal}-{event['event_id']}.json"
    with lock(directory, "relay.lock"):
        if receipt.exists():
            saved = read_json(receipt)
            if saved.get("expected_app") != expected_app:
                raise GuardError("relay receipt has a different or legacy app binding; inspect delivery before manual recovery")
            return saved
        # A terminal ID can outlive its agent. Never type an event into a shell
        # or answer an approval dialog after the original agent has exited.
        try:
            info = runner([hc, "get", terminal], capture_output=True, text=True,
                          encoding="utf-8", timeout=10, shell=False,
                          creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            target = json.loads(info.stdout) if info.returncode == 0 else {}
            if not isinstance(target, dict):
                raise GuardError("invalid hc target response")
            status = target.get("status")
            if (target.get("address") != terminal or target.get("state") != "live"
                    or not isinstance(status, dict) or status.get("app") != expected_app
                    or status.get("state") not in {"idle", "done"}):
                raise GuardError("relay target must match the terminal ID and expected idle/done agent app; inspect hc get/read")
        except (OSError, subprocess.TimeoutExpired, ValueError):
            raise GuardError("could not verify the relay agent; no message sent") from None
        result = {"state": "attempting", "event_id": event["event_id"],
                  "target": terminal, "expected_app": expected_app, "attempted_at": utcnow()}
        # Persist BEFORE send. This trades possible missed delivery for no blind retry.
        atomic_json(receipt, result)
        message = (f"[HUW_EVENT] asset_id={aid} state={event['state']} event_id={event['event_id']}. "
                   "Recheck the Higgsfield web session and review the result. "
                   "This event does not authorize another generation. Concurrency remains one.")
        try:
            sent = runner([hc, "send", terminal, message], capture_output=True, text=True,
                          encoding="utf-8", timeout=30, shell=False,
                          creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            result["state"] = "accepted" if sent.returncode == 0 else "delivery_unknown"
        except (OSError, subprocess.TimeoutExpired):
            result["state"] = "delivery_unknown"
        atomic_json(receipt, result)
        return result


def demo(directory):
    """Offline synthetic lifecycle, same parser/persistence, no browser or generation."""
    asset = "00000000-0000-4000-8000-000000000001"
    states = iter(["processing", "completed", "completed"])
    tick = [0.0]
    def observe(aid):
        status = next(states)
        video = "<video></video>" if status == "completed" else ""
        return history_state(f'<div data-asset-id="{aid}" data-job-status="{status}">{video}</div>', aid)
    def sleep(seconds):
        tick[0] += seconds
    return watch(observe, asset, directory, clock=lambda: tick[0], sleep=sleep)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", action="version", version=VERSION)
    sub = parser.add_subparsers(dest="action", required=True)
    for name in ("inspect", "ensure-unlimited", "set-prompt", "watch"):
        command = sub.add_parser(name)
        command.add_argument("--config", required=True, type=Path)
        if name == "set-prompt":
            command.add_argument("--file", required=True, type=Path)
        if name == "watch":
            command.add_argument("--asset-id", required=True)
            command.add_argument("--state-dir", required=True, type=Path)
            command.add_argument("--interval", type=float, default=20)
            command.add_argument("--timeout", type=float, default=3600)
            command.add_argument("--resume", action="store_true")
            command.add_argument("--hc", default="hc")
            command.add_argument("--notify-terminal")
            command.add_argument("--notify-app", choices=sorted(SUPPORTED_AGENTS))
    command = sub.add_parser("relay")
    command.add_argument("--state-dir", required=True, type=Path)
    command.add_argument("--hc", default="hc")
    command.add_argument("--notify-terminal", required=True)
    command.add_argument("--notify-app", required=True, choices=sorted(SUPPORTED_AGENTS))
    command = sub.add_parser("demo")
    command.add_argument("--state-dir", type=Path, default=Path("local/demo"))
    args = parser.parse_args(argv)
    if args.action == "watch" and bool(args.notify_terminal) != bool(args.notify_app):
        parser.error("--notify-terminal and --notify-app must be supplied together")
    try:
        if args.action == "demo":
            output = {"synthetic": True, "event": demo(args.state_dir)}
        elif args.action == "relay":
            output = relay(args.state_dir, args.hc, args.notify_terminal, expected_app=args.notify_app)
        else:
            browser = Browser(read_json(args.config))
            if args.action == "inspect":
                output = browser.inspect()
            elif args.action == "ensure-unlimited":
                output = browser.ensure_unlimited()
            elif args.action == "set-prompt":
                prompt = normalize_newlines(args.file.read_bytes().decode("utf-8-sig"))
                if not prompt.strip():
                    raise GuardError("prompt file is empty")
                output = browser.replace_prompt(prompt)
            else:
                output = watch(browser.observe, args.asset_id, args.state_dir,
                               interval=args.interval, timeout=args.timeout, resume=args.resume)
                if args.notify_terminal:
                    output = {"event": output, "relay": relay(args.state_dir, args.hc, args.notify_terminal, expected_app=args.notify_app)}
        print(json.dumps(output, ensure_ascii=False, indent=2))
        return 0
    except (GuardError, OSError, ValueError, KeyError) as exc:
        # Errors are operational, never an instruction to retry a generation.
        print(json.dumps({"error": str(exc), "generation_submitted": False}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
