//! Kuromoji number filter.
//!
//! Converts Kanji numerals and fullwidth digit sequences in tokens to
//! normalized Arabic digit strings. This makes numeric searches consistent
//! regardless of input representation.

use alloc::borrow::Cow;

use pizza_engine::analysis::{Token, TokenFilter};

/// Normalizes Kanji/fullwidth numeral tokens to Arabic digits.
///
/// Examples:
/// - 一 → 1
/// - 十二 → 12
/// - ３４ → 34
/// - 百二十三 → 123
///
/// Equivalent to Elasticsearch's `kuromoji_number` filter.
#[derive(Clone, Debug, Default)]
pub struct KuromojiNumberFilter;

impl KuromojiNumberFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for KuromojiNumberFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();

        // Try fullwidth digit normalization first
        if term.chars().all(|ch| is_fullwidth_digit(ch) || ch == '．' || ch == '，') {
            let normalized: String = term
                .chars()
                .filter_map(|ch| {
                    if is_fullwidth_digit(ch) {
                        Some(fullwidth_to_ascii(ch))
                    } else if ch == '．' {
                        Some('.')
                    } else if ch == '，' {
                        Some(',')
                    } else {
                        None
                    }
                })
                .collect();
            if !normalized.is_empty() {
                token.term = Cow::Owned(normalized);
            }
            return (false, None);
        }

        // Try kanji numeral conversion
        if term.chars().all(|ch| is_kanji_numeral(ch)) && !term.is_empty() {
            if let Some(value) = parse_kanji_number(term) {
                token.term = Cow::Owned(value.to_string());
            }
        }

        (false, None)
    }
}

fn is_fullwidth_digit(ch: char) -> bool {
    ('０'..='９').contains(&ch)
}

fn fullwidth_to_ascii(ch: char) -> char {
    // Fullwidth digits ０-９ are U+FF10..U+FF19, ASCII 0-9 are U+0030..U+0039
    char::from(b'0' + (ch as u32 - '０' as u32) as u8)
}

fn is_kanji_numeral(ch: char) -> bool {
    matches!(
        ch,
        '〇' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十' | '百' | '千'
            | '万' | '億' | '兆'
    )
}

/// Parse a kanji numeral string into a u64 value.
/// Handles standard Sino-Japanese numeral patterns like 百二十三 → 123.
fn parse_kanji_number(s: &str) -> Option<u64> {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // Simple digit-only case (〇一二三 positional style)
    if chars.iter().all(|&ch| kanji_digit(ch).is_some()) {
        let result: String = chars.iter().filter_map(|&ch| kanji_digit(ch).map(|d| char::from(b'0' + d as u8))).collect();
        return result.parse::<u64>().ok();
    }

    // Multiplicative style (e.g., 百二十三)
    parse_kanji_multiplicative(&chars)
}

fn kanji_digit(ch: char) -> Option<u64> {
    match ch {
        '〇' => Some(0),
        '一' => Some(1),
        '二' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

fn kanji_multiplier(ch: char) -> Option<u64> {
    match ch {
        '十' => Some(10),
        '百' => Some(100),
        '千' => Some(1000),
        '万' => Some(10_000),
        '億' => Some(100_000_000),
        '兆' => Some(1_000_000_000_000),
        _ => None,
    }
}

/// Parse multiplicative kanji numbers (e.g., 三百二十一 → 321).
fn parse_kanji_multiplicative(chars: &[char]) -> Option<u64> {
    let mut total: u64 = 0;
    let mut current: u64 = 0;
    let mut big_unit_acc: u64 = 0;

    for &ch in chars {
        if let Some(d) = kanji_digit(ch) {
            current = d;
        } else if let Some(mult) = kanji_multiplier(ch) {
            if mult >= 10_000 {
                // Large unit (万, 億, 兆): accumulate everything below
                big_unit_acc += if current == 0 && total == 0 {
                    mult
                } else {
                    (total + current) * mult
                };
                total = 0;
                current = 0;
            } else {
                // Small unit (十, 百, 千): multiply current digit
                if current == 0 {
                    current = 1;
                }
                total += current * mult;
                current = 0;
            }
        }
    }

    let result = big_unit_acc + total + current;
    if result == 0 && !chars.iter().all(|&c| c == '〇') {
        None
    } else {
        Some(result)
    }
}
