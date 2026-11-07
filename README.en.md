# Bing Search MCP

An MCP server that lets AI agents search Microsoft Bing. **No API key, no account, no configuration.**

[日本語](README.md)

[![CI](https://github.com/mouseos/Bing_MCP/actions/workflows/ci.yml/badge.svg)](https://github.com/mouseos/Bing_MCP/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Verticals

| Tool | Search | Per page |
|---|---|---|
| `bing_web_search` | Web search | 10 |
| `bing_image_search` | Image search | 12 |
| `bing_video_search` | Video search | 35 |
| `bing_news_search` | News search | up to 50 (no paging) |
| `bing_suggest` | Suggestions (autocomplete) | — |
| `bing_build_search_url` | Build a search URL only (no request) | — |
| `bing_list_verticals` | List the above | — |

Every search accepts `mkt` (market, default `ja-JP`) and `setlang` (default `ja`).
Bing mis-detects the market and returns an empty result set without them, so the
defaults are always applied.

## Install

### 1. From a release

1. Download the ZIP for your OS from [Releases](https://github.com/mouseos/Bing_MCP/releases) and extract it
2. Run `bing-mcp-launcher` and click **Install** for your agent
3. Restart the agent

The launcher auto-detects Claude Desktop / Claude Code / Google Antigravity / Codex CLI.

### 2. From the command line

```bash
# print copy-paste config
bing-mcp setup

# Claude Code
claude mcp add Bing_MCP -- /path/to/bing-mcp
```

### 3. Manual config

```json
{
  "mcpServers": {
    "Bing_MCP": {
      "command": "/path/to/bing-mcp",
      "args": []
    }
  }
}
```

Leave `env` empty — there is nothing to configure.

### 4. Build from source

```bash
git clone https://github.com/mouseos/Bing_MCP.git
cd Bing_MCP
cargo build --release
# → target/release/bing-mcp
```

Rust 1.80+ is required (for `std::sync::LazyLock`). On Linux the launcher
additionally needs `libfontconfig1-dev libxkbcommon-dev libgtk-3-dev`; the
server itself has no system dependencies.

## Use as a Rust library

The search logic lives in the provider crate (package `bing-mcp-provider`,
imported as `bing_search`), so you can use it directly from your own Rust
program without the MCP server. No API key, login or configuration — it runs
anonymously, exactly like the MCP server.

```toml
[dependencies]
# From GitHub
bing_search = { git = "https://github.com/mouseos/Bing_MCP", package = "bing-mcp-provider" }
# From a local checkout
# bing_search = { path = "../Bing_MCP/crates/providers/bing", package = "bing-mcp-provider" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

```rust
use bing_search::BingSearchClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = BingSearchClient::new()?;
    let result = client.web_search("miso ramen recipe", 1, 5, "en-US", "en").await?;
    for item in &result.items {
        println!("{} — {}", item.title, item.url);
    }
    let images = client.image_search("Mount Fuji", 1, 12, "en-US", "en").await?;
    for item in &images.items {
        println!("{}", item.url);
    }
    let suggest = client.suggest("ramen", "web", "en-US", "en").await?;
    for s in &suggest.suggestions {
        println!("{}", s.keyword);
    }
    Ok(())
}
```

Public API (all importable from the crate root):

| Kind | Names |
|---|---|
| Client | `BingSearchClient` (`web_search` / `image_search` / `video_search` / `news_search` / `suggest`, returning `anyhow::Result`) |
| Shared result type | `SearchResult<T>`, `PROVIDER`, `verticals()` |
| Per-vertical item types | `WebItem`, `ImageItem`, `VideoItem`, `NewsItem`, `SuggestResult` / `Suggestion` |
| Offline helpers | `parse_web_results` (v7 search API JSON) / `parse_image_results` / `parse_video_results` / `parse_news_results` / `parse_suggest_json` (parse saved page HTML / API JSON), `build_search_url` / `build_page_url` / `suggest_url` / `web_api_url` (URL generation), `clean_text`, `decode_entities`, `urlencode` |

All result types implement serde `Serialize` / `Deserialize`, so they can be
saved to and loaded from JSON. Offline examples live in
`crates/providers/bing/examples/` (`parse_saved_html`, `build_search_url`),
and an external-use-shaped test lives in
`crates/providers/bing/tests/library_api.rs`.

When a search fails through the MCP server, the tool result is
`{provider, ok:false, error:{kind, message}}` — the same form as Baidu_MCP
and the Yahoo! JAPAN server. Access restrictions (403/429) come back as
`access_restricted`; the server never retries or tries to evade them.
Argument errors (such as a missing query) are MCP `isError` results, as usual.

## Examples

Prompts for an agent:

- "Look up Rust async runtimes" → `bing_web_search`
- "Find photos of Mount Fuji" → `bing_image_search`
- "Give me the 5 latest AI news stories" → `bing_news_search`
- "What's the original link of this video?" → `bing_video_search` (the real URL is recovered from the link's `churl` parameter)

Tool call:

```json
{
  "name": "bing_web_search",
  "arguments": { "query": "miso ramen recipe", "limit": 5 }
}
```

Response (excerpt):

```json
{
  "provider": "bing",
  "vertical": "web",
  "ok": true,
  "query": "miso ramen recipe",
  "page": 1,
  "count": 5,
  "total_hits": 70500,
  "next_page": 2,
  "related_queries": ["miso ramen recipe easy", "miso ramen homemade"],
  "items": [
    {
      "rank": 1,
      "title": "Miso ramen recipe",
      "url": "https://example.com/recipe",
      "display_url": "https://example.com › recipe",
      "snippet": "Steps · 1. Cut the cabbage into bite-size pieces. …"
    }
  ]
}
```

Web result links are the direct destination URLs returned by the native
API — no click-tracking redirect is involved.

## Design

```
crates/
├── providers/bing/           # search logic
│   ├── common.rs             #   HTTP, HTML cleanup, redirect unwrapping
│   ├── urlbuild.rs           #   search URL generation
│   ├── web.rs image.rs video.rs news.rs suggest.rs
│   └── lib.rs                #   HTTP client and shared response types
├── server/                   # the MCP server (stdio JSON-RPC 2.0)
│   ├── main.rs               #   protocol handling
│   └── tools.rs              #   tool definitions and dispatch
└── launcher/                 # registration GUI (egui)
```

The implementation follows what an APK analysis of the official Bing Android
app established: the app displays results in a WebView shell over
`https://www.bing.com/search?q=%s&form=%s&sdkhh=1`, and for images, video
and news this server likewise reads the server-rendered result pages Bing
already ships to browsers, using CSS selectors. No certificate pinning or
login stands in the search path. Web search is the one exception: it calls
the app's embedded native search API instead of a page (next section).

### Why web search is fetched differently

Web search uses Bing's native search API
(`https://www.bingapis.com/api/v7/search`). The API is embedded in the
Bing app, where its in-app role is fetching related queries — but its
response carries the full web result set (title, URL, snippet, total hit
count) plus the related queries, so this server consumes the response
directly and now returns `related_queries` as well. URLs come back as
direct links, so no redirect unwrapping is needed. Image, video and news
results are not part of this API's response, so those three keep reading
result pages as before. Suggestions likewise use the app's own API (see
below).

### Why image search is fetched differently

The desktop image page loads its results with JavaScript; the initial HTML
contains no results. The form the app's WebView receives — a mobile user
agent plus the app-style parameters `form=L2GV01&sdkhh=1` and the pinned
`mkt`/`setlang` — returns fully server-rendered results, so
`bing_image_search` alone uses that form. That page shows no total count,
so image results carry no `total`.

### About the "appid" values

The suggestions backend and the web search backend (v7 search API)
require an `appid`. These are not user secrets: they are public values
embedded in the Bing app itself — the suggestions one is what the app's
search box uses, and the v7 search one is what the app uses to fetch
related queries. There is nothing for the user to provide.

## Development

```bash
cargo test --workspace                  # all tests, including live e2e against Bing
cargo test --workspace -- --skip e2e_   # offline tests only
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

The `e2e_*` tests connect to the real Bing and verify that each vertical
still parses. The parsers are also verified offline against real result
pages captured during the analysis (fixtures). Note that running the
`e2e_*` tests in parallel (the default) can trip Bing's rate limiting
(HTTP 429); rerun with `-- --test-threads=1` in that case.

Manual check over stdio:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"x","version":"1"}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"bing_web_search","arguments":{"query":"ramen","limit":2}}}' \
  | ./target/debug/bing-mcp
```

## Notes

- This reads publicly served search result pages. Please use it at a reasonable rate.
- If Bing changes its page structure, the affected vertical may fail temporarily; the error message says so.
- This project is not affiliated with Microsoft. Bing is a trademark of the Microsoft group of companies.

## License

MIT — see [LICENSE](LICENSE).
