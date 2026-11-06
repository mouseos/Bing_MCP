//! Offline parser tests against real Bing result pages captured during
//! the APK analysis (Japanese market, query "テスト", 2026-11-06/07).
//! These assert counts and representative fields of the actual payloads.

use bing_search::{
    parse_image_results, parse_news_results, parse_suggest_json, parse_video_results,
    parse_web_results,
};

const WEB_HTML: &str = include_str!("fixtures/fixture-web-ja.html");
const VIDEO_HTML: &str = include_str!("fixtures/fixture-video-ja.html");
const NEWS_HTML: &str = include_str!("fixtures/fixture-news-ja.html");
const IMAGE_HTML: &str = include_str!("fixtures/fixture-image-ja.html");
const SUGGEST_JSON: &str = include_str!("fixtures/fixture-suggest-ja.json");

#[test]
fn web_fixture_parses_ten_results_with_total_and_next_page() {
    let page = parse_web_results(WEB_HTML, 50).unwrap();
    assert_eq!(page.items.len(), 10);
    assert_eq!(page.total_hits, Some(70500));
    assert!(page.has_next_page);

    let first = &page.items[0];
    assert_eq!(first.rank, 1);
    assert_eq!(first.title, "Bing");
    assert_eq!(first.url, "https://www.bing.com/version");
    assert_eq!(
        first.display_url.as_deref(),
        Some("https://www.bing.com › version")
    );

    // The second result's href is a Bing click-tracking redirect; the
    // parser must surface the decoded destination.
    let second = &page.items[1];
    assert_eq!(
        second.url,
        "https://news.mynavi.jp/techplus/article/20260902-4900008/"
    );
    assert!(second.title.contains("Windows Search"));
    assert!(second
        .snippet
        .as_deref()
        .unwrap_or("")
        .contains("Microsoftは8月27日"));
}

#[test]
fn web_fixture_limit_truncates() {
    let page = parse_web_results(WEB_HTML, 3).unwrap();
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[2].rank, 3);
}

#[test]
fn video_fixture_parses_units_with_churl_targets() {
    let page = parse_video_results(VIDEO_HTML, 50).unwrap();
    assert_eq!(page.total_parsed, 39);
    assert_eq!(page.items.len(), 39);

    let first = &page.items[0];
    assert_eq!(
        first.title,
        "ニセ天然vsガチ天然【テストの点数どっちが上？】替え歌で解説"
    );
    assert_eq!(first.url, "https://www.tiktok.com/@latte_1oo");
    assert_eq!(first.source.as_deref(), Some("TikTok"));
    assert_eq!(first.uploader.as_deref(), Some("latte_1oo"));
    assert_eq!(first.duration.as_deref(), Some("0:31"));
    assert_eq!(first.views.as_deref(), Some("視聴回数: 25.8万 回"));
    assert_eq!(first.upload_date.as_deref(), Some("1 週間前"));
    assert!(first
        .thumbnail
        .as_deref()
        .unwrap_or("")
        .contains("mm.bing.net"));
}

#[test]
fn news_fixture_parses_cards_from_attributes() {
    // 10 real cards (the page also has 10 inner `news-card-body` blocks,
    // which are not results).
    let items = parse_news_results(NEWS_HTML, 50).unwrap();
    assert_eq!(items.len(), 10);

    let first = &items[0];
    assert_eq!(
        first.url,
        "https://www.4gamer.net/games/037/G103718/20261106007/"
    );
    assert_eq!(first.publisher.as_deref(), Some("4Gamer.net"));
    assert!(first.title.starts_with("ダークファンタジーアクションRPG"));
    assert!(first
        .snippet
        .as_deref()
        .unwrap_or("")
        .contains("クローズドαテスト"));
    assert_eq!(first.published.as_deref(), Some("3 時間前"));
}

#[test]
fn image_fixture_parses_twelve_iusc_units() {
    let page = parse_image_results(IMAGE_HTML, 1, 50).unwrap();
    assert_eq!(page.items.len(), 12);
    assert!(page.has_next_page);

    let first = &page.items[0];
    assert_eq!(
        first.url,
        "https://wp1.kimini.online/blog/wp-content/uploads/2023/01/pixta_73448850_M-1118x538.jpg"
    );
    assert_eq!(
        first.thumbnail.as_deref(),
        Some("https://ts1.mm.bing.net/th?id=OIP.gXGZwFWrxMd8e-IGuQXbgwHaDk&pid=15.1")
    );
    assert_eq!(
        first.page_url.as_deref(),
        Some("https://kimini.online/blog/archives/28597")
    );
    assert!(first.title.contains("Kimini英会話"));
    assert!(first
        .detail_url
        .as_deref()
        .unwrap_or("")
        .starts_with("https://www.bing.com/images/search?view=detailV2"));
}

#[test]
fn suggest_fixture_parses_eight_candidates() {
    let value: serde_json::Value = serde_json::from_str(SUGGEST_JSON).unwrap();
    let suggestions = parse_suggest_json(&value);
    assert_eq!(suggestions.len(), 8);
    assert_eq!(suggestions[0].keyword, "テスト");
    assert_eq!(suggestions[1].keyword, "テストセンター");
    assert!(suggestions[0].search_url.contains("bing.com/search"));
}
