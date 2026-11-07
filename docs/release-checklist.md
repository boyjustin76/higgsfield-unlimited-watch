# Experimental release handoff

Published at [boyjustin76/higgsfield-unlimited-watch](https://github.com/boyjustin76/higgsfield-unlimited-watch). No credentials are bundled. The initial publication instructions below are retained for reproducibility; do not rerun repository creation on this existing repository.

- [x] Competitor and official-entitlement research, with source links and caveats
- [x] Self-contained skill and standard-library CLI
- [x] Synthetic behavior tests and offline demo
- [x] Read-only live form + existing completed asset smoke test
- [x] Synthetic event delivered through real herdr-core relay
- [x] Public tree separated from local sessions, prompts and result IDs
- [x] GitHub authentication and repository creation/push
- [x] Observe hosted CI results after push (Windows/Ubuntu, Python 3.10/3.12)
- [ ] Personal-media live lifecycle and English demo
- [ ] Second-machine install verification

Owner workflow from a new PowerShell terminal:

```powershell
gh auth login --web
cd path\to\higgsfield-unlimited-watch
& .\scripts\publish.ps1
```

The publishing script verifies account identity, requires a clean committed tree, and refuses an existing origin rather than guessing. If your execution policy blocks scripts, use the equivalent reviewed `gh repo create ... --public --source . --remote origin --push` command directly; do not change system policy just for this project.

After publication, inspect `gh run list` and the actual workflow result before adding a release tag. The source version is `0.1.0a1`; no stable-release claim is intended. Do not upload local/ or config.local.json. A release demo should use personal assets with permission, never company material.
