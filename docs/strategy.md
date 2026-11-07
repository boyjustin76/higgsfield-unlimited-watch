# Product decision: a narrow web workflow

## User problem

An account has a useful web Unlimited entitlement, but the chosen generation integration cannot select it. The user wants an agent to operate the website and resume after a long generation without personally watching the page or asking an LLM for every status check.

The seven-day window motivated the original workflow. The product should not depend on every user receiving that exact offer. Current entitlement evidence and existing alternatives are documented in [competitors.md](competitors.md).

## Positioning

**Higgsfield web Unlimited, assisted by your agent; waiting handled locally.**

The distinguishing bundle is visible toggle/submit verification, no exported browser credentials/private generation endpoint in our code, durable exact-asset monitoring, and conservative event recovery. None of those components alone is novel. Competitors already have waiting, queues and MCP support.

We chose `higgsfield-unlimited-watch` because `higgsfield-unlimited-skill` already exists. The skill is the entry point; a small local CLI implements repeatable operations. Do not build a generic automation platform or another full model catalog before validating this workflow.

## v0.1 boundary

Ship: one self-contained skill, compact form checks, prompt replacement, exact-ID local monitoring, persisted events, optional herdr-core notification, offline demo, tests and documented recovery.

Agent responsibilities: source/reference identity, complete parameter comparison, actual authorized Generate click, one active generation across sessions, binding the new result and creative review. The CLI does not implement an account-wide scheduler or atomic submit transaction.

Deferred: personal-image English video demo (requested for later), new-media live lifecycle validation, second-machine setup, other browser/host adapters, MCP wrapper, unattended batches and image/audio completion support. Do not imply these work in v0.1.

## Validation and adoption plan

1. Verify read-only behavior against the real web form and an existing result; publish only sanitized outcomes.
2. Test missing/duplicate assets, OFF/disabled switch, credit label, failure/timeout, interrupted observation and crash-ambiguous relay with synthetic fixtures.
3. Before stable release, record a personal-image live lifecycle and second-machine installation. Separate observed setup duration from download/first-time WSL setup; do not impose an invented ten-minute guarantee.
4. Measure status-check model calls separately from setup, browser interaction and completion review. The implementation makes no model calls in the watcher; savings compared with another workflow are not yet benchmarked.
5. Ask early adopters to report setup success, the actual entitlement path and recovery failures. Use those reports to decide whether broader adapters are worth maintaining.

## README and distribution

Lead with the user's problem, then an executable offline demo, short setup, and explicit tested scope. Use English for the public package. No company images, prompts, asset IDs, account databases, absolute personal paths or browser session files. The future live demo uses personal media only.

Maintain license, focused contribution instructions, test CI and a fair alternatives table. Publish an experimental release rather than suggesting mature unattended reliability. Do not invent star forecasts, adoption counts or token-reduction percentages.
