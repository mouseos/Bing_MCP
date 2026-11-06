# Bing Search MCP

Microsoft Bing の検索をAIエージェントから使えるようにするMCPサーバー。**APIキー不要・アカウント不要・設定不要**。

[English](README.en.md)

[![CI](https://github.com/mouseos/Bing_MCP/actions/workflows/ci.yml/badge.svg)](https://github.com/mouseos/Bing_MCP/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 対応する検索

| ツール | 検索 | 1ページあたり |
|---|---|---|
| `bing_web_search` | ウェブ検索 | 10 |
| `bing_image_search` | 画像検索 | 12 |
| `bing_video_search` | 動画検索 | 35 |
| `bing_news_search` | ニュース検索 | 最大50（ページングなし）|
| `bing_suggest` | サジェスト（キーワード補完） | — |
| `bing_build_search_url` | 検索URLの生成のみ（通信なし） | — |
| `bing_list_verticals` | 上記の一覧を返す | — |

どの検索も `mkt`（市場、既定 `ja-JP`）と `setlang`（既定 `ja`）を指定できます。
Bingはこの2つを付けないと市場判定を誤って「結果なし」を返すため、既定値が必ず付きます。

## インストール

### 1. リリースから

1. [Releases](https://github.com/mouseos/Bing_MCP/releases) から自分のOS向けZIPをダウンロードして展開
2. `bing-mcp-launcher` を起動し、使いたいエージェントの **インストール** をクリック
3. エージェントを再起動

ランチャーは Claude Desktop / Claude Code / Google Antigravity / Codex CLI を自動検出して登録します。

### 2. コマンドラインから

```bash
# 設定用のJSONとコマンドを表示
bing-mcp setup

# Claude Code
claude mcp add Bing_MCP -- /path/to/bing-mcp
```

### 3. 手動設定

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

`env` は空のままで構いません。設定する項目はありません。

### 4. ソースからビルド

```bash
git clone https://github.com/mouseos/Bing_MCP.git
cd Bing_MCP
cargo build --release
# → target/release/bing-mcp
```

Rust 1.80以上が必要です（`std::sync::LazyLock` のため）。Linuxではランチャーのビルドに
`libfontconfig1-dev libxkbcommon-dev libgtk-3-dev` が追加で必要です。サーバー本体に
システム依存はありません。

## Rustライブラリとして使う

検索ロジックは provider crate（package名 `bing-mcp-provider`、import名は
lib名の `bing_search`）にあり、MCPサーバーを介さず自分のRustプログラムから
そのまま使えます。APIキー・ログイン・設定は不要で、MCP版と同じく匿名で動きます。

```toml
[dependencies]
# GitHubから
bing_search = { git = "https://github.com/mouseos/Bing_MCP", package = "bing-mcp-provider" }
# ローカルチェックアウトから
# bing_search = { path = "../Bing_MCP/crates/providers/bing", package = "bing-mcp-provider" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

```rust
use bing_search::BingSearchClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = BingSearchClient::new()?;
    let result = client.web_search("味噌ラーメン 作り方", 1, 5, "ja-JP", "ja").await?;
    for item in &result.items {
        println!("{} — {}", item.title, item.url);
    }
    let images = client.image_search("富士山", 1, 12, "ja-JP", "ja").await?;
    for item in &images.items {
        println!("{}", item.url);
    }
    let suggest = client.suggest("ラーメン", "web", "ja-JP", "ja").await?;
    for s in &suggest.suggestions {
        println!("{}", s.keyword);
    }
    Ok(())
}
```

公開API（すべて crate ルートから import できます）:

| 種類 | 名前 |
|---|---|
| クライアント | `BingSearchClient`（`web_search` / `image_search` / `video_search` / `news_search` / `suggest`、戻り値は `anyhow::Result`） |
| 共通の結果型 | `SearchResult<T>`、`PROVIDER`、`verticals()` |
| 各検索の結果型 | `WebItem`、`ImageItem`、`VideoItem`、`NewsItem`、`SuggestResult` / `Suggestion` |
| オフライン用ヘルパー | `parse_web_results` / `parse_image_results` / `parse_video_results` / `parse_news_results` / `parse_suggest_json`（保存したページHTML・JSONの解析）、`build_search_url` / `build_page_url` / `suggest_url`（URL生成）、`clean_text`、`decode_entities`、`urlencode` |

結果型は serde の `Serialize` / `Deserialize` に対応し、JSONへの保存と読み戻しが
できます。ネットワーク不要の利用例が
`crates/providers/bing/examples/`（`parse_saved_html`、`build_search_url`）に、
外部利用の形そのままのテストが `crates/providers/bing/tests/library_api.rs` にあります。

なお、MCPサーバー経由で検索が失敗した場合は、ツール結果として
`{provider, ok:false, error:{kind, message}}` の形で返ります（Baidu_MCP / Yahoo と共通の形式）。
403・429などのアクセス制限は `access_restricted` として返し、リトライや回避はしません。
引数の誤り（クエリ未指定など）は従来どおり MCP の `isError` で返ります。

## 使用例

エージェントへの指示例:

- 「Rustの非同期ランタイムについて調べて」 → `bing_web_search`
- 「富士山の写真を探して」 → `bing_image_search`
- 「最新のAIニュースを5件」 → `bing_news_search`
- 「この動画の元リンクは？」 → `bing_video_search`（実URLはリンクの `churl` から復元します）

ツール呼び出し:

```json
{
  "name": "bing_web_search",
  "arguments": { "query": "味噌ラーメン 作り方", "limit": 5 }
}
```

レスポンス（抜粋）:

```json
{
  "provider": "bing",
  "vertical": "web",
  "ok": true,
  "query": "味噌ラーメン 作り方",
  "page": 1,
  "count": 5,
  "total_hits": 70500,
  "next_page": 2,
  "items": [
    {
      "rank": 1,
      "title": "味噌ラーメンのレシピ",
      "url": "https://example.com/recipe",
      "display_url": "https://example.com › recipe",
      "snippet": "手順 · 1. キャベツは食べやすい大きさに切る。…"
    }
  ]
}
```

ウェブ検索の結果リンクは、Bingの計測リダイレクト（`/ck/a`）を展開した実URLで返ります。

## 設計

```
crates/
├── providers/bing/           # 検索ロジック
│   ├── common.rs             #   HTTP・HTML整形・リダイレクト展開
│   ├── urlbuild.rs           #   検索URLの生成
│   ├── web.rs image.rs video.rs news.rs suggest.rs
│   └── lib.rs                #   HTTPクライアントと共通レスポンス型
├── server/                   # MCPサーバー本体（stdio JSON-RPC 2.0）
│   ├── main.rs               #   プロトコル処理
│   └── tools.rs              #   ツール定義とディスパッチ
└── launcher/                 # 登録用GUI（egui）
```

作り方は、Bing公式AndroidアプリのAPK解析で確認した仕様に基づきます。アプリには署名付きの
ネイティブ検索APIはなく、WebViewで `https://www.bing.com/search?q=%s&form=%s&sdkhh=1` を
開いているだけでした。本サーバーも同じく、Bingがブラウザへ配信しているサーバーレンダリング
済みの結果ページをセレクタで読み取ります。証明書ピン留めやログインも検索経路にはありません。

### 画像検索だけ取得方式が違う理由

デスクトップ版の画像検索ページは、結果をJavaScriptの非同期読み込みに寄せていて、初期HTMLに
結果が入っていません。アプリ内WebViewが受け取る形（モバイルUA＋アプリ方式パラメータ
`form=L2GV01&sdkhh=1`＋`mkt`/`setlang`）で取得すると、初期HTMLに結果が入って返ります。
`bing_image_search` だけがこの方式を使います。この形式のページには総件数表示がないため、
画像検索の結果に `total` は付きません。

### サジェストの appid について

サジェストのバックエンドだけは `appid` を要求します。これはユーザーの秘密情報ではなく、
Bingアプリに埋め込まれている公開値です（アプリの検索ボックスが使っているものと同じ）。
ユーザーが用意するものは何もありません。

## 開発

```bash
cargo test --workspace                  # 全テスト（実際にBingへ通信するE2E含む）
cargo test --workspace -- --skip e2e_   # ネットワーク不要のテストのみ
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

`e2e_*` テストは実際の Bing に接続し、各検索が正しく解析できているかを検証します。
パーサーは解析時に保存した実際の結果ページ（フィクスチャ）でもオフライン検証しています。

stdio越しの手動確認:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"x","version":"1"}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"bing_web_search","arguments":{"query":"ラーメン","limit":2}}}' \
  | ./target/debug/bing-mcp
```

## 注意

- 公開されている検索結果ページを読み取ります。常識的な頻度で利用してください。
- Bing がページ構造を変更した場合、該当する検索が一時的に失敗することがあります。エラーメッセージにその旨が示されます。
- 本プロジェクトは Microsoft とは無関係です。Bing は Microsoft グループの商標です。

## ライセンス

MIT — [LICENSE](LICENSE) を参照。
