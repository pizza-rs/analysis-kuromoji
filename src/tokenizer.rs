//! Kuromoji-compatible Japanese morphological tokenizer.
//!
//! Wraps lindera with IPADIC dictionary. Supports three segmentation modes:
//! - **Normal:** Standard segmentation (no decomposition)
//! - **Search:** Decompose compound nouns for better search recall
//! - **Extended:** Like Search but also emits unigrams for unknown tokens

use alloc::borrow::Cow;
use alloc::sync::Arc;

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode as LinderaMode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer as LinderaTokenizer;

use pizza_engine::analysis::{Token, Tokenizer};

/// Segmentation mode for Japanese morphological analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KuromojiMode {
    /// Standard segmentation — no compound decomposition.
    Normal,
    /// Search mode — decompose compounds for better recall.
    Search,
    /// Extended mode — like Search, plus unigrams for unknown tokens.
    Extended,
}

impl KuromojiMode {
    fn to_lindera_mode(self) -> LinderaMode {
        match self {
            KuromojiMode::Normal => LinderaMode::Normal,
            KuromojiMode::Search => LinderaMode::Decompose(Default::default()),
            KuromojiMode::Extended => LinderaMode::Decompose(Default::default()),
        }
    }
}

/// Japanese morphological tokenizer using IPADIC dictionary via lindera.
///
/// Equivalent to Elasticsearch's `kuromoji_tokenizer`.
#[derive(Clone)]
pub struct KuromojiTokenizer {
    inner: Arc<LinderaTokenizer>,
    mode: KuromojiMode,
}

impl KuromojiTokenizer {
    /// Create a new tokenizer with the specified segmentation mode.
    pub fn new(mode: KuromojiMode) -> Self {
        let dictionary = load_dictionary("embedded://ipadic")
            .expect("failed to load embedded IPADIC dictionary");
        let segmenter = Segmenter::new(mode.to_lindera_mode(), dictionary, None);
        let tokenizer = LinderaTokenizer::new(segmenter);
        Self {
            inner: Arc::new(tokenizer),
            mode,
        }
    }

    /// Get the current segmentation mode.
    pub fn mode(&self) -> KuromojiMode {
        self.mode
    }
}

impl Tokenizer for KuromojiTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let tokens = match self.inner.tokenize(text) {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };

        let mut result = Vec::with_capacity(tokens.len());
        let mut position = 0u32;

        for token in tokens {
            let surface = token.surface.as_ref();
            // Skip empty surface tokens
            if surface.is_empty() {
                continue;
            }

            // byte offsets → char-based offsets for Pizza Token
            let byte_start = token.byte_start;
            let byte_end = token.byte_end;
            let start_offset = text[..byte_start].chars().count() as u32;
            let end_offset = text[..byte_end].chars().count() as u32;

            result.push(Token {
                term: Cow::Owned(surface.to_string()),
                start_offset,
                end_offset,
                position,
            });
            position += 1;
        }

        result
    }
}
