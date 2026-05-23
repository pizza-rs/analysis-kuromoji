//! Kuromoji baseform token filter.
//!
//! Replaces conjugated token surface forms with their base (dictionary) form.
//! In IPADIC, the base form is stored at index 6 in the detail fields.
//!
//! This filter relies on POS detail information attached during tokenization.
//! It works with tokens that carry IPADIC-format metadata.

use alloc::borrow::Cow;
use alloc::sync::Arc;

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode as LinderaMode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer as LinderaTokenizer;

use pizza_engine::analysis::{Token, TokenFilter};

/// Reduces conjugated Japanese tokens to their base/dictionary form.
///
/// For example, 食べた → 食べる, 走った → 走る.
///
/// This filter re-tokenizes the input to access IPADIC morphological details,
/// extracting the base form (原形) from position 6 in the detail array.
#[derive(Clone)]
pub struct KuromojiBaseformFilter {
    inner: Arc<LinderaTokenizer>,
}

impl KuromojiBaseformFilter {
    /// Create a new baseform filter.
    pub fn new() -> Self {
        let dictionary = load_dictionary("embedded://ipadic")
            .expect("failed to load embedded IPADIC dictionary");
        let segmenter = Segmenter::new(LinderaMode::Normal, dictionary, None);
        let tokenizer = LinderaTokenizer::new(segmenter);
        Self {
            inner: Arc::new(tokenizer),
        }
    }

    /// Look up the base form of the given surface text using lindera.
    fn get_baseform(&self, surface: &str) -> Option<String> {
        let tokens = self.inner.tokenize(surface).ok()?;
        if tokens.len() != 1 {
            return None;
        }
        let token = &tokens[0];
        if let Some(ref details) = token.details {
            // IPADIC detail[6] = base form (原形)
            if details.len() > 6 && details[6] != "*" && details[6] != surface {
                return Some(details[6].to_string());
            }
        }
        None
    }
}

impl Default for KuromojiBaseformFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for KuromojiBaseformFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let surface = token.term.to_string();
        if let Some(baseform) = self.get_baseform(&surface) {
            token.term = Cow::Owned(baseform);
        }
        (false, None)
    }
}
