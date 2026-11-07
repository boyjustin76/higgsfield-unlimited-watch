# Alternatives and competitive research

Checked **2026-11-07**. Bounded public GitHub/web searches plus README and selected-source review by two agents. No competitor was installed, authenticated or executed. Stars are a snapshot, not proof of quality or demand; machine-readable metadata is in [research.json](research.json).

## Direct candidates

| Project | Stars at check | Documented approach | Implication for this project |
|---|---:|---|---|
| [Ali-Homsi/higgsfield-unlimited-skill](https://github.com/Ali-Homsi/higgsfield-unlimited-skill) | 0 | Skill with browser-context backend generation engines, batching and polling | Closest name and use case. We chose a different name; focus on rendered UI checks and local persistent monitoring. |
| [nukIeer/higgsfield-unlimited-mcp](https://github.com/nukIeer/higgsfield-unlimited-mcp) | 17 | Unofficial MCP using browser-session credentials and `use_unlim`; status/queue tools | Already makes Unlimited available through an MCP interface. Never claim MCP cannot technically support Unlimited. |
| [WinstonJunCong/cgl-higgsfield](https://github.com/WinstonJunCong/cgl-higgsfield) | 2 | Session-authenticated MCP generation, async queue and job tools | Broad functionality exceeds this project's scope; compete on workflow/setup preferences, not model count. |
| [build-with-bala/higgs-unlim](https://github.com/build-with-bala/higgs-unlim) | 0 | CLI using web backend generation and polling; documents billing behavior | Local waiting already exists. Our distinct choice is DOM observation and explicit web controls. |
| [FarisHijazi/higgsfield-web-api](https://github.com/FarisHijazi/higgsfield-web-api) | 10 | Headless CLI/HTTP service over the web app, job create/get/wait/list | Another direct alternative; API generation may be preferable for users willing to maintain that integration. |

These tools describe different authentication and transport choices. This table does not establish security superiority, current reliability, or compliance of any project. We did not copy their code.

Selected code reviewed by the collaborating agent: [Ali-Homsi engines.js](https://github.com/Ali-Homsi/higgsfield-unlimited-skill/blob/main/scripts/engines.js), [nukIeer jobs.py](https://github.com/nukIeer/higgsfield-unlimited-mcp/blob/main/higgsfield_unlimited_mcp/jobs.py), and [cgl registry.py](https://github.com/WinstonJunCong/cgl-higgsfield/blob/main/src/higgsfield_unlimited_mcp/registry.py). That review found page/in-process tracking in those modules. It does **not** prove the entire projects lack other persistence or recovery mechanisms. Visible toggle verification and an equivalent durable relay were not established in the reviewed surfaces.

## Official and adjacent alternatives

| Project | Stars at check | Why it matters |
|---|---:|---|
| [higgsfield-ai/cli](https://github.com/higgsfield-ai/cli) | 698 | Official generation CLI already offers `--wait`, `generate get` and `generate wait`. Prefer it when its entitlement/cost path fits. |
| [higgsfield-ai/skills](https://github.com/higgsfield-ai/skills) | 1,909 | Official agent skills; an established distribution and workflow alternative. |
| [pixelab-ch/higgsfield-skills](https://github.com/pixelab-ch/higgsfield-skills) | 12 | Community MCP skills with model discovery, credits and job status guidance. |
| [robonuggets/higgsfield-skill](https://github.com/robonuggets/higgsfield-skill) | 71 | Lightweight MCP skill, subscription context and cost confirmation. |
| [jurislm/higgsfield-plugin](https://github.com/jurislm/higgsfield-plugin) | 1 | Plugin packaging for hosted MCP and CLI skills. |
| [zenbu-labs/terminal-browser](https://github.com/zenbu-labs/terminal-browser) | 4,476 | Our browser dependency, not an end-to-end Higgsfield competitor. |
| [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser) | 46,359 | General browser automation CLI; users can build their own local polling. |
| [microsoft/playwright-mcp](https://github.com/microsoft/playwright-mcp) | 39,289 | General browser control over MCP; another possible foundation. |

## Official entitlement evidence is conditional

The current [MCP help page](https://higgsfield.ai/creator-hub/help-center/integrations/what-is-higgsfield-mcp) and [pricing page](https://higgsfield.ai/pricing) describe web Unlimited/free generations separately from credit-using MCP/CLI routes. This supports the target problem for those offers.

However, the [Unlimited MCP promotion article](https://higgsfield.ai/blog/unlimited-mcp) describes a separate 24-hour MCP trial claimable through July 31, 2026. It is evidence that exceptions have existed, not that the offer remains available. The [talking-avatar guide](https://higgsfield.ai/blog/talking-ai-avatar-inside-claude) explicitly distinguishes past limited MCP promotions from ordinary credit billing.

Therefore: do not say “MCP never supports Unlimited,” “every new subscriber always gets seven days,” or “this is the only solution.” Check the actual account, model and current offer. Our README targets **web-entitled accounts whose selected integration does not cover that entitlement**.

## Search scope and limits

Queries included `Higgsfield Unlimited skill in:readme`, `higgsfield MCP`, `higgsfield CLI`, `org:higgsfield-ai MCP`, `higgsfield unlimited automation`, and browser/job-monitor/notification variants. Results were filtered for relevant source repositories. No results for a query means only that this search did not find them. Adjacent generic job monitors were not comprehensively audited.

Demand is unproven. Direct candidate star counts are modest at this snapshot; a catchy description alone is not evidence that this repository will attract significant adoption.
