# Validation record

Date: 2026-11-07. Experimental release; this is a capability matrix, not a claim that the complete user journey is production-ready.

| Capability | Evidence | Limits |
|---|---|---|
| Unit/behavior tests | 34 tests pass on local Windows Python 3.12 | CI matrix supplied; remote CI result not yet observed |
| Offline demo | CLI runs synthetic processing → completed → completed through the actual parser/persistence | No real generation or browser involved |
| Live form inspection | New helper verified actual Unlimited switch=true, enabled Unlimited submit label, prompt hash, one reference | Read-only; no credit/billing audit |
| Stable DOM identity | Actual data-asset-id/data-job-status observed in signed-in browser | App attributes may change; virtualized nodes may disappear |
| Live completed asset watch | New helper confirmed a real completed asset in two polls (5-second interval) and persisted its event | Existing completed asset only; no new generation |
| Prompt replacement and toggle change | Original internal workflow exercised clear/fill and toggle; new helper has mocks | New helper mutation path has not yet been field-tested |
| Pending/failure/timeout recovery | Synthetic tests | Pending/failure enum coverage not established live for this version |
| Herdr notification | New relay delivered a synthetic event to an active Copilot terminal; receiver acknowledged it | Synthetic event, live transport; no closed-app wake guarantee |
| Second machine | Not tested | Office setup remains pending |
| Personal-media English demo | Deferred at user's request | No company footage is shipped |
| Skill packaging | Bundled skill validator passed in Python UTF-8 mode | Behavioral installation on a second host remains untested |
| Independent review | Copilot reviewed competitor README/source and product positioning | Source review of this implementation was not completed because its workspace required additional read approval |

Tested browser stack: Windows, WSL2 Ubuntu, terminal-browser 0.15.3, Noctty 1.3.135. Agent coordination in the current desktop app uses herdr-core `hc`, not the older `herdr` multiplexer. Native Linux transport is configurable but not field-validated; macOS/browser adapters are not advertised as supported.

## Guarantees are deliberately bounded

- A precheck reads a moment in time. It does not atomically gate an independently performed Generate click or guarantee billing.
- Completion requires a known asset ID, recognized terminal DOM state, video element and repeated observation. It does not verify playback or creative quality.
- File locks/events prevent duplicate local monitoring/send attempts only within the same state directory and tested local failure model. Different directories or machines are not coordinated.
- Relay acceptance is not exactly-once receipt or execution. Crashes can lose a notification; ambiguous sends are not automatically retried.
- The watch loop contains zero LLM calls by construction. No empirical token-savings percentage is claimed.

Company data and private session evidence stay outside the public repository. Synthetic fixtures deliberately use reserved example UUIDs.
