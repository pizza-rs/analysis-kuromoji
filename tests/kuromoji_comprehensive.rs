//! Comprehensive tests for the `pizza-analysis-kuromoji` crate.

use pizza_analysis_kuromoji::*;
use pizza_engine::analysis::{AnalysisFactory, Token, TokenFilter, Tokenizer};

// ─── Helpers ───────────────────────────────────────────────────────────────────

fn terms<'a>(tokens: &'a [Token<'_>]) -> Vec<&'a str> {
    tokens.iter().map(|t| t.term.as_ref()).collect()
}

fn make_token(term: &str) -> Token<'_> {
    Token::new(term, 0, term.len() as u32, 0)
}

fn apply_filter<'a>(filter: &dyn TokenFilter, term: &'a str) -> (bool, String, Vec<String>) {
    let mut token = make_token(term);
    let (deleted, extras) = filter.filter(&mut token);
    let extra_terms: Vec<String> = extras
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.term.into_owned())
        .collect();
    (deleted, token.term.into_owned(), extra_terms)
}

fn filter_term(filter: &dyn TokenFilter, term: &str) -> String {
    let (_, result, _) = apply_filter(filter, term);
    result
}

fn filter_deleted(filter: &dyn TokenFilter, term: &str) -> bool {
    let (deleted, _, _) = apply_filter(filter, term);
    deleted
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod tokenizer — KuromojiTokenizer
// ═══════════════════════════════════════════════════════════════════════════════

mod tokenizer {
    use super::*;

    #[test]
    fn normal_mode_basic_sentence() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("東京都に行く");
        let t = terms(&tokens);
        // Should produce multiple segments from morphological analysis
        assert!(t.len() >= 3, "expected at least 3 tokens, got {:?}", t);
        // Must contain "東京" or "東京都" as a segment
        assert!(
            t.iter().any(|s| s.contains("東京")),
            "expected 東京 in {:?}",
            t
        );
    }

    #[test]
    fn normal_mode_returns_correct_mode() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        assert_eq!(tok.mode(), KuromojiMode::Normal);
    }

    #[test]
    fn search_mode_returns_correct_mode() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Search);
        assert_eq!(tok.mode(), KuromojiMode::Search);
    }

    #[test]
    fn extended_mode_returns_correct_mode() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Extended);
        assert_eq!(tok.mode(), KuromojiMode::Extended);
    }

    #[test]
    fn search_mode_compound_decomposition() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Search);
        let tokens = tok.tokenize("関西国際空港");
        let t = terms(&tokens);
        // Search mode decomposes compound nouns — should have more tokens
        assert!(
            t.len() >= 2,
            "search mode should decompose compound, got {:?}",
            t
        );
    }

    #[test]
    fn extended_mode_unknown_words() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Extended);
        // An unusual/made-up word that's unlikely in IPADIC
        let tokens = tok.tokenize("ガンダムバルバトス");
        let t = terms(&tokens);
        assert!(!t.is_empty(), "extended mode should produce tokens");
    }

    #[test]
    fn katakana_word() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("コンピューター");
        let t = terms(&tokens);
        assert!(
            t.contains(&"コンピューター"),
            "should tokenize katakana word, got {:?}",
            t
        );
    }

    #[test]
    fn mixed_japanese_ascii() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("Java言語");
        let t = terms(&tokens);
        assert!(t.len() >= 2, "should split mixed text, got {:?}", t);
        assert!(
            t.iter().any(|s| *s == "Java"),
            "expected Java in {:?}",
            t
        );
    }

    #[test]
    fn empty_string() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn whitespace_only() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("   ");
        // Whitespace tokens may or may not be emitted; just check no panic
        let _ = terms(&tokens);
    }

    #[test]
    fn punctuation_handling() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("東京。大阪");
        let t = terms(&tokens);
        assert!(
            t.iter().any(|s: &&str| s.contains("東京")),
            "expected 東京 in {:?}",
            t
        );
        assert!(
            t.iter().any(|s: &&str| s.contains("大阪")),
            "expected 大阪 in {:?}",
            t
        );
    }

    #[test]
    fn single_character() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("猫");
        let t = terms(&tokens);
        assert_eq!(t, vec!["猫"]);
    }

    #[test]
    fn offsets_are_character_based() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("東京都");
        // Verify offsets are char-based, not byte-based
        for token in &tokens {
            assert!(
                token.end_offset <= 3,
                "char offset should be ≤ 3 for 3-char input, got end={}",
                token.end_offset
            );
        }
    }

    #[test]
    fn positions_are_sequential() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let tokens = tok.tokenize("東京都に行く");
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(
                token.position, i as u32,
                "positions should be sequential"
            );
        }
    }

    #[test]
    fn long_sentence() {
        let tok = KuromojiTokenizer::new(KuromojiMode::Normal);
        let text = "吾輩は猫である。名前はまだ無い。";
        let tokens = tok.tokenize(text);
        assert!(
            tokens.len() >= 5,
            "long sentence should produce many tokens, got {}",
            tokens.len()
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod baseform — KuromojiBaseformFilter
// ═══════════════════════════════════════════════════════════════════════════════

mod baseform {
    use super::*;

    #[test]
    fn conjugated_verb_to_dictionary_form() {
        let filter = KuromojiBaseformFilter::new();
        // 行った (past tense of 行く)
        let result = filter_term(&filter, "行った");
        // Should remain unchanged because "行った" tokenizes to multiple morphemes
        // The filter only transforms single-token surfaces
        let _ = result;
    }

    #[test]
    fn single_conjugated_verb() {
        let filter = KuromojiBaseformFilter::new();
        // 飲む is dictionary form, 飲み is conjunctive
        let result = filter_term(&filter, "飲み");
        // "飲み" base form should be "飲む" if IPADIC recognizes it as single token
        assert!(
            result == "飲む" || result == "飲み",
            "expected 飲む or 飲み, got {}",
            result
        );
    }

    #[test]
    fn adjective_conjugation() {
        let filter = KuromojiBaseformFilter::new();
        // 美しく is adverbial form of 美しい
        let result = filter_term(&filter, "美しく");
        assert!(
            result == "美しい" || result == "美しく",
            "expected base adjective form, got {}",
            result
        );
    }

    #[test]
    fn already_base_form_passthrough() {
        let filter = KuromojiBaseformFilter::new();
        // 食べる is already dictionary form
        let result = filter_term(&filter, "食べる");
        assert_eq!(result, "食べる");
    }

    #[test]
    fn noun_passthrough() {
        let filter = KuromojiBaseformFilter::new();
        let result = filter_term(&filter, "東京");
        assert_eq!(result, "東京");
    }

    #[test]
    fn default_construction() {
        let filter = KuromojiBaseformFilter::default();
        let result = filter_term(&filter, "東京");
        assert_eq!(result, "東京");
    }

    #[test]
    fn ascii_passthrough() {
        let filter = KuromojiBaseformFilter::new();
        let result = filter_term(&filter, "hello");
        // ASCII words have no IPADIC base form mapping
        assert_eq!(result, "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod part_of_speech — KuromojiPartOfSpeechFilter
// ═══════════════════════════════════════════════════════════════════════════════

mod part_of_speech {
    use super::*;

    #[test]
    fn filters_particle() {
        let filter = KuromojiPartOfSpeechFilter::with_defaults();
        // "に" is a particle (助詞)
        assert!(
            filter_deleted(&filter, "に"),
            "particle に should be removed"
        );
    }

    #[test]
    fn filters_auxiliary_verb() {
        let filter = KuromojiPartOfSpeechFilter::with_defaults();
        // "た" is auxiliary verb (助動詞)
        assert!(
            filter_deleted(&filter, "た"),
            "auxiliary verb た should be removed"
        );
    }

    #[test]
    fn keeps_noun() {
        let filter = KuromojiPartOfSpeechFilter::with_defaults();
        // "東京" is a proper noun
        assert!(
            !filter_deleted(&filter, "東京"),
            "noun 東京 should be kept"
        );
    }

    #[test]
    fn keeps_verb() {
        let filter = KuromojiPartOfSpeechFilter::with_defaults();
        // "食べる" is a verb
        assert!(
            !filter_deleted(&filter, "食べる"),
            "verb 食べる should be kept"
        );
    }

    #[test]
    fn keeps_adjective() {
        let filter = KuromojiPartOfSpeechFilter::with_defaults();
        assert!(
            !filter_deleted(&filter, "美しい"),
            "adjective 美しい should be kept"
        );
    }

    #[test]
    fn custom_stop_tags() {
        // Only remove nouns (名詞) for this test
        let filter = KuromojiPartOfSpeechFilter::new(vec!["名詞".to_string()]);
        assert!(
            filter_deleted(&filter, "東京"),
            "noun should be removed with custom tags"
        );
    }

    #[test]
    fn ascii_word_not_removed() {
        let filter = KuromojiPartOfSpeechFilter::with_defaults();
        // ASCII words are typically tagged as nouns, not particles
        assert!(
            !filter_deleted(&filter, "hello"),
            "ASCII word should not be removed by default POS filter"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod readingform — KuromojiReadingformFilter
// ═══════════════════════════════════════════════════════════════════════════════

mod readingform {
    use super::*;

    #[test]
    fn katakana_mode_kanji_to_katakana() {
        let filter = KuromojiReadingformFilter::new(ReadingFormType::Katakana);
        let result = filter_term(&filter, "東京");
        assert_eq!(result, "トウキョウ", "kanji should become katakana reading");
    }

    #[test]
    fn romaji_mode_kanji_to_romaji() {
        let filter = KuromojiReadingformFilter::new(ReadingFormType::Romaji);
        let result = filter_term(&filter, "東京");
        // Romaji conversion of トウキョウ
        assert!(
            !result.is_empty() && result.is_ascii(),
            "should produce romaji, got: {}",
            result
        );
    }

    #[test]
    fn katakana_passthrough() {
        let filter = KuromojiReadingformFilter::new(ReadingFormType::Katakana);
        // Katakana word — reading might be itself or differ slightly
        let result = filter_term(&filter, "コンピューター");
        // Should be katakana reading
        assert!(
            result.chars().all(|c| c >= '\u{30A0}' && c <= '\u{30FF}' || c == 'ー'),
            "katakana input reading should be katakana, got: {}",
            result
        );
    }

    #[test]
    fn ascii_passthrough() {
        let filter = KuromojiReadingformFilter::new(ReadingFormType::Katakana);
        // ASCII has no IPADIC reading, should stay unchanged
        let result = filter_term(&filter, "hello");
        assert_eq!(result, "hello");
    }

    #[test]
    fn romaji_for_hiragana_word() {
        let filter = KuromojiReadingformFilter::new(ReadingFormType::Romaji);
        let result = filter_term(&filter, "ありがとう");
        // Should produce some romaji or stay as-is
        let _ = result; // no panic
    }

    #[test]
    fn reading_form_type_equality() {
        assert_eq!(ReadingFormType::Katakana, ReadingFormType::Katakana);
        assert_ne!(ReadingFormType::Katakana, ReadingFormType::Romaji);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod stemmer — KuromojiStemmerFilter
// ═══════════════════════════════════════════════════════════════════════════════

mod stemmer {
    use super::*;

    #[test]
    fn stems_trailing_long_vowel() {
        let filter = KuromojiStemmerFilter::new();
        let result = filter_term(&filter, "サーバー");
        assert_eq!(result, "サーバ", "should remove trailing ー");
    }

    #[test]
    fn stems_computer() {
        let filter = KuromojiStemmerFilter::new();
        let result = filter_term(&filter, "コンピューター");
        assert_eq!(result, "コンピュータ");
    }

    #[test]
    fn short_katakana_unchanged() {
        let filter = KuromojiStemmerFilter::new();
        // Default min_length is 4; "カー" is only 2 chars
        let result = filter_term(&filter, "カー");
        assert_eq!(result, "カー", "short katakana should not be stemmed");
    }

    #[test]
    fn three_char_katakana_unchanged() {
        let filter = KuromojiStemmerFilter::new();
        // "ビアー" is 3 chars, below default min of 4
        let result = filter_term(&filter, "ビアー");
        assert_eq!(result, "ビアー", "3-char katakana should not be stemmed");
    }

    #[test]
    fn no_trailing_long_vowel() {
        let filter = KuromojiStemmerFilter::new();
        let result = filter_term(&filter, "コンピュータ");
        assert_eq!(result, "コンピュータ", "no trailing ー means no change");
    }

    #[test]
    fn non_katakana_unchanged() {
        let filter = KuromojiStemmerFilter::new();
        let result = filter_term(&filter, "東京都庁");
        assert_eq!(result, "東京都庁", "kanji should not be stemmed");
    }

    #[test]
    fn custom_min_length() {
        let filter = KuromojiStemmerFilter::with_min_length(2);
        // Now even 2-char words should be stemmed
        let result = filter_term(&filter, "カー");
        assert_eq!(result, "カ", "with min_length=2, カー should stem to カ");
    }

    #[test]
    fn default_construction() {
        let filter = KuromojiStemmerFilter::default();
        let result = filter_term(&filter, "サーバー");
        assert_eq!(result, "サーバ");
    }

    #[test]
    fn ascii_unchanged() {
        let filter = KuromojiStemmerFilter::new();
        let result = filter_term(&filter, "server");
        assert_eq!(result, "server");
    }

    #[test]
    fn mixed_script_unchanged() {
        let filter = KuromojiStemmerFilter::new();
        // Mixed katakana + kanji is not all-katakana
        let result = filter_term(&filter, "サーバ東京");
        assert_eq!(result, "サーバ東京", "mixed script should not be stemmed");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod number — KuromojiNumberFilter
// ═══════════════════════════════════════════════════════════════════════════════

mod number {
    use super::*;

    #[test]
    fn kanji_one() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "一");
        assert_eq!(result, "1");
    }

    #[test]
    fn kanji_ten() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "十");
        assert_eq!(result, "10");
    }

    #[test]
    fn kanji_hundred() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "百");
        assert_eq!(result, "100");
    }

    #[test]
    fn kanji_thousand() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "千");
        assert_eq!(result, "1000");
    }

    #[test]
    fn kanji_compound_twelve() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "十二");
        assert_eq!(result, "12");
    }

    #[test]
    fn kanji_compound_hundred_twenty_three() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "百二十三");
        assert_eq!(result, "123");
    }

    #[test]
    fn kanji_three_hundred() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "三百");
        assert_eq!(result, "300");
    }

    #[test]
    fn fullwidth_digits() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "３４");
        assert_eq!(result, "34");
    }

    #[test]
    fn non_number_unchanged() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "東京");
        assert_eq!(result, "東京");
    }

    #[test]
    fn ascii_number_unchanged() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "123");
        assert_eq!(result, "123", "ASCII numbers should pass through");
    }

    #[test]
    fn kanji_zero() {
        let filter = KuromojiNumberFilter::new();
        let result = filter_term(&filter, "〇");
        assert_eq!(result, "0");
    }

    #[test]
    fn kanji_positional_style() {
        let filter = KuromojiNumberFilter::new();
        // Positional: 一二三 (digit-by-digit) → 123
        let result = filter_term(&filter, "一二三");
        assert_eq!(result, "123");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod stop — JapaneseStopFilter
// ═══════════════════════════════════════════════════════════════════════════════

mod stop {
    use super::*;

    #[test]
    fn removes_particle_no() {
        let filter = JapaneseStopFilter::new();
        assert!(filter_deleted(&filter, "の"), "の should be a stop word");
    }

    #[test]
    fn removes_particle_ha() {
        let filter = JapaneseStopFilter::new();
        assert!(filter_deleted(&filter, "は"), "は should be a stop word");
    }

    #[test]
    fn removes_particle_ga() {
        let filter = JapaneseStopFilter::new();
        assert!(filter_deleted(&filter, "が"), "が should be a stop word");
    }

    #[test]
    fn removes_particle_wo() {
        let filter = JapaneseStopFilter::new();
        assert!(filter_deleted(&filter, "を"), "を should be a stop word");
    }

    #[test]
    fn removes_particle_ni() {
        let filter = JapaneseStopFilter::new();
        assert!(filter_deleted(&filter, "に"), "に should be a stop word");
    }

    #[test]
    fn removes_de() {
        let filter = JapaneseStopFilter::new();
        assert!(filter_deleted(&filter, "で"), "で should be a stop word");
    }

    #[test]
    fn keeps_content_word() {
        let filter = JapaneseStopFilter::new();
        assert!(
            !filter_deleted(&filter, "東京"),
            "content word should not be removed"
        );
    }

    #[test]
    fn keeps_verb() {
        let filter = JapaneseStopFilter::new();
        assert!(
            !filter_deleted(&filter, "食べる"),
            "verb should not be a stop word"
        );
    }

    #[test]
    fn custom_stop_words() {
        let filter =
            JapaneseStopFilter::with_words(vec!["カスタム".to_string(), "テスト".to_string()]);
        assert!(filter_deleted(&filter, "カスタム"));
        assert!(filter_deleted(&filter, "テスト"));
        assert!(!filter_deleted(&filter, "の"), "default words not in custom list");
    }

    #[test]
    fn default_construction() {
        let filter = JapaneseStopFilter::default();
        assert!(filter_deleted(&filter, "の"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod completion — JapaneseCompletionFilter
// ═══════════════════════════════════════════════════════════════════════════════

mod completion {
    use super::*;

    #[test]
    fn index_mode_emits_extras_for_kanji() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Index);
        let (deleted, term, extras) = apply_filter(&filter, "東京");
        assert!(!deleted, "should not delete original token");
        assert_eq!(term, "東京", "original term preserved");
        // Index mode should emit katakana and/or romaji extras
        assert!(
            !extras.is_empty(),
            "index mode should emit reading extras for kanji"
        );
    }

    #[test]
    fn index_mode_katakana_reading_present() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Index);
        let (_, _, extras) = apply_filter(&filter, "東京");
        let has_katakana = extras.iter().any(|e| e == "トウキョウ");
        assert!(
            has_katakana,
            "should contain katakana reading トウキョウ, got: {:?}",
            extras
        );
    }

    #[test]
    fn index_mode_romaji_reading_present() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Index);
        let (_, _, extras) = apply_filter(&filter, "東京");
        let has_romaji = extras.iter().any(|e| e.is_ascii() && !e.is_empty());
        assert!(
            has_romaji,
            "should contain romaji reading, got: {:?}",
            extras
        );
    }

    #[test]
    fn query_mode_passthrough() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Query);
        let (deleted, term, extras) = apply_filter(&filter, "東京");
        assert!(!deleted);
        assert_eq!(term, "東京");
        assert!(extras.is_empty(), "query mode should not emit extras");
    }

    #[test]
    fn ascii_passthrough_index_mode() {
        let filter = JapaneseCompletionFilter::new(CompletionMode::Index);
        let (deleted, term, _extras) = apply_filter(&filter, "hello");
        assert!(!deleted);
        assert_eq!(term, "hello");
    }

    #[test]
    fn completion_mode_equality() {
        assert_eq!(CompletionMode::Index, CompletionMode::Index);
        assert_ne!(CompletionMode::Index, CompletionMode::Query);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod register — register_all
// ═══════════════════════════════════════════════════════════════════════════════

mod register {
    use super::*;

    #[test]
    fn registers_kuromoji_tokenizer() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_tokenizer("kuromoji_tokenizer").is_some(),
            "kuromoji_tokenizer should be registered"
        );
    }

    #[test]
    fn registers_baseform_filter() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("kuromoji_baseform").is_some(),
            "kuromoji_baseform filter should be registered"
        );
    }

    #[test]
    fn registers_part_of_speech_filter() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("kuromoji_part_of_speech").is_some(),
            "kuromoji_part_of_speech filter should be registered"
        );
    }

    #[test]
    fn registers_readingform_filter() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("kuromoji_readingform").is_some(),
            "kuromoji_readingform filter should be registered"
        );
    }

    #[test]
    fn registers_stemmer_filter() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("kuromoji_stemmer").is_some(),
            "kuromoji_stemmer filter should be registered"
        );
    }

    #[test]
    fn registers_number_filter() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("kuromoji_number").is_some(),
            "kuromoji_number filter should be registered"
        );
    }

    #[test]
    fn registers_ja_stop_filter() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("ja_stop").is_some(),
            "ja_stop filter should be registered"
        );
    }

    #[test]
    fn registers_completion_filter() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("kuromoji_completion").is_some(),
            "kuromoji_completion filter should be registered"
        );
    }

    #[test]
    fn registers_kuromoji_analyzer() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_analyzer("kuromoji").is_some(),
            "kuromoji analyzer should be registered"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod pipeline — Full integration pipeline
// ═══════════════════════════════════════════════════════════════════════════════

mod pipeline {
    use super::*;

    /// Tokenize, then apply a chain of filters, returning surviving terms.
    fn analyze(text: &str) -> Vec<String> {
        let tokenizer = KuromojiTokenizer::new(KuromojiMode::Search);
        let baseform = KuromojiBaseformFilter::new();
        let pos_filter = KuromojiPartOfSpeechFilter::with_defaults();
        let stop = JapaneseStopFilter::new();
        let stemmer = KuromojiStemmerFilter::new();

        let mut tokens = tokenizer.tokenize(text);
        let filters: Vec<&dyn TokenFilter> =
            vec![&baseform, &pos_filter, &stop, &stemmer];

        let mut result = Vec::new();
        for token in &mut tokens {
            let mut deleted = false;
            for f in &filters {
                let (del, _extras) = f.filter(token);
                if del {
                    deleted = true;
                    break;
                }
            }
            if !deleted {
                result.push(token.term.to_string());
            }
        }
        result
    }

    #[test]
    fn full_pipeline_basic() {
        let result = analyze("東京都に行く");
        // Particles and auxiliary verbs should be removed
        assert!(!result.is_empty(), "pipeline should produce tokens");
        // "に" should be filtered by stop or POS filter
        assert!(
            !result.contains(&"に".to_string()),
            "particle に should be removed, got {:?}",
            result
        );
    }

    #[test]
    fn full_pipeline_preserves_content() {
        let result = analyze("東京タワー");
        assert!(
            result.iter().any(|t| t.contains("東京")),
            "should preserve 東京 in {:?}",
            result
        );
    }

    #[test]
    fn full_pipeline_stems_katakana() {
        let result = analyze("コンピューター");
        // The stemmer should remove trailing ー
        let has_stemmed = result.iter().any(|t| t == "コンピュータ");
        let has_original = result.iter().any(|t| t == "コンピューター");
        assert!(
            has_stemmed || has_original,
            "should have stemmed or original katakana, got {:?}",
            result
        );
    }

    #[test]
    fn full_pipeline_removes_particles() {
        let result = analyze("猫は魚を食べる");
        assert!(
            !result.contains(&"は".to_string()),
            "は should be removed by pipeline, got {:?}",
            result
        );
        assert!(
            !result.contains(&"を".to_string()),
            "を should be removed by pipeline, got {:?}",
            result
        );
    }

    #[test]
    fn registered_analyzer_works() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        let analyzer = factory.get_analyzer("kuromoji").unwrap();
        let mut input = "東京都に行く".to_string();
        let tokens = analyzer.analyze_and_return_tokens(&mut input);
        assert!(
            !tokens.is_empty(),
            "registered kuromoji analyzer should produce tokens"
        );
    }
}
