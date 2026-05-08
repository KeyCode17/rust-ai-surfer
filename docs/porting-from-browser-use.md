# Porting from browser-use

This is a phased port of [browser-use](https://github.com/browser-use/browser-use) to Rust. Each phase ships a working slice. After every phase the workspace builds, the test suite passes, and the version bumps by one minor.

## Phase map

| Phase | Subject | Version |
|-------|---------|---------|
| 0 | Workspace scaffold (this) | 0.1.0 |
| 1 | Domain layer - entities, ports, value objects | 0.2.0 |
| 2 | CDP + cosmium launcher + ChatAnthropicClaudeCode | 0.3.0 |
| 3 | Agent step loop and use cases | 0.4.0 |
| 4 | Tools registry and built-in actions | 0.5.0 |
| 5 | DOM extraction, clickable detection, hashing | 0.6.0 |
| 6 | Watchdogs and event bus | 0.7.0 |
| 7 | All LLM providers, serializers, token cost | 0.8.0 |
| 8 | FileSystem and file types | 0.9.0 |
| 9 | CLI, daemon, cloud | 0.10.0 |
| 10 | MCP, skills, sandbox, recording, judge | 0.11.0 |
| 11 | Examples, integration tests, OAuth + cosmium PoC | 0.12.0 |
| Release | 1.0.0 | 1.0.0 |

## Naming map

| browser-use (Python) | rust-ai-surfer (Rust) |
|----------------------|------------------------|
| `browser_use.agent.service:Agent` | `ras-agent::application::run_agent::RunAgent` |
| `browser_use.browser.session:BrowserSession` | `ras-browser::domain::session::BrowserSession` |
| `browser_use.tools.service:Tools` | `ras-tools::domain::registry::ActionRegistry` |
| `browser_use.dom.service:DomService` | `ras-dom::application::extract_tree::ExtractTree` |
| `browser_use.llm.anthropic.claude_code:ChatAnthropicClaudeCode` | `ras-llm-anthropic::infrastructure::http::claude_code::ChatAnthropicClaudeCode` |
| `BrowserProfile` | `ras-browser::domain::browser_profile::BrowserProfile` |
| `EnhancedDOMTreeNode` | `ras-dom::domain::tree::EnhancedDomTreeNode` |
| `ActionResult` | `ras-types::domain::action_result::ActionResult` |
| `AgentHistoryList` | `ras-agent::domain::agent_history::AgentHistoryList` |

## Verbatim ports

A handful of files port byte-for-byte (modulo idiom):

- `ChatAnthropicClaudeCode` - 4-tier auth + billing header + `cc_version` resolution.
- `compute_action_hash` - same normalization rules to produce stable hashes.
- `filter_dynamic_classes` - same dynamic-class filter for DOM hash determinism.
- `_normalize_csv` - RFC 4180 normalization with the same edge-case fixes.
- URL allowlist matching - same www auto-add, root-domain heuristic, IPv4/IPv6 block, credentials guard.
