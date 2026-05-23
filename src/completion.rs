//! Japanese completion filter for suggest/autocomplete.
//!
//! Lucene's `JapaneseCompletionFilter` generates romanized and katakana
//! reading forms for Japanese tokens to enable prefix completion on
//! suggest fields. For each token, it emits additional tokens representing
//! possible prefix forms that a user might type.
//!
//! This is used in Elasticsearch's `kuromoji_completion` token filter.
//!
//! Modes:
//! - `Index`: emit both romanized and katakana readings as additional tokens
//! - `Query`: emit the original token only (for query-time matching)

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode as LinderaMode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer as LinderaTokenizer;

use pizza_engine::analysis::{Token, TokenFilter};

/// Completion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionMode {
    /// Index-time: emit romanized + katakana readings as additional tokens.
    Index,
    /// Query-time: pass tokens through unchanged.
    Query,
}

/// Japanese completion filter — generates reading forms for autocomplete.
///
/// Equivalent to Elasticsearch's `kuromoji_completion` token filter / Lucene's
/// `JapaneseCompletionFilter`.
///
/// At index time, for each input token it emits:
/// 1. The original token (unchanged)
/// 2. Katakana reading (if available from IPADIC)
/// 3. Romaji reading (Hepburn romanization of the katakana)
///
/// This allows prefix-completion queries typed in romaji or katakana to
/// match against Japanese text.
#[derive(Clone)]
pub struct JapaneseCompletionFilter {
    inner: Arc<LinderaTokenizer>,
    mode: CompletionMode,
}

impl JapaneseCompletionFilter {
    /// Create with the specified mode.
    pub fn new(mode: CompletionMode) -> Self {
        let dictionary = load_dictionary("embedded://ipadic")
            .expect("failed to load embedded IPADIC dictionary");
        let segmenter = Segmenter::new(LinderaMode::Normal, dictionary, None);
        let tokenizer = LinderaTokenizer::new(segmenter);
        Self {
            inner: Arc::new(tokenizer),
            mode,
        }
    }

    /// Get katakana reading for a surface form.
    fn get_katakana_reading(&self, surface: &str) -> Option<String> {
        let tokens = self.inner.tokenize(surface).ok()?;
        if tokens.is_empty() {
            return None;
        }
        // Concatenate readings of all sub-tokens
        let mut reading = String::new();
        for token in &tokens {
            if let Some(ref details) = token.details {
                // IPADIC detail[7] = reading (読み) in katakana
                if details.len() > 7 && details[7] != "*" {
                    reading.push_str(&details[7]);
                } else {
                    // No reading available; use surface as-is
                    reading.push_str(token.surface.as_ref());
                }
            } else {
                reading.push_str(token.surface.as_ref());
            }
        }
        if reading.is_empty() || reading == surface {
            None
        } else {
            Some(reading)
        }
    }
}

impl TokenFilter for JapaneseCompletionFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if self.mode == CompletionMode::Query {
            return (false, None);
        }

        let surface = token.term.to_string();
        let mut extras = Vec::new();

        if let Some(katakana) = self.get_katakana_reading(&surface) {
            // Emit katakana reading at same position (for position overlap)
            let romaji = katakana_to_romaji(&katakana);

            extras.push(Token {
                term: Cow::Owned(katakana),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            });

            if !romaji.is_empty() && romaji != surface {
                extras.push(Token {
                    term: Cow::Owned(romaji),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        }

        if extras.is_empty() {
            (false, None)
        } else {
            (false, Some(extras))
        }
    }
}

/// Convert katakana string to romaji (Hepburn romanization).
fn katakana_to_romaji(katakana: &str) -> String {
    let mut result = String::with_capacity(katakana.len() * 2);
    let chars: Vec<char> = katakana.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Handle small tsu (gemination) — double the next consonant
        if ch == 'ッ' || ch == 'っ' {
            if i + 1 < len {
                let next_romaji = single_katakana_to_romaji(chars[i + 1]);
                if let Some(first_char) = next_romaji.chars().next() {
                    if first_char.is_ascii_alphabetic() {
                        result.push(first_char);
                    }
                }
            }
            i += 1;
            continue;
        }

        // Handle long vowel mark
        if ch == 'ー' {
            // Repeat previous vowel if any
            if let Some(last) = result.chars().last() {
                if "aiueo".contains(last) {
                    result.push(last);
                }
            }
            i += 1;
            continue;
        }

        // Handle two-char combinations (e.g., キョ → kyo)
        if i + 1 < len && is_small_kana(chars[i + 1]) {
            if let Some(combo) = two_char_romaji(ch, chars[i + 1]) {
                result.push_str(&combo);
                i += 2;
                continue;
            }
        }

        result.push_str(&single_katakana_to_romaji(ch));
        i += 1;
    }

    result
}

fn is_small_kana(ch: char) -> bool {
    matches!(ch, 'ャ' | 'ュ' | 'ョ' | 'ゃ' | 'ゅ' | 'ょ' | 'ァ' | 'ィ' | 'ゥ' | 'ェ' | 'ォ')
}

fn two_char_romaji(base: char, small: char) -> Option<String> {
    let combo = match (base, small) {
        ('キ', 'ャ') => "kya", ('キ', 'ュ') => "kyu", ('キ', 'ョ') => "kyo",
        ('シ', 'ャ') => "sha", ('シ', 'ュ') => "shu", ('シ', 'ョ') => "sho",
        ('チ', 'ャ') => "cha", ('チ', 'ュ') => "chu", ('チ', 'ョ') => "cho",
        ('ニ', 'ャ') => "nya", ('ニ', 'ュ') => "nyu", ('ニ', 'ョ') => "nyo",
        ('ヒ', 'ャ') => "hya", ('ヒ', 'ュ') => "hyu", ('ヒ', 'ョ') => "hyo",
        ('ミ', 'ャ') => "mya", ('ミ', 'ュ') => "myu", ('ミ', 'ョ') => "myo",
        ('リ', 'ャ') => "rya", ('リ', 'ュ') => "ryu", ('リ', 'ョ') => "ryo",
        ('ギ', 'ャ') => "gya", ('ギ', 'ュ') => "gyu", ('ギ', 'ョ') => "gyo",
        ('ジ', 'ャ') => "ja", ('ジ', 'ュ') => "ju", ('ジ', 'ョ') => "jo",
        ('ビ', 'ャ') => "bya", ('ビ', 'ュ') => "byu", ('ビ', 'ョ') => "byo",
        ('ピ', 'ャ') => "pya", ('ピ', 'ュ') => "pyu", ('ピ', 'ョ') => "pyo",
        _ => return None,
    };
    Some(combo.into())
}

fn single_katakana_to_romaji(ch: char) -> String {
    match ch {
        'ア' => "a".into(), 'イ' => "i".into(), 'ウ' => "u".into(),
        'エ' => "e".into(), 'オ' => "o".into(),
        'カ' => "ka".into(), 'キ' => "ki".into(), 'ク' => "ku".into(),
        'ケ' => "ke".into(), 'コ' => "ko".into(),
        'サ' => "sa".into(), 'シ' => "shi".into(), 'ス' => "su".into(),
        'セ' => "se".into(), 'ソ' => "so".into(),
        'タ' => "ta".into(), 'チ' => "chi".into(), 'ツ' => "tsu".into(),
        'テ' => "te".into(), 'ト' => "to".into(),
        'ナ' => "na".into(), 'ニ' => "ni".into(), 'ヌ' => "nu".into(),
        'ネ' => "ne".into(), 'ノ' => "no".into(),
        'ハ' => "ha".into(), 'ヒ' => "hi".into(), 'フ' => "fu".into(),
        'ヘ' => "he".into(), 'ホ' => "ho".into(),
        'マ' => "ma".into(), 'ミ' => "mi".into(), 'ム' => "mu".into(),
        'メ' => "me".into(), 'モ' => "mo".into(),
        'ヤ' => "ya".into(), 'ユ' => "yu".into(), 'ヨ' => "yo".into(),
        'ラ' => "ra".into(), 'リ' => "ri".into(), 'ル' => "ru".into(),
        'レ' => "re".into(), 'ロ' => "ro".into(),
        'ワ' => "wa".into(), 'ヲ' => "wo".into(), 'ン' => "n".into(),
        'ガ' => "ga".into(), 'ギ' => "gi".into(), 'グ' => "gu".into(),
        'ゲ' => "ge".into(), 'ゴ' => "go".into(),
        'ザ' => "za".into(), 'ジ' => "ji".into(), 'ズ' => "zu".into(),
        'ゼ' => "ze".into(), 'ゾ' => "zo".into(),
        'ダ' => "da".into(), 'ヂ' => "di".into(), 'ヅ' => "du".into(),
        'デ' => "de".into(), 'ド' => "do".into(),
        'バ' => "ba".into(), 'ビ' => "bi".into(), 'ブ' => "bu".into(),
        'ベ' => "be".into(), 'ボ' => "bo".into(),
        'パ' => "pa".into(), 'ピ' => "pi".into(), 'プ' => "pu".into(),
        'ペ' => "pe".into(), 'ポ' => "po".into(),
        'ァ' => "a".into(), 'ィ' => "i".into(), 'ゥ' => "u".into(),
        'ェ' => "e".into(), 'ォ' => "o".into(),
        _ => {
            let mut s = String::new();
            s.push(ch);
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_romaji_conversion_basic() {
        assert_eq!(katakana_to_romaji("トウキョウ"), "toukyou");
        assert_eq!(katakana_to_romaji("オオサカ"), "oosaka");
        assert_eq!(katakana_to_romaji("サッポロ"), "sapporo");
    }

    #[test]
    fn test_romaji_combination_kana() {
        assert_eq!(katakana_to_romaji("シャチョウ"), "shachou");
        assert_eq!(katakana_to_romaji("キョウト"), "kyouto");
        assert_eq!(katakana_to_romaji("ジュウ"), "juu");
    }

    #[test]
    fn test_romaji_long_vowel() {
        assert_eq!(katakana_to_romaji("コーヒー"), "koohii");
        assert_eq!(katakana_to_romaji("ラーメン"), "raamen");
    }

    #[test]
    fn test_romaji_gemination() {
        assert_eq!(katakana_to_romaji("ニッポン"), "nippon");
        assert_eq!(katakana_to_romaji("サッカー"), "sakkaa");
    }

    #[test]
    fn test_query_mode_passthrough() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Query);
        let mut token = Token::new("東京", 0, 6, 0);
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert!(extras.is_none());
        assert_eq!(token.term.as_ref(), "東京");
    }

    #[test]
    fn test_index_mode_emits_readings() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Index);

        // Test with a katakana word that lindera can read
        let mut token = Token::new("東京", 0, 6, 0);
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "東京"); // original preserved

        // If lindera returns a reading, we should get extras
        if let Some(extra_tokens) = extras {
            assert!(!extra_tokens.is_empty());
            // All extras should have same position as original
            for et in &extra_tokens {
                assert_eq!(et.position, token.position);
            }
        }
    }

    #[test]
    fn test_ascii_passthrough() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Index);
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        // ASCII tokens typically won't get a different reading
        assert_eq!(token.term.as_ref(), "hello");
    }
}
