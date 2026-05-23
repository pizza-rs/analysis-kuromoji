<div align="center">

# 🇯🇵 pizza-analysis-kuromoji

**Japanese morphological analysis plugin for [INFINI Pizza](https://pizza.rs)**

[![Crate](https://img.shields.io/badge/crate-pizza--analysis--kuromoji-blue)](https://github.com/pizza-rs/analysis-kuromoji)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

</div>

---

## Overview

A full-featured Japanese morphological analyzer built on [lindera](https://github.com/lindera/lindera)
with IPADIC dictionary. Provides tokenization with part-of-speech analysis, base form
reduction, reading form conversion, and katakana stemming — matching the Elasticsearch
`analysis-kuromoji` plugin feature set.

## Components

| Type | Name | Description |
|:-----|:-----|:------------|
| Tokenizer | `kuromoji_tokenizer` | Morphological tokenizer (Normal / Search / Extended modes) |
| TokenFilter | `kuromoji_baseform` | Replace conjugated forms with dictionary base form |
| TokenFilter | `kuromoji_part_of_speech` | Remove tokens by POS tag (stop-by-grammar) |
| TokenFilter | `kuromoji_readingform` | Replace with katakana or romaji reading |
| TokenFilter | `kuromoji_stemmer` | Stem katakana long vowels (コンピューター → コンピュータ) |
| TokenFilter | `kuromoji_number` | Normalize Japanese Kanji numbers to Arabic |
| TokenFilter | `ja_stop` | Japanese stop words |
| Analyzer | `kuromoji` | Full pipeline: kuromoji_tokenizer → baseform → POS → stop |

### Tokenizer Modes

| Mode | Behavior | Use Case |
|:-----|:---------|:---------|
| `Normal` | Standard morphological segmentation | General purpose |
| `Search` | Decompound + emit both original & parts | Search indexing |
| `Extended` | Like Search, but unknown chars → unigrams | Maximum recall |

## Example

```rust
use pizza_engine::analysis::Tokenizer;
use pizza_analysis_kuromoji::{KuromojiTokenizer, KuromojiMode};

let tk = KuromojiTokenizer::new(KuromojiMode::Search);
let tokens = tk.tokenize("関西国際空港");
// Search mode decompounds: ["関西", "国際", "空港", "関西国際空港"]
```

## Installation

```toml
[dependencies]
pizza-analysis-kuromoji = "0.1"
```

Or via `pizza-analysis-all`:

```toml
[dependencies]
pizza-analysis-all = { version = "0.1", features = ["kuromoji"] }
```

## License

MIT

---

<div align="center">
<sub>Part of the <a href="https://pizza.rs">INFINI Pizza</a> ecosystem</sub>
</div>
