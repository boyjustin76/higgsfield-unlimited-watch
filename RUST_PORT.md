# Rust migration checkpoint

Baseline: public main edc525dcf62c3e3d0678db1a3be2f54a1d2973df.
Local branch only; do not push, publish, replace the shared Python installation,
or submit generation during migration verification.

## Compatibility contract

- Preserve inspect, ensure-unlimited, set-prompt, watch, relay, demo and --version.
- Config command array/browser/tab/command_timeout_seconds unchanged. Default video;
  additive media_kind=image supports the observed image form.
- Preserve target.json/status.json/transitions.jsonl/event.json and relay receipts,
  SHA256 event identity, attempt/resume semantics, 2 confirmations, private state,
  exclusive monitor, no blind retry after ambiguous send, exact recipient/app binding.
- Add settings completeness instead of treating an empty object as validation.
- Nested same-ID wrappers can collapse only when one root and statuses agree.
- Image completion additionally requires rendered result image loaded dimensions.
- Supervision must be opt-in, bounded, resume same asset only, never generate or
  restart user browser. Stale lock recovery requires process-identity evidence;
  never erase a lock based solely on a reused PID.

## Modules and five commits

1. Cargo scaffold, compatibility map and migration plan (this checkpoint).
2. dom + transport + browser inspect/ensure; origin/selector/label guards.
3. set-prompt; exact whitespace, BOM/newline normalization, clear/readback guards.
4. storage + monitor + relay; timeout/cancel/resume/replay, recovery supervision.
5. image form/results; expected prepared-form snapshot, full regression matrix,
   read-only live inspection, final handoff.

## Validation obligations

Port every test scenario from tests/test_huw.py and tests/test_permissions.py.
Keep Python baseline tests intact as an independent reference. Rust tests must
cover process arguments/timeouts, privacy, failed clear, receipt-before-send,
readiness/app mismatch, recovery and the local image-watcher scenarios.
Windows cannot prove POSIX permissions: execute Unix tests on a real Unix FS or
report them as unverified. Live browser tests limited to inspect; no ensure,
set-prompt, submit or live notification tests during this task.

## Evidence

- Existing SKILL.md, references/setup.md and recovery.md.
- Public issues #1–4; local image path supplied by the task owner.
- scraper API: https://docs.rs/scraper/latest/scraper/ and upstream
  https://github.com/rust-scraper/scraper (Html/Selector/ElementRef).
- Toolchain verified: cargo/rustc 1.99.1, Windows user installation.

## Current progress

Stages 1–5 implemented locally. Stage 1 b584482; stage 2 d56b6b8;
stage 3 2022691; stage 4 0425f1d; stage 5 is this commit. Do not deploy or
publish on the strength of a local pass; review the limitations below.


## Running the Rust CLI

```
cargo test
cargo build --release
huw inspect --config config.local.json
huw ensure-unlimited --config config.local.json
huw set-prompt --config config.local.json --file prompt.txt
huw watch --config config.local.json --asset-id UUID --state-dir local/job
huw relay --state-dir local/job --hc hc --notify-terminal t17 --notify-app codex
huw demo --state-dir local/offline-demo
```

All old commands/options and file names are retained. New fields in output are
additive. `settings_complete`/`missing_settings` report incomplete observations;
Unlimited verification only verifies the switch and submit control. A missing
Audio setting remains unknown: an `enhancePrompt` checkbox is not audio evidence.
New tabs must always be inspected and ensured independently before a separately
authorized generation. No CLI subcommand submits generation.

Image config: add `"media_kind": "image"` to the existing config JSON (default is
video). `inspect`, `ensure-unlimited`, `set-prompt`, `watch`, `relay` also work for
images. Additive `ensure --config CONFIG --expected EXPECTED` is read-only and
compares model/resolution/ratio/batch/reference_ids/prompt_sha256. `observe
--config CONFIG --asset-id UUID` makes one read-only observation. Image completion
requires a loaded result img and completed status twice; references/previews do
not establish completion. Preserve the pre-submission baseline and bind a new UUID.

### Opt-in recovery

```
huw supervise --config config.local.json --asset-id UUID --state-dir local/job --max-restarts 2 --backoff 5 --notify-terminal t17 --notify-app codex --relay-wait 900
```

The supervisor launches this same binary's watch command. It retries only a
failed local child, against the same asset and state directory, bounded to
max_restarts (0–10). Timeout/completed/failed/cancelled monitor events are not
resubmitted. Restart backoff is capped at 300 seconds. It holds supervisor.lock;
a crashed child's monitor.lock is removable only after the exact child handle
has been reaped and its PID AND supervisor token match. Legacy/foreign locks
fail closed. No browser restart, machine restart, or generation retry exists.
The supervisor itself is not restarted automatically: no OS service/task is installed.

`--relay-wait SECONDS` on watch/relay/supervise defers a send while the matching
agent is working using hc wait; default 0 preserves the Python immediate check.
Blocked/error/wrong app/exited states stop without force. A receipt is persisted
before send, and attempting/delivery_unknown remain non-retriable until manual
reconciliation. A successful hc exit reporting started=false is delivery_unknown.

## Regression matrix

| Python reference group | Rust coverage |
|---|---|
| FormTests (9) | rust_dom: verified, badge, credit labels, disabled, Korean, duplicates, inline/paragraph whitespace, privacy |
| HistoryTests (5) | rust_dom: completed, missing/duplicate, order, video required, enums; additional nested agreement/conflict |
| BrowserTests (7) | rust_dom origin/argv/ensure; rust_diagnostics transport redaction; rust_prompt clear/readback guards |
| WatchTests (16) | rust_state: confirmation/replay/disruption, timeout/resume, failure, binding, exclusive lock, cancellation, invalid/finite input, relay once/ambiguous/crash/target/app/legacy guards |
| CLITests (3) | rust_prompt BOM/whitespace render modes; rust_state notification-pair parse and offline CLI demo |
| PermissionTests (5 Unix-only) | rust_permissions: private artifacts, broad dir rejection, targeted modes, symlinks, temp cleanup |
| Local image path | rust_image: prepared-state equality, disabled/credit, model controls, nested status, loaded result, failed; common monitor/relay tests reused |
| New recovery | rust_recovery: real failed child bounded restart, token-bound locks, deferred hc wait, blocked stop |
| Windows storage | rust_windows: sharing violation recovery and permanent failure preservation |

The serialization-failure cleanup case uses a destination failure in Rust because
serde_json::Value cannot contain a non-serializable Python object. The old file
is never unlinked for replacement. Windows atomic replacement closes the temp
handle first and retries only OS errors 5/32/33 up to 350 ms; no ACL changes.
Evidence: https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist
and https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw.
One transient access failure occurred during the suite; bounded sharing tests
were added and the full suite then passed.

## Verified / not verified

- Windows Rust tests: 59 passed. Original Python: 45 run, 40 passed, 5 Unix-only skipped.
- Five Rust Unix permission scenarios are ported but **not executed**: the provided
  toolchain is Windows-only and no WSL cargo was present. No toolchain installation
  or system configuration change was made. Run on a real Unix filesystem before
  claiming Unix parity (not /mnt/c).
- Read-only real video and image inspect passed. Video collected duration/ratio/
  quality/bitrate and explicitly marked Audio missing; image read resolution,
  ratio, batch and reference identity. No live ensure/set-prompt/submit/relay was run.
- Supervision tested with an actual failed local watch child plus synthetic lock
  ownership tests, not by killing the real browser or a production watcher.
- No paid generation, credential handling, CA changes, install replacement,
  GitHub push, PR, issue update, or public posting in this migration.
- Independent offline Python/Rust event comparison: run
  `python tests/compare_demo.py --rust target/release/huw.exe` (omit .exe on Unix).
  Timestamps are excluded; all event/target/status/journal fields must agree.

Final local validation: release build succeeded; Python/Rust demo CLI output and target/status/event/transitions matched with only timestamps excluded. Windows suite 59 passed; fmt and diff whitespace checks completed. Unix tests remain unexecuted. No publication performed.
