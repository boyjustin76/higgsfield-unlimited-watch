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

Stage 1 scaffold only. No feature parity claimed. Next: implement DOM/transport
and inspect/ensure. Record validation and unresolved gaps at each checkpoint.
