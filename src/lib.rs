#![cfg_attr(not(feature = "std"), no_std)]
//! Japanese morphological analysis for Pizza search engine.
//!
//! This crate provides Kuromoji-compatible Japanese text analysis using
//! [lindera](https://github.com/lindera/lindera) with the IPADIC dictionary.
//!
//! # Components
//!
//! - [`KuromojiTokenizer`] — Japanese morphological tokenizer (normal/search/extended modes)
//! - [`KuromojiBaseformFilter`] — Reduce conjugated forms to base/dictionary form
//! - [`KuromojiPartOfSpeechFilter`] — Remove tokens by part-of-speech tags
//! - [`KuromojiReadingformFilter`] — Output katakana or romaji readings
//! - [`KuromojiStemmerFilter`] — Stem katakana long vowels (ー)
//! - [`KuromojiNumberFilter`] — Normalize Kanji numerals to Arabic digits
//! - [`JapaneseStopFilter`] — Japanese stop word removal
extern crate alloc;
mod baseform;
mod completion;
mod dict;
mod number;
mod part_of_speech;
mod readingform;
mod stemmer;
mod stop;
mod tokenizer;

pub use baseform::KuromojiBaseformFilter;
pub use completion::CompletionMode;
pub use completion::JapaneseCompletionFilter;
pub use number::KuromojiNumberFilter;
pub use part_of_speech::KuromojiPartOfSpeechFilter;
pub use readingform::KuromojiReadingformFilter;
pub use readingform::ReadingFormType;
pub use stemmer::KuromojiStemmerFilter;
pub use stop::JapaneseStopFilter;
pub use tokenizer::KuromojiMode;
pub use tokenizer::KuromojiTokenizer;
pub mod register;
pub use register::register_all;
