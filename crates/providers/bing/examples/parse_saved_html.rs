//! Parse saved Bing result markup without any network access.
//!
//! Run with: `cargo run -p bing-mcp-provider --example parse_saved_html`

use bing_search::{parse_image_results, parse_suggest_json, parse_web_results};

fn main() -> anyhow::Result<()> {
    // A minimal web result block, shaped like Bing's real markup.
    let web_html = r#"<ol id="b_results">
      <li class="b_algo">
        <h2><a href="https://www.bing.com/ck/a?!&amp;&amp;p=x&amp;u=a1aHR0cHM6Ly9leGFtcGxlLmNvbS9wYWdl">Example result</a></h2>
        <cite>https://example.com › page</cite>
        <div class="b_caption"><p>An example snippet.</p></div>
      </li>
    </ol>"#;
    let page = parse_web_results(web_html, 10)?;
    println!("web: {} item(s)", page.items.len());
    for item in &page.items {
        // The /ck/a redirect is unwrapped to the real destination.
        println!("  #{} {} -> {}", item.rank, item.title, item.url);
    }

    // One image unit: the `m` attribute carries the metadata JSON.
    let image_html = r##"<a class="iusc" href="/images/search?view=detailV2" m="{&quot;murl&quot;:&quot;https://example.com/cat.jpg&quot;,&quot;turl&quot;:&quot;https://ts1.mm.bing.net/th?id=OIP.x&amp;pid=15.1&quot;,&quot;purl&quot;:&quot;https://example.com/cats&quot;,&quot;t&quot;:&quot;A cat&quot;}">x</a>"##;
    let images = parse_image_results(image_html, 1, 12)?;
    println!("image: {} item(s)", images.items.len());
    for item in &images.items {
        println!("  {} -> {}", item.title, item.url);
    }

    // Suggestions API JSON.
    let json: serde_json::Value = serde_json::json!({
        "suggestionGroups": [{
            "name": "Web",
            "searchSuggestions": [
                { "displayText": "テスト", "query": "テスト", "url": "https://www.bing.com/search?q=%E3%83%86%E3%82%B9%E3%83%88" },
                { "displayText": "テストセンター", "query": "テストセンター", "url": "https://www.bing.com/search?q=x" }
            ]
        }]
    });
    let suggestions = parse_suggest_json(&json);
    println!("suggest: {} candidate(s)", suggestions.len());
    for s in &suggestions {
        println!("  {}", s.keyword);
    }
    Ok(())
}
