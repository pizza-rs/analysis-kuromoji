//! Kuromoji reading form token filter.
//!
//! Replaces token surface with its reading in katakana or romaji.
//! IPADIC stores the reading at detail index 7.

use alloc::borrow::Cow;
use alloc::sync::Arc;

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode as LinderaMode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer as LinderaTokenizer;

use pizza_engine::analysis::{Token, TokenFilter};

/// Output format for the reading form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingFormType {
    /// Output katakana reading (カタカナ).
    Katakana,
    /// Output romaji reading (romanization of katakana).
    Romaji,
}

/// Replaces token surface with the reading form (katakana or romaji).
///
/// Equivalent to Elasticsearch's `kuromoji_readingform` filter.
#[derive(Clone)]
pub struct KuromojiReadingformFilter {
    inner: Arc<LinderaTokenizer>,
    reading_type: ReadingFormType,
}

impl KuromojiReadingformFilter {
    /// Create a reading form filter with the specified output type.
    pub fn new(reading_type: ReadingFormType) -> Self {
        let dictionary = load_dictionary("embedded://ipadic")
            .expect("failed to load embedded IPADIC dictionary");
        let segmenter = Segmenter::new(LinderaMode::Normal, dictionary, None);
        let tokenizer = LinderaTokenizer::new(segmenter);
        Self {
            inner: Arc::new(tokenizer),
            reading_type,
        }
    }

    /// Get the reading for a token surface.
    fn get_reading(&self, surface: &str) -> Option<String> {
        let tokens = self.inner.tokenize(surface).ok()?;
        if tokens.len() != 1 {
            return None;
        }
        let token = &tokens[0];
        if let Some(ref details) = token.details {
            // IPADIC detail[7] = reading (読み) in katakana
            if details.len() > 7 && details[7] != "*" {
                let katakana = &details[7];
                match self.reading_type {
                    ReadingFormType::Katakana => return Some(katakana.to_string()),
                    ReadingFormType::Romaji => return Some(katakana_to_romaji(katakana)),
                }
            }
        }
        None
    }
}

impl TokenFilter for KuromojiReadingformFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let surface = token.term.to_string();
        if let Some(reading) = self.get_reading(&surface) {
            token.term = Cow::Owned(reading);
        }
        (false, None)
    }
}

/// Convert katakana string to romaji (Hepburn romanization).
fn katakana_to_romaji(katakana: &str) -> String {
    let mut result = String::with_capacity(katakana.len());
    let chars: Vec<char> = katakana.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Handle small tsu (ッ) — doubles the next consonant
        if ch == 'ッ' {
            if i + 1 < chars.len() {
                let next_romaji = single_katakana_to_romaji(chars[i + 1]);
                if let Some(first_consonant) = next_romaji.chars().next() {
                    if first_consonant != 'a'
                        && first_consonant != 'i'
                        && first_consonant != 'u'
                        && first_consonant != 'e'
                        && first_consonant != 'o'
                    {
                        result.push(first_consonant);
                    }
                }
            }
            i += 1;
            continue;
        }

        // Handle long vowel mark (ー) — repeat previous vowel
        if ch == 'ー' {
            if let Some(last) = result.chars().last() {
                result.push(last);
            }
            i += 1;
            continue;
        }

        // Handle compound kana (キャ, シュ, チョ, etc.)
        if i + 1 < chars.len() && is_small_kana(chars[i + 1]) {
            if let Some(compound) = compound_katakana_to_romaji(ch, chars[i + 1]) {
                result.push_str(&compound);
                i += 2;
                continue;
            }
        }

        result.push_str(single_katakana_to_romaji(ch));
        i += 1;
    }

    result
}

fn is_small_kana(ch: char) -> bool {
    matches!(ch, 'ャ' | 'ュ' | 'ョ' | 'ァ' | 'ィ' | 'ゥ' | 'ェ' | 'ォ')
}

fn compound_katakana_to_romaji(main: char, small: char) -> Option<String> {
    let base = match main {
        'キ' => "ky",
        'シ' => "sh",
        'チ' => "ch",
        'ニ' => "ny",
        'ヒ' => "hy",
        'ミ' => "my",
        'リ' => "ry",
        'ギ' => "gy",
        'ジ' => "j",
        'ビ' => "by",
        'ピ' => "py",
        _ => return None,
    };
    let vowel = match small {
        'ャ' => "a",
        'ュ' => "u",
        'ョ' => "o",
        'ァ' => "a",
        'ィ' => "i",
        'ゥ' => "u",
        'ェ' => "e",
        'ォ' => "o",
        _ => return None,
    };
    Some(format!("{}{}", base, vowel))
}

fn single_katakana_to_romaji(ch: char) -> &'static str {
    match ch {
        'ア' => "a",
        'イ' => "i",
        'ウ' => "u",
        'エ' => "e",
        'オ' => "o",
        'カ' => "ka",
        'キ' => "ki",
        'ク' => "ku",
        'ケ' => "ke",
        'コ' => "ko",
        'サ' => "sa",
        'シ' => "shi",
        'ス' => "su",
        'セ' => "se",
        'ソ' => "so",
        'タ' => "ta",
        'チ' => "chi",
        'ツ' => "tsu",
        'テ' => "te",
        'ト' => "to",
        'ナ' => "na",
        'ニ' => "ni",
        'ヌ' => "nu",
        'ネ' => "ne",
        'ノ' => "no",
        'ハ' => "ha",
        'ヒ' => "hi",
        'フ' => "fu",
        'ヘ' => "he",
        'ホ' => "ho",
        'マ' => "ma",
        'ミ' => "mi",
        'ム' => "mu",
        'メ' => "me",
        'モ' => "mo",
        'ヤ' => "ya",
        'ユ' => "yu",
        'ヨ' => "yo",
        'ラ' => "ra",
        'リ' => "ri",
        'ル' => "ru",
        'レ' => "re",
        'ロ' => "ro",
        'ワ' => "wa",
        'ヲ' => "wo",
        'ン' => "n",
        'ガ' => "ga",
        'ギ' => "gi",
        'グ' => "gu",
        'ゲ' => "ge",
        'ゴ' => "go",
        'ザ' => "za",
        'ジ' => "ji",
        'ズ' => "zu",
        'ゼ' => "ze",
        'ゾ' => "zo",
        'ダ' => "da",
        'ヂ' => "di",
        'ヅ' => "du",
        'デ' => "de",
        'ド' => "do",
        'バ' => "ba",
        'ビ' => "bi",
        'ブ' => "bu",
        'ベ' => "be",
        'ボ' => "bo",
        'パ' => "pa",
        'ピ' => "pi",
        'プ' => "pu",
        'ペ' => "pe",
        'ポ' => "po",
        'ヴ' => "vu",
        _ => "",
    }
}
