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
mod tokenizer;
mod baseform;
mod part_of_speech;
mod readingform;
mod stemmer;
mod number;
mod stop;

pub use tokenizer::{KuromojiTokenizer, KuromojiMode};
pub use baseform::KuromojiBaseformFilter;
pub use part_of_speech::KuromojiPartOfSpeechFilter;
pub use readingform::{KuromojiReadingformFilter, ReadingFormType};
pub use stemmer::KuromojiStemmerFilter;
pub use number::KuromojiNumberFilter;
pub use stop::JapaneseStopFilter;
pub mod register;
pub use register::register_all;
