# Launch preparation

Rust is primary and Python remains available. This is an experimental development update, not a stable release or a marketing launch.

- [x] README first screen: purpose, account-free demo, Rust/Python placement, limits.
- [x] Explicitly distinguish terminal-browser from optional Herdr notifications.
- [x] Add Windows/Ubuntu Rust tests and Python/Rust comparison to CI.
- [x] Preserve original 45 Python scenarios and add issue regressions.
- [x] Separate current code tests from historical live observations.
- [ ] Record and review the 30-second demo GIF below.
- [ ] Validate this revision's live lifecycle with separately authorized, owned media.
- [ ] Check all hosted CI results immediately before release.
- [ ] Publish a release note that retains the limitation: **this revision's full live lifecycle and suspend/recovery remain unvalidated**.
- [ ] Owner decides launch date and authorizes each external post.

## 30-second GIF storyboard (pending, issue #6)

Use only owned sample media and a sanitized terminal. No private paths, actual result IDs, account details, or production captures. Do not label synthetic frames as live.

1. 0–5s: offline `huw demo`; visible subtitle “Synthetic / no account needed”.
2. 5–12s: separately authorized read-only `inspect` capture; show toggle/submit verification and incomplete settings if any.
3. 12–22s: a separately authorized known result's `watch` completion/event output; substitute only clearly labelled example identifiers in the published edit.
4. 22–30s: “No LLM polling while waiting. No cookies exported. Browser must stay open.” Include the unvalidated-scope caption.

Save the approved recording as docs/demo.gif and embed it under README badges. A script/storyboard is not a completed recording. No generation or capture is required to run the offline demo.

## Optional launch copy (draft only)

Show HN title: “Show HN: Let your agent stop polling Higgsfield while a video renders”.
Body should include the waiting problem, offline demo/build/inspect commands, and the live-validation/browser-open limits. Adapt for r/ClaudeCode or r/AI_Agents (agent skill) and X only with separate posting authorization. Coordinating posts is an owner choice; no trending or star-growth result is promised.

The initial scripts/publish.ps1 is for creating a repository from scratch; do not rerun it against this existing project. Never publish local configs, prompts, browser profiles or state directories.
