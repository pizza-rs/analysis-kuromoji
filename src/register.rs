//! Register Kuromoji (Japanese) analysis components into [`AnalysisFactory`].

use alloc::boxed::Box;
use alloc::vec;

use pizza_engine::analysis::AnalysisFactory;
use pizza_engine::analysis::Analyzer;

use crate::{
    KuromojiBaseformFilter, KuromojiMode, KuromojiNumberFilter,
    KuromojiPartOfSpeechFilter, KuromojiReadingformFilter, KuromojiStemmerFilter,
    KuromojiTokenizer, JapaneseStopFilter, JapaneseCompletionFilter, CompletionMode,
    ReadingFormType,
};

/// Register Kuromoji tokenizers, token filters, and analyzers.
///
/// Matches Elasticsearch's analysis-kuromoji plugin registration:
/// - Tokenizer: `kuromoji_tokenizer` (search mode by default)
/// - Filters: `kuromoji_baseform`, `kuromoji_part_of_speech`, `kuromoji_readingform`,
///   `kuromoji_stemmer`, `kuromoji_number`, `ja_stop`, `kuromoji_completion`
/// - Analyzer: `kuromoji` (JapaneseAnalyzer pipeline)
pub fn register_all(factory: &mut AnalysisFactory) {
    // Tokenizers
    factory.register_tokenizer("kuromoji_tokenizer", Box::new(KuromojiTokenizer::new(KuromojiMode::Search)));

    // Token filters
    factory.register_token_filter("kuromoji_baseform", Box::new(KuromojiBaseformFilter::new()));
    factory.register_token_filter("kuromoji_part_of_speech", Box::new(KuromojiPartOfSpeechFilter::with_defaults()));
    factory.register_token_filter("kuromoji_readingform", Box::new(KuromojiReadingformFilter::new(ReadingFormType::Katakana)));
    factory.register_token_filter("kuromoji_stemmer", Box::new(KuromojiStemmerFilter::new()));
    factory.register_token_filter("kuromoji_number", Box::new(KuromojiNumberFilter::new()));
    factory.register_token_filter("ja_stop", Box::new(JapaneseStopFilter::new()));
    factory.register_token_filter("kuromoji_completion", Box::new(JapaneseCompletionFilter::new(CompletionMode::Index)));

    // Analyzer: kuromoji (matches Lucene JapaneseAnalyzer pipeline)
    // Pipeline: tokenizer(search) → baseform → part_of_speech → ja_stop → stemmer
    factory.register_analyzer(
        "kuromoji",
        Analyzer::new(
            vec![],
            Box::new(KuromojiTokenizer::new(KuromojiMode::Search)),
            vec![
                Box::new(KuromojiBaseformFilter::new()),
                Box::new(KuromojiPartOfSpeechFilter::with_defaults()),
                Box::new(JapaneseStopFilter::new()),
                Box::new(KuromojiStemmerFilter::new()),
            ],
        ),
    );
}
