//! Exact addresses are evidence of identity, not a stronger body-text score.
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

/// The retrieval evidence behind a section, not a relevance confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchKind {
    /// An exact concept ID or unambiguous numbered shorthand.
    Identifier,
    /// An exact knowledge-relative or physical path.
    Path,
    /// Found by lexical retrieval only.
    Lexical,
    /// Found by semantic retrieval only.
    Semantic,
    /// Found by both lexical and semantic retrieval.
    LexicalAndSemantic,
}

impl MatchKind {
    /// The concise label printed beside a result's section locator.
    pub fn label(self) -> &'static str {
        match self {
            Self::Identifier => "identifier",
            Self::Path => "path",
            Self::Lexical => "lexical",
            Self::Semantic => "semantic",
            Self::LexicalAndSemantic => "lexical + semantic",
        }
    }
}

/// Return matching logical paths, in stable path order. Single-word IDs are
/// direct only when they are the whole query: otherwise ordinary words such
/// as "testing" would outrank a more relevant semantic result in every phrase.
pub(super) fn matches<'a>(
    query: &str,
    identities: &'a BTreeMap<String, Option<String>>,
    root: &Path,
) -> BTreeMap<&'a str, MatchKind> {
    static NUMBERED: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-z]+(?:-[a-z]+)*-\d{3}(?:-|$)").unwrap());
    static ATOMS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"`[^`]*`|"[^"]*"|'[^']*'|[^\s`"']+"#).unwrap());
    // The whole query can be a path, including spaces. Otherwise quoted paths
    // remain one atom; do not mistake a basename fragment for another file.
    let whole = unquote(query);
    let whole_paths: BTreeMap<_, _> = identities
        .keys()
        .filter(|path| names_path(query, path, root))
        .map(|path| (path.as_str(), MatchKind::Path))
        .collect();
    if !whole_paths.is_empty() {
        return whole_paths;
    }
    let atoms: Vec<&str> = ATOMS.find_iter(query).map(|atom| atom.as_str()).collect();
    let tokens: Vec<&str> = atoms
        .iter()
        .filter(|atom| !atom.contains('/') && !atom.contains(".md"))
        .map(|atom| unquote(atom))
        .flat_map(|atom| atom.split(|c: char| !(c.is_alphanumeric() || "-_.".contains(c))))
        .filter(|word| !word.is_empty())
        .collect();
    let shorthand = |id: &str| {
        NUMBERED
            .find(id)
            .map(|found| found.as_str().trim_end_matches('-').to_string())
    };
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for id in identities.values().flatten() {
        if let Some(short) = shorthand(id) {
            *counts.entry(short).or_default() += 1;
        }
    }
    let mut result = BTreeMap::new();
    for (path, id) in identities {
        let id_matches = id.as_ref().is_some_and(|id| {
            (tokens.contains(&id.as_str()) && (id.contains('-') || whole == id))
                || shorthand(id)
                    .is_some_and(|short| counts[&short] == 1 && tokens.contains(&short.as_str()))
        });
        if id_matches {
            result.insert(path.as_str(), MatchKind::Identifier);
        } else if atoms.iter().any(|atom| names_path(atom, path, root)) {
            result.insert(path.as_str(), MatchKind::Path);
        }
    }
    result
}

fn unquote(text: &str) -> &str {
    let mut text = text.trim().trim_end_matches([',', ';', '.', '?', '!']);
    loop {
        let pair = [('`', '`'), ('"', '"'), ('\'', '\''), ('(', ')'), ('[', ']')]
            .into_iter()
            .find(|(start, end)| {
                text.len() >= 2 && text.starts_with(*start) && text.ends_with(*end)
            });
        if pair.is_none() {
            return text;
        }
        text = text[1..text.len() - 1].trim();
    }
}

fn names_path(word: &str, path: &str, root: &Path) -> bool {
    [word.trim(), unquote(word)].into_iter().any(|word| {
        let word = Path::new(word);
        let word = word.strip_prefix(".").unwrap_or(word);
        // At least the whole bundle-relative path is required. Arbitrary
        // basenames of nested documents are not unique physical addresses.
        word.ends_with(Path::new(path)) && root.join(path).ends_with(word)
    })
}
