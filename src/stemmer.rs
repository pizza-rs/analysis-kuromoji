//! Kuromoji katakana stemmer filter.
//!
//! Stems katakana words by removing trailing long vowel marks (ー).
//! This normalizes loanwords that may vary in their trailing prolonged
//! sound mark usage (e.g., コンピューター → コンピュータ).
//!
//! Only stems tokens that are entirely katakana and whose length
//! exceeds a configurable minimum.

use alloc::borrow::Cow;

use pizza_engine::analysis::{Token, TokenFilter};

/// Default minimum token length (in chars) before stemming applies.
const DEFAULT_MIN_LENGTH: usize = 4;

/// Stems katakana tokens by removing trailing long vowel marks (ー).
///
/// Equivalent to Elasticsearch's `kuromoji_stemmer` filter.
#[derive(Clone, Debug)]
pub struct KuromojiStemmerFilter {
    /// Minimum character length to apply stemming.
    min_length: usize,
}

impl KuromojiStemmerFilter {
    /// Create with the default minimum length (4).
    pub fn new() -> Self {
        Self {
            min_length: DEFAULT_MIN_LENGTH,
        }
    }

    /// Create with a custom minimum character length.
    pub fn with_min_length(min_length: usize) -> Self {
        Self { min_length }
    }
}

impl Default for KuromojiStemmerFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for KuromojiStemmerFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let chars: Vec<char> = term.chars().collect();

        // Only stem if token is all katakana and long enough
        if chars.len() < self.min_length || !is_all_katakana(&chars) {
            return (false, None);
        }

        // Remove trailing ー (long vowel mark)
        if chars.last() == Some(&'ー') {
            let stemmed: String = chars[..chars.len() - 1].iter().collect();
            token.term = Cow::Owned(stemmed);
        }

        (false, None)
    }
}

/// Check if all characters in the slice are katakana (including ー).
fn is_all_katakana(chars: &[char]) -> bool {
    chars.iter().all(|&ch| {
        ('\u{30A0}'..='\u{30FF}').contains(&ch)  // Katakana block
            || ch == '\u{30FC}'                    // Long vowel mark (ー)
            || ('\u{31F0}'..='\u{31FF}').contains(&ch) // Katakana Phonetic Extensions
    })
}
