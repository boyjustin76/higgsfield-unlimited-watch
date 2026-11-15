# Validation and limits

## Current update (2026-11-15)

This update performs **offline tests only**, not fresh live generation or account verification.

| Area | Evidence | Limit |
|---|---|---|
| Original Python baseline | Original 45 scenarios retained; Windows skips 5 POSIX tests | Windows cannot verify Unix permission semantics |
| Python issue regressions | Added settings/diagnostic/nested-card/new-tab/relay tests | Sanitized synthetic DOM |
| Rust | Unit/integration suites, image and recovery tests | Local child failure is not live browser recovery |
| Cross-language comparison | tests/compare_demo.py: 11 inspect/diagnostic CLI fixtures plus all demo event/state/journal fields; timestamps excluded | Not exhaustive equivalence; Rust extensions documented separately |
| Hosted CI | [Windows/Ubuntu Python and Rust jobs](https://github.com/boyjustin76/higgsfield-unlimited-watch/actions/workflows/test.yml) | Refer to the run for the exact commit; a workflow definition alone is not a pass |
| Current Unlimited documentation | Official help/pricing consulted on 2026-11-15 | No account's eligibility, model list, expiry or billing verified |
| 30-second demo GIF | Storyboard in release-checklist.md | Recording pending (#6) |

Run locally:

```text
python -m unittest discover -s tests -v
cargo fmt --check
cargo test --locked
cargo build --locked --release
python tests/compare_demo.py --rust target/release/huw
```

On Windows use huw.exe for the comparison and Python -X utf8 for non-ASCII terminal output. CI runs Unix tests on its native filesystem, not a WSL Windows mount.

## Historical field reports, not revalidated by this update

Issues [#1](https://github.com/boyjustin76/higgsfield-unlimited-watch/issues/1), [#2](https://github.com/boyjustin76/higgsfield-unlimited-watch/issues/2), [#3](https://github.com/boyjustin76/higgsfield-unlimited-watch/issues/3), and [#4](https://github.com/boyjustin76/higgsfield-unlimited-watch/issues/4) record earlier second-PC setup/read-only checks and one separately authorized generation/completion/relay observation. Those reports motivated these fixes. They do not establish a fresh full lifecycle for this Rust revision.

The previously tested browser environment was Windows + WSL Ubuntu + terminal-browser. Native Linux/macOS browser integration, general-agent notifications, minimized/suspend behavior, live crash recovery, billing and output quality remain unverified here.

## Bounded guarantees

- A UI precheck is a point-in-time observation, not an atomic submission guard or billing guarantee.
- Settings completeness means required nonempty labels were observed, not that values match the user's request. Unknown/renamed controls remain missing.
- Diagnostic codes use explicit readiness/login evidence. Missing controls do not prove entitlement loss or a site redesign.
- Video completion requires exact asset identity, agreeing DOM status, a descendant video and consecutive confirmation. Image completion also requires loaded dimensions. Neither verifies playback/creative quality.
- Locks/receipts coordinate one local state directory. They do not enforce account-wide concurrency or exactly-once remote processing.
- Ordinary watch does not restart itself. Opt-in Rust supervise has bounded local-child recovery and does not restart the browser, machine, or generation.
- Accepted relay means transport acceptance. Attempting/delivery_unknown receipts block automatic retries.
- The watch loop has zero LLM calls by construction. No empirical savings percentage is claimed.

Do not attach private sessions, configs, raw DOM, credentials, real asset identifiers or production media to public issues.
