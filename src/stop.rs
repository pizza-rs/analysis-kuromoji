//! Japanese stop word filter.
//!
//! Removes common Japanese stop words (particles, auxiliary verbs, etc.)
//! that add little value to full-text search.

use hashbrown::HashSet;

use pizza_engine::analysis::{Token, TokenFilter};

/// Default Japanese stop words (common particles, copulas, punctuation words).
pub const JAPANESE_STOP_WORDS: &[&str] = &[
    "の", "に", "は", "を", "た", "が", "で", "て", "と", "し",
    "れ", "さ", "ある", "いる", "も", "する", "から", "な", "こと",
    "として", "い", "や", "れる", "など", "なっ", "ない", "この",
    "ため", "その", "あっ", "よう", "また", "もの", "という", "あり",
    "まで", "られ", "なる", "へ", "か", "だ", "これ", "によって",
    "により", "おり", "より", "による", "ず", "なり", "られる",
    "において", "に対して", "ほか", "ながら", "うち", "そして",
    "とともに", "ただし", "かつて", "それぞれ", "または", "お",
    "ほど", "ものの", "についで", "あ", "う", "え", "お", "か",
    "き", "く", "け", "こ",
];

/// Removes Japanese stop words from the token stream.
///
/// Equivalent to Elasticsearch's `ja_stop` filter.
#[derive(Clone)]
pub struct JapaneseStopFilter {
    stop_words: HashSet<String>,
}

impl JapaneseStopFilter {
    /// Create with default Japanese stop words.
    pub fn new() -> Self {
        Self {
            stop_words: JAPANESE_STOP_WORDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    /// Create with custom stop words.
    pub fn with_words(words: Vec<String>) -> Self {
        Self {
            stop_words: words.into_iter().collect(),
        }
    }
}

impl Default for JapaneseStopFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for JapaneseStopFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let deleted = self.stop_words.contains(token.term.as_ref());
        (deleted, None)
    }
}
