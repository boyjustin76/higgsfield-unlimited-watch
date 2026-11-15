# Rust implementation and compatibility

Rust in `src/` is the primary implementation. Build with `cargo build --locked --release` or install with `cargo install --path . --locked`. Python remains at `skills/higgsfield-unlimited-watch/scripts/huw.py` for existing callers and as an independent video reference. No shared installation is automatically changed.

## Compatibility contract

Both retain inspect, ensure-unlimited, set-prompt, watch, relay, demo and --version; command-array/browser/tab/timeout config; target.json, status.json, transitions.jsonl, event.json and relay receipts; SHA256 event IDs, explicit resume, two confirmations, private state, exclusive locks and exact recipient/app binding.

The original 45 Python scenarios remain in test_huw.py/test_permissions.py. Additional issue regressions exercise incomplete/empty/duplicate settings, signed-out/loading/missing controls, nested cards, new-tab OFF and uncertain relay acceptance. Rust ports the original behavioral groups:

| Python group | Rust tests |
|---|---|
| Form (9), history (5) | rust_dom, rust_issue_regressions |
| Browser (7) | rust_dom, rust_diagnostics, rust_prompt |
| Watch/relay (16) | rust_state |
| CLI (3) | rust_state, rust_prompt |
| POSIX permissions (5) | rust_permissions (Unix filesystem only) |
| New diagnostics/settings | rust_diagnostics, rust_issue_regressions |
| Rust extensions | rust_image, rust_recovery, rust_windows |

`tests/compare_demo.py` runs both real CLIs against 11 offline form/diagnostic fixtures and compares demo output, target, status, event and transition journal with only timestamps excluded. This is scoped equivalence, not a proof that all possible inputs or platforms behave identically.

## Rust-only extensions

- `media_kind: "image"` config (default video); result images must be loaded with positive dimensions.
- `observe --config CONFIG --asset-id UUID`: single read-only observation.
- `ensure --config CONFIG --expected FILE`: read-only prepared-image snapshot comparison (model/resolution/ratio/batch/reference_ids/prompt_sha256). Not generation authorization.
- `supervise`: bounded restarts of its own failed watch child, same asset/directory.
- `--relay-wait SECONDS`: bounded defer while exact recipient is working. Blocked/exited/wrong app stops.

These extensions are not exposed by Python. Python users can migrate explicitly or retain manual watch/relay recovery.

## Verification boundaries

See [validation](docs/validation.md) and actual [CI results](https://github.com/boyjustin76/higgsfield-unlimited-watch/actions/workflows/test.yml). Windows local tests and offline comparison are recorded there; Ubuntu CI exercises real Unix permission semantics. Windows tests cannot establish POSIX permissions.

No new live generation, live notification, browser-kill recovery, suspend/resume, or billing verification is performed by this update. No browser or machine restart is implemented. The 30-second live demo remains pending.

Windows atomic replacement closes the temporary handle and bounds retries to OS errors 5/32/33 (350ms); old destinations are preserved on failure. Sources: [tempfile persist](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist), [MoveFileEx](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw).
