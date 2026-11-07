# Contributing

Start with a reproducible observation: tool version, OS, UI language, command and expected/actual state. Include sanitized synthetic HTML when selectors change. Never post cookies, sessions, private media URLs, company images or real prompts.

Run `python -m unittest discover -s tests -v`. Runtime helpers use the Python standard library. Keep the skill directory self-contained so copying it installs the complete behavior.

Useful contributions: field-verified pending/failure status variants, second-machine setup reports, reconnect tests and adapters for other hosts. Mark synthetic coverage separately from real browser observations. Avoid arbitrary token-savings percentages or claiming support based only on a unit test.

Do not add automatic credit fallback, implicit retry after uncertain submission, multi-account scheduling or token extraction to this project. Discuss new scope in an issue first. A future MCP adapter should reuse the existing local interfaces rather than change the entitlement assumptions.
