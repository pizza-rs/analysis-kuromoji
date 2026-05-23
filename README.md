# pizza-analysis-kuromoji

Japanese morphological analysis for the [Pizza](https://pizza.rs) search engine. Wraps the [Lindera](https://github.com/lindera/lindera) library with the IPADIC dictionary for tokenization, reading form extraction, and stemming.

## Components

| Name | Type | Description |
|------|------|-------------|
| `kuromoji_tokenizer` | Tokenizer | Japanese morphological tokenizer with search mode |
| `kuromoji_baseform` | Token Filter | Reduce conjugated forms to dictionary form |
| `kuromoji_part_of_speech` | Token Filter | Remove tokens by part-of-speech (POS) tags |
| `kuromoji_readingform` | Token Filter | Output katakana reading of tokens |
| `kuromoji_stemmer` | Token Filter | Stem trailing long vowels (ー) in katakana |
| `kuromoji_number` | Token Filter | Normalize kanji numerals to Arabic digits |
| `ja_stop` | Token Filter | Remove Japanese stop words |
| `kuromoji` | Analyzer | Full Japanese pipeline |

## Usage

### Full Analyzer

The `kuromoji` analyzer combines all components into a standard Japanese analysis pipeline:

```json
{
  "analyzer": {
    "type": "kuromoji"
  }
}
```

Pipeline: `kuromoji_tokenizer` → `kuromoji_baseform` → `kuromoji_part_of_speech` → `ja_stop` → `kuromoji_stemmer`

### Tokenizer Modes

| Mode | Description |
|------|-------------|
| `normal` | Standard segmentation, no decomposition |
| `search` | Decomposes compound words for better recall (default) |
| `extended` | Like search but also segments unknown katakana words |

### Examples

**Input:** `関西国際空港`

| Mode | Output |
|------|--------|
| Normal | `関西国際空港` |
| Search | `関西`, `国際`, `空港`, `関西国際空港` |

**Input:** `寿司が食べたい`

| Component | Output |
|-----------|--------|
| Tokenizer | `寿司`, `が`, `食べ`, `たい` |
| + Baseform | `寿司`, `が`, `食べる`, `たい` |
| + POS filter | `寿司`, `食べる` |

### Custom Analyzer

```json
{
  "analyzer": {
    "type": "custom",
    "tokenizer": "kuromoji_tokenizer",
    "filter": ["kuromoji_baseform", "kuromoji_part_of_speech", "ja_stop", "kuromoji_stemmer"]
  }
}
```

## Stop Words

Default Japanese stop words include common particles and auxiliary verbs:
`の`, `に`, `は`, `を`, `た`, `が`, `で`, `て`, `と`, `し`, `れ`, `さ`, `ある`, `いる`, `する`, `から`, `こと`, `として`, `できる`, `これ`, `ない`, `なる`, `ため`, `その`, `もの`, `という`, `よう`, ...

## Data Sources

- **Dictionary**: IPADIC (IPA Dictionary for MeCab) — the same dictionary used by Apache Lucene's Kuromoji
- **Embedded via**: `lindera` 3.0 with `embed-ipadic` feature

## Features

- `embed-dict` (default) — Embeds the IPADIC dictionary at compile time (~14MB binary)

## License

Apache-2.0
