//! IPADIC dictionary resolution for the kuromoji plugin.
//!
//! Resolution order (matching the engine-wide policy in
//! `pizza_engine::analysis::dict`):
//!
//! 1. An external lindera IPADIC dictionary directory at
//!    `<analysis dict dir>/kuromoji/ipadic` (when the analysis dict directory
//!    is configured and the entry exists).
//! 2. The embedded IPADIC dictionary — only compiled in when this crate is
//!    built with the `embed-dict` feature.
//!
//! Because the kuromoji analyzers are now registered lazily, this is only ever
//! invoked when a schema actually uses a kuromoji component, so an
//! `embed-dict`-free build with no external dictionary stays valid until then.

use lindera::dictionary::load_dictionary;
use lindera::dictionary::Dictionary;

/// Load the IPADIC dictionary using the external-first, embedded-fallback policy.
pub(crate) fn load_ipadic() -> Dictionary {
    #[cfg(feature = "std")]
    {
        if let Some(path) = pizza_engine::analysis::dict::resolve("kuromoji", "ipadic") {
            let p = path.to_str().expect("non-UTF-8 kuromoji dictionary path");
            return load_dictionary(p)
                .unwrap_or_else(|e| panic!("failed to load IPADIC dictionary from {p}: {e}"));
        }
    }

    #[cfg(feature = "embed-dict")]
    {
        // lindera-ipadic's embedded dictionary (the previous code recursed
        // into load_ipadic itself, overflowing the stack on first use)
        lindera_ipadic::embedded::load()
            .unwrap_or_else(|e| panic!("failed to load embedded IPADIC dictionary: {e}"))
    }

    #[cfg(not(feature = "embed-dict"))]
    {
        panic!(
            "kuromoji IPADIC dictionary not available: place a lindera IPADIC dictionary \
             directory at config/analysis/kuromoji/ipadic ('make copy-analysis-dicts'; the \
             analyzer sub-repo ships a cache under data/), or build pizza-analysis-kuromoji \
             with the 'embed-dict' feature"
        )
    }
}
