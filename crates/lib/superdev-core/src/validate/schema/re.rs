//! re.rs — the grammar's patterns, compiled once.
//!
//! The grammar carries its patterns as data, so they cannot be `LazyLock`
//! statics the way this module's fixed regexes are. Compiling one per value
//! checked was most of the cost of a whole-set run: the condition pattern
//! alone recompiles for every `when` attribute in every unit. The set of
//! patterns is bounded by the grammar, so memoising them is bounded too.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

use regex::Regex;

/// Compiled patterns, keyed by their source. A pattern that does not compile
/// is remembered as `None`, so a bad one is not recompiled on every value
/// either.
static CACHE: LazyLock<RwLock<HashMap<String, Option<Arc<Regex>>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// `pattern` compiled, or `None` when it does not compile.
///
/// Callers treat `None` as "matches nothing": a pattern the grammar cannot
/// compile is reported where the grammar is read, not once per value.
#[must_use]
pub fn compile(pattern: &str) -> Option<Arc<Regex>> {
    if let Ok(cache) = CACHE.read()
        && let Some(hit) = cache.get(pattern)
    {
        return hit.clone();
    }
    let compiled = Regex::new(pattern).ok().map(Arc::new);
    if let Ok(mut cache) = CACHE.write() {
        cache.insert(pattern.to_string(), compiled.clone());
    }
    compiled
}

/// Whether `value` matches `pattern`. A pattern that does not compile matches
/// nothing, which is what `Regex::new(p).is_ok_and(…)` did at every call site
/// this replaced.
#[must_use]
pub fn matches(pattern: &str, value: &str) -> bool {
    compile(pattern).is_some_and(|re| re.is_match(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pattern_compiles_once_and_answers_the_same_every_time() {
        assert!(matches(r"^a+$", "aaa"));
        assert!(matches(r"^a+$", "a"));
        assert!(!matches(r"^a+$", "b"));
        assert!(Arc::ptr_eq(
            &compile(r"^a+$").unwrap(),
            &compile(r"^a+$").unwrap()
        ));
    }

    #[test]
    fn a_pattern_that_does_not_compile_matches_nothing() {
        assert!(compile("(unclosed").is_none());
        assert!(!matches("(unclosed", "anything"));
    }
}
