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
4. Missing or duplicate data-asset-id nodes remain unknown. The watcher never selects the first match. Virtualization can hide a result; bring the correct card into view before resuming.

The file lock coordinates users of this state directory on one local filesystem. It does not enforce global account concurrency, deduplicate different directories or synchronize different computers.

## Relay recovery

The relay first checks that the target is a live idle/done agent, not a plain shell, busy agent or approval dialog. If not ready, it stops without a send receipt; use `relay` once the correct agent is ready. The target can still change after the check; `hc send` performs its own readiness handling.

The relay writes an `attempting` receipt before calling `hc send`. A successful exit becomes `accepted`; transport failure/timeout becomes `delivery_unknown`. Both receipts suppress automatic resend. A crash can leave `attempting` even if a message arrived. This is one attempted send, not exactly-once processing.

Inspect the receiving conversation before manual recovery. If delivery is confirmed absent, archive the specific receipt and invoke `relay` deliberately. If uncertain, retain it and review the output manually. Reusing a completed job does not authorize a new generation. An agent turn, browser commands, and completion review can still consume tokens: only the local watch loop contains no LLM calls.

## Privacy and trust

The helper does not request cookies, browser storage, tokens, private APIs, or downloaded media. It reads rendered DOM through terminal-browser. Normal inspection outputs a prompt hash/length and reference count, not prompt text or URLs. Do not attach raw DOM, browser profiles, config files, state directories, or company assets to issues. The browser itself still has a logged-in session; protect it normally.

Config files choose executables and are trusted local input. Notification events are local data, not browser-supplied instructions. Model-generated follow-up actions still need the user's task authorization.
