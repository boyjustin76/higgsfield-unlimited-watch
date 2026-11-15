# Higgsfield Unlimited Watch

**Your AI agent submits the Higgsfield job. A local watcher waits — no LLM polling while it renders.**

[![Tests](https://github.com/boyjustin76/higgsfield-unlimited-watch/actions/workflows/test.yml/badge.svg)](https://github.com/boyjustin76/higgsfield-unlimited-watch/actions/workflows/test.yml)
![Status](https://img.shields.io/badge/status-experimental-orange)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

- Verifies the actual Unlimited toggle + submit label (a badge alone does not count).
- Follows one result by `data-asset-id`, writes a completion event, optionally notifies your agent.
- No exported cookies/tokens, no LLM calls while waiting. **Rust is primary; dependency-free Python is retained.**

> Try without an account: `python skills/higgsfield-unlimited-watch/scripts/huw.py demo`
> Experimental Rust 0.2.0-dev / Python compatibility helper. Full live lifecycle of this revision is **not validated**; see [validation](docs/validation.md).

`demo` needs no browser or Herdr. `inspect` needs terminal-browser, but no Herdr. The field-tested browser stack is Windows + WSL; portable core tests do not prove other live environments. The real 30-second GIF remains pending ([#6](https://github.com/boyjustin76/higgsfield-unlimited-watch/issues/6)).

## Start offline

```bash
git clone https://github.com/boyjustin76/higgsfield-unlimited-watch.git
cd higgsfield-unlimited-watch
cargo build --locked --release
cargo install --path . --locked
huw demo
```

Installing adds `huw` to Cargo's bin directory; ensure it is on PATH. Without installing, use `target/release/huw` (`.\target\release\huw.exe` on Windows). Python 3.10+ stays at `skills/higgsfield-unlimited-watch/scripts/huw.py`, requires no pip dependencies, and is not automatically replaced.

The demo is **synthetic**: virtual clock, real classifier/event persistence, no browser or generation.

```bash
python -m unittest discover -s tests -v
cargo test --locked
python tests/compare_demo.py --rust target/release/huw
```

Use `target/release/huw.exe` for comparison on Windows. [Compatibility matrix](RUST_PORT.md).

## Use a signed-in browser

1. Follow [setup](skills/higgsfield-unlimited-watch/references/setup.md). Copy `config.example.json` to `config.local.json`; select the actual executable/browser/tab from `terminal-browser ls --all`.
2. Run `huw inspect --config config.local.json`. `unlimited_verified` covers only switch/submit controls. Inspect `settings_complete` / `missing_settings` and separately reconcile every requested setting and reference.
3. Follow [the skill](skills/higgsfield-unlimited-watch/SKILL.md) for one authorized job. **Every new tab, navigation and setting change requires fresh verification.** Never inherit another tab's ON state.
4. After the separately authorized agent submits, bind the new result's exact UUID:

```bash
huw watch --config config.local.json --asset-id RESULT_UUID --state-dir local/job-001
```

No helper command submits generation. Unknown/timeout is not remote failure and never authorizes resubmission. Keep the browser open on the relevant history. A precheck guarantees neither billing nor creative quality.

For an active Herdr-hosted Codex/Copilot agent, add `--hc HC_EXECUTABLE --notify-terminal AGENT_TERMINAL_ID --notify-app codex`. Address and app must match. Without Herdr, consume `event.json` through your host's supported callback; the file alone does not wake an agent.

## Commands

| Command | Rust | Retained Python |
|---|---|---|
| `inspect`, `ensure-unlimited`, `set-prompt` | Video and image | Video |
| `watch`, `relay`, `demo`, `--version` | Supported | Supported |
| `observe` | One read-only result observation | Use watcher |
| `ensure --expected FILE` | Read-only prepared-image comparison | Not available |
| `supervise` | Bounded restart of own watcher child | Manual restart |
| `--relay-wait SECONDS` | Bounded wait for working recipient | Explicit relay when ready |

Ordinary `watch` is **not** a process watchdog. Rust `supervise` resumes the same asset/directory, never generation or the browser. It preserves uncertain send receipts and records exhausted restarts in `supervisor.json`. Read [recovery](skills/higgsfield-unlimited-watch/references/recovery.md).

## Install the agent skill

Copy the entire `skills/higgsfield-unlimited-watch/` folder to your host's skill directory (for example `~/.codex/skills/higgsfield-unlimited-watch/`), or point the agent at this checkout. A skill-only copy includes Python; build/install Rust separately. No MCP server is required.

## Unlimited availability

Checked 2026-11-15: [official help](https://higgsfield.ai/creator-hub/help-center/credits/what-are-unlimited-models-and-which-plans-include-them) describes model-specific, time-limited Unlimited access on Higgsfield's website. Consult [pricing](https://higgsfield.ai/pricing) and your signed-in account for models, duration and settings. This project grants no entitlement and promises no account eligibility or zero-cost generation. Local browser automation is distinct from paid API/MCP generation.

## Limits and credits

Navigation/virtualization can hide history. Nested wrappers require one same-ID root and agreeing statuses; conflicting or separate cards remain unknown. Video completion requires a descendant video and two consecutive observations; image mode additionally requires a loaded result image. Neither verifies creative quality.

[Validation](docs/validation.md) · [Launch checklist](docs/release-checklist.md) · [Contributing](CONTRIBUTING.md)

Built on [terminal-browser](https://github.com/zenbu-labs/terminal-browser); optional Herdr notifications use `hc`. Independent community project, not affiliated with Higgsfield. MIT licensed.
