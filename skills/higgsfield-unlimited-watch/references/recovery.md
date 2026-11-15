# State, events and recovery

Every job uses its own local state directory. Keep it outside Git or under ignored `local/`.

| File | Purpose |
|---|---|
| target.json | Immutable asset binding for that directory |
| monitor.lock | Exclusive monitor PID and start time |
| status.json | Most recent compact observation |
| transitions.jsonl | State changes, not every unchanged poll |
| event.json | Terminal monitor result, stable event ID |
| relay-TARGET-EVENT.json | Send attempt and transport result |

`completed` / `failed` are job observations. `timeout` means no terminal result was established before the monitor deadline. `cancelled` means Ctrl+C stopped the monitor, not the remote job. All unknown states block a new submission until reconciled.

## Monitor recovery

1. Read event.json first. A repeated watch returns an existing completed/failed event without polling again.
2. After timeout/cancel, restore the correct browser/tab and visible history card. Update the local config if browser IDs changed. Reuse the same asset and directory with `--resume` to start a new monitoring attempt. Never regenerate as a monitoring repair.
3. A stale monitor.lock deliberately blocks restart. Read its PID, confirm that process has stopped, then remove only that lock. Never remove a live process's lock. If a PID has been reused, inspect process identity and start time too.
4. Missing, conflicting or separate duplicate data-asset-id cards remain unknown. Nested wrappers collapse only when there is one same-ID root and every status agrees. The watcher never blindly selects the first match. Virtualization can hide a result; bring the correct card into view before resuming.

The file lock coordinates users of this state directory on one local filesystem. It does not enforce global account concurrency, deduplicate different directories or synchronize different computers.

## Process supervision (Rust only, opt-in)

Ordinary `watch` does not restart after process exit. If no event exists, inspect local process/lock/state before deliberately resuming the exact asset. A missing event is not failed generation. Suspend pauses local observation; resume may reveal unknown/timeout. Live suspend/crash recovery remains unverified.

```text
huw supervise --config config.local.json --asset-id RESULT_UUID --state-dir local/job-001 --max-restarts 2 --backoff 5
```

The supervisor holds supervisor.lock and starts its own watch child. Only failed local children are restarted (0–10 restarts, bounded exponential backoff capped at 300s), preserving the exact asset/directory and `--resume`. An existing terminal event is reused. Completed/failed/timeout/cancelled monitor events are not generation retries. supervisor.json records running/finished/restart_budget_exhausted; supervisor-transitions.jsonl records child exits. Exhaustion exits nonzero for the caller to handle.

Automatic monitor.lock cleanup requires a spawned-and-reaped child handle plus matching PID and supervisor token. Foreign/legacy locks require manual reconciliation; a PID alone is insufficient. The supervisor itself is not installed as an OS service and is not restarted automatically. No browser/machine restart or generation resubmission occurs.

Rust `watch`, `relay`, and `supervise` support `--relay-wait SECONDS` (default 0). This defers delivery only while the exact recipient/app is live and working. Blocked/error/exited/wrong-app stops immediately; expiration leaves the event for manual relay. Python retains immediate readiness checks and manual recovery. No force-send is used.

## Relay recovery

The relay requires `--notify-app codex` or `--notify-app copilot` and verifies the returned terminal address and exact app identity, as well as live idle/done state. A different supported agent is still a mismatch. If not ready or mismatched, it stops without a send receipt; use `relay` once the correct agent is ready. The target can still change after the check; this is a point-in-time verification, not an atomic guarantee that the same process receives the subsequent send.

The relay writes an `attempting` receipt before calling `hc send`. A successful exit becomes `accepted` unless the response explicitly reports `started=false`; that response or a transport failure/timeout becomes `delivery_unknown`. All existing receipts suppress automatic resend. A crash can leave `attempting` even if a message arrived. This is one attempted send, not exactly-once processing.

Receipts record the expected app. A receipt from v0.1.0a1 has no app binding and is not automatically resent or silently migrated. Inspect the prior delivery and explicitly archive that receipt only when a resend is appropriate.

Inspect the receiving conversation before manual recovery. If delivery is confirmed absent, archive the specific receipt and invoke `relay` deliberately. If uncertain, retain it and review the output manually. Reusing a completed job does not authorize a new generation. An agent turn, browser commands, and completion review can still consume tokens: only the local watch loop contains no LLM calls.

## Privacy and trust

On POSIX, new state directories (including newly created parents) use mode `0700`; JSON, temporary files, journals and locks use `0600`. An existing state directory must already be owned by the current user with mode `0700`, otherwise the helper refuses it without changing its permissions. Choose a new private job directory, or explicitly secure a known dedicated state directory; do not chmod a shared working directory. Atomic replacement and journal writes secure only their own target files. Symlink state directories and journals are rejected.

Windows uses inherited filesystem ACLs; POSIX mode bits are not a Windows privacy guarantee. Put state in a user-private folder and manage ACLs through Windows. POSIX permission tests run on Linux; they are skipped on Windows. WSL-mounted Windows filesystems may not support the required POSIX modes; use the Linux home/tmp filesystem when running the helper inside WSL.

The helper does not request cookies, browser storage, tokens, private APIs, or downloaded media. It reads rendered DOM through terminal-browser. Normal inspection outputs a prompt hash/length and reference count, not prompt text or URLs. Do not attach raw DOM, browser profiles, config files, state directories, or company assets to issues. The browser itself still has a logged-in session; protect it normally.

Config files choose executables and are trusted local input. Notification events are local data, not browser-supplied instructions. Model-generated follow-up actions still need the user's task authorization.
