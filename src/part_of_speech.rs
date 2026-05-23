//! Kuromoji part-of-speech stop filter.
//!
//! Removes tokens whose part-of-speech matches any of the configured stop tags.
//! POS tags in IPADIC are hierarchical, e.g. "助詞,格助詞,一般".
//! A configured tag "助詞" will match "助詞,格助詞,一般" (prefix matching).

use hashbrown::HashSet;
use alloc::sync::Arc;

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode as LinderaMode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer as LinderaTokenizer;

use pizza_engine::analysis::{Token, TokenFilter};

/// Default Japanese stop tags (functional words that add little search value).
pub const DEFAULT_STOP_TAGS: &[&str] = &[
    "助詞",
    "助動詞",
    "接続詞",
    "記号",
    "フィラー",
    "非言語音",
];

/// Removes tokens matching specified part-of-speech tags.
///
/// Equivalent to Elasticsearch's `kuromoji_part_of_speech` filter.
#[derive(Clone)]
pub struct KuromojiPartOfSpeechFilter {
    inner: Arc<LinderaTokenizer>,
    stop_tags: HashSet<String>,
}

impl KuromojiPartOfSpeechFilter {
    /// Create with a set of POS stop tags.
    /// Tags are matched by prefix: "助詞" matches "助詞,格助詞,一般".
    pub fn new(stop_tags: Vec<String>) -> Self {
        let dictionary = load_dictionary("embedded://ipadic")
            .expect("failed to load embedded IPADIC dictionary");
        let segmenter = Segmenter::new(LinderaMode::Normal, dictionary, None);
        let tokenizer = LinderaTokenizer::new(segmenter);
        Self {
            inner: Arc::new(tokenizer),
            stop_tags: stop_tags.into_iter().collect(),
        }
    }

    /// Create with default Japanese stop tags.
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_STOP_TAGS.iter().map(|s| s.to_string()).collect())
    }

    /// Check if token's POS matches any stop tag.
    fn should_remove(&self, surface: &str) -> bool {
        let tokens = match self.inner.tokenize(surface) {
            Ok(t) => t,
            Err(_) => return false,
        };
        if tokens.len() != 1 {
            return false;
        }
        let token = &tokens[0];
        if let Some(ref details) = token.details {
            // Build full POS tag string: "名詞,一般,*,*"
            let pos = details.iter().take(4).cloned().collect::<Vec<_>>().join(",");
            // Check if any stop tag is a prefix of the full POS
            for tag in &self.stop_tags {
                if pos.starts_with(tag.as_str()) {
                    return true;
                }
            }
        }
        false
    }
}

impl TokenFilter for KuromojiPartOfSpeechFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let surface = token.term.to_string();
        let deleted = self.should_remove(&surface);
        (deleted, None)
    }
}
