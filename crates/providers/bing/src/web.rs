//! Bing web search (`https://www.bing.com/search`).
//!
//! Results are server-rendered into `ol#b_results > li.b_algo` for a
//! desktop user agent *provided* the market is pinned with
//! `mkt`/`setlang` (APK analysis round 1: without them Bing answers with
//! an empty "no results" page). Most result links are click-tracking
//! redirects; [`decode_click_url`] unwraps them to the real destination.

use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::common::*;
use crate::urlbuild::{build_page_url, WEB_PER_PAGE};
use crate::{BingSearchClient, SearchResult};

static RESULTS_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("ol#b_results > li.b_algo").unwrap());
static NO_RESULTS_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("li.b_no").unwrap());
static TITLE_LINK_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h2 > a").unwrap());
static CITE_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("cite").unwrap());
static SNIPPET_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".b_caption p").unwrap());
static CAPTION_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".b_caption").unwrap());
static COUNT_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".sb_count").unwrap());
static NEXT_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("a.sb_pagN").unwrap());

/// One organic web result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebItem {
    pub rank: i64,
    pub title: String,
    pub url: String,
    /// The breadcrumb-style URL Bing displays (`example.com › page`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// Parsed web results page: items plus the paging facts the envelope needs.
#[derive(Debug, Clone)]
pub struct WebResults {
    pub items: Vec<WebItem>,
    pub total_hits: Option<i64>,
    pub has_next_page: bool,
}

/// Parse a saved Bing web results page. `limit` caps the returned items;
/// ranks are 1-based *within the page* (the client re-bases them).
pub fn parse_web_results(html: &str, limit: usize) -> Result<WebResults> {
    let doc = Html::parse_document(html);
    let total_hits = doc
        .select(&COUNT_SEL)
        .next()
        .map(|el| parse_total_count(&el.text().collect::<String>()))
        .unwrap_or(None);
    let has_next_page = doc.select(&NEXT_SEL).next().is_some();

    let mut items = Vec::new();
    for el in doc.select(&RESULTS_SEL) {
        if items.len() >= limit {
            break;
        }
        let Some(link) = el.select(&TITLE_LINK_SEL).next() else {
            continue;
        };
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let title = clean_text(&link.text().collect::<String>());
        let url = decode_click_url(href);
        if title.is_empty() {
            continue;
        }
        let display_url = el
            .select(&CITE_SEL)
            .next()
            .map(|c| clean_text(&c.text().collect::<String>()))
            .filter(|s| !s.is_empty());
        let snippet = el
            .select(&SNIPPET_SEL)
            .next()
            .or_else(|| el.select(&CAPTION_SEL).next())
            .map(|p| clean_text(&p.text().collect::<String>()))
            .filter(|s| !s.is_empty());
        items.push(WebItem {
            rank: items.len() as i64 + 1,
            title,
            url,
            display_url,
            snippet,
        });
    }

    if items.is_empty() && doc.select(&NO_RESULTS_SEL).next().is_none() {
        // Neither results nor an explicit "no results" block: the layout
        // changed, or a block page was served.
        return Err(layout_error("web"));
    }

    Ok(WebResults {
        items,
        total_hits,
        has_next_page,
    })
}

impl BingSearchClient {
    /// Search the web index. `page` is 1-based (converted to Bing's
    /// `first` offset); `limit` is clamped to 10, the page size.
    pub async fn web_search(
        &self,
        query: &str,
        page: i64,
        limit: i64,
        mkt: &str,
        setlang: &str,
    ) -> Result<SearchResult<WebItem>> {
        let page = page.max(1);
        let limit = clamp_limit(limit, WEB_PER_PAGE);
        let url = build_page_url(query, "web", page, mkt, setlang)?;
        let html = self.get_html(&url, "https://www.bing.com/").await?;
        let parsed = parse_web_results(&html, limit)?;
        let mut items = parsed.items;
        for item in &mut items {
            item.rank += (page - 1) * WEB_PER_PAGE;
        }
        let count = items.len();
        Ok(SearchResult::new("web", query, page, items)
            .with_total(parsed.total_hits)
            .with_next_page(parsed.has_next_page.then_some(page + 1))
            .with_note_opt((count == 0).then(|| "Bing returned no results".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_shell_is_a_layout_error() {
        let html = "<html><body><p>shell only</p></body></html>";
        assert!(parse_web_results(html, 10).is_err());
    }

    #[test]
    fn parse_explicit_no_results_is_ok() {
        let html = r#"<html><body><ol id="b_results"><li class="b_no">結果がありません</li></ol></body></html>"#;
        let parsed = parse_web_results(html, 10).unwrap();
        assert!(parsed.items.is_empty());
    }
}
