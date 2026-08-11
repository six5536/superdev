//! concept.rs — one AOKF concept file: frontmatter plus body sections.

use serde_yaml_ng::Value;

/// Publication state of a concept; an absent frontmatter `status` is
/// [`Status::Stable`], as the spec requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Work in progress.
    Draft,
    /// The default state.
    Stable,
    /// Kept for reference; superseded or retired.
    Deprecated,
}

/// One entry of the frontmatter `sources` list: a material the concept
/// derives from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Footnote key, local to the file.
    pub id: Option<String>,
    /// Repo-root-relative path or URL.
    pub resource: Option<String>,
    /// Display label.
    pub title: Option<String>,
}

/// One entry of the frontmatter `links` list: a typed edge to another
/// concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Relationship type, e.g. `depends-on`.
    pub rel: Option<String>,
    /// Target concept `id` or `/`-rooted path.
    pub to: Option<String>,
    /// One-line explanation of this edge.
    pub note: Option<String>,
}

/// A retrievable slice of a concept: the frontmatter block, or (from Task 2
/// on) one heading's body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// Headings from the document root down to this section; empty for the
    /// root section.
    pub heading_path: Vec<String>,
    /// First line of the section, 1-based and inclusive.
    pub start_line: usize,
    /// Last line of the section, 1-based and inclusive.
    pub end_line: usize,
    /// The section's searchable text.
    pub text: String,
}

/// A parsed concept file.
#[derive(Debug, Clone)]
pub struct Concept {
    /// Bundle-relative path, forward slashes.
    pub path: String,
    /// Frontmatter `type`; empty when absent, which the validator flags.
    pub kind: String,
    /// Stable identity slug.
    pub id: Option<String>,
    /// Display name.
    pub title: Option<String>,
    /// One-line summary.
    pub description: Option<String>,
    /// Publication state.
    pub status: Status,
    /// Grouping labels.
    pub tags: Vec<String>,
    /// Repo path or URL of the thing described.
    pub resource: Option<String>,
    /// Materials the concept derives from.
    pub sources: Vec<Source>,
    /// Typed edges to other concepts.
    pub links: Vec<Link>,
    /// The whole frontmatter, unmodified, for checks the typed fields drop —
    /// unknown keys, `stamped` fields, wrongly typed values.
    pub raw: Value,
    /// Retrievable slices of the document.
    pub sections: Vec<Section>,
}

/// A file that could not be parsed at all. Malformed *content* is data for
/// the validator; only a missing or unreadable frontmatter block lands here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Bundle-relative path of the offending file.
    pub path: String,
    /// What went wrong.
    pub message: String,
}

/// Parse one concept file's full text.
///
/// Field-level problems never fail the parse: a wrongly typed field degrades
/// to empty or `None` and survives in [`Concept::raw`] for the validator.
///
/// The body is split at headings in a later task; `sections` currently holds
/// only the root section covering the frontmatter.
///
/// # Errors
///
/// Returns [`ParseError`] when the file has no frontmatter block, or when
/// that block is not valid YAML.
pub fn parse_concept(path: &str, text: &str) -> Result<Concept, ParseError> {
    let error = |message: String| ParseError {
        path: path.to_string(),
        message,
    };
    let (yaml, fm_end_line) = split_frontmatter(text).ok_or_else(|| {
        error("no frontmatter: expected a `---` line, then a closing `---`".into())
    })?;
    let raw: Value = serde_yaml_ng::from_str(&yaml).map_err(|e| error(e.to_string()))?;

    let title = string_field(&raw, "title");
    let description = string_field(&raw, "description");
    let tags: Vec<String> = sequence_field(&raw, "tags")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    Ok(Concept {
        path: path.to_string(),
        kind: string_field(&raw, "type").unwrap_or_default(),
        id: string_field(&raw, "id"),
        status: match raw["status"].as_str() {
            Some("draft") => Status::Draft,
            Some("deprecated") => Status::Deprecated,
            _ => Status::Stable,
        },
        resource: string_field(&raw, "resource"),
        sources: sequence_field(&raw, "sources")
            .iter()
            .map(|v| Source {
                id: string_field(v, "id"),
                resource: string_field(v, "resource"),
                title: string_field(v, "title"),
            })
            .collect(),
        links: sequence_field(&raw, "links")
            .iter()
            .map(|v| Link {
                rel: string_field(v, "rel"),
                to: string_field(v, "to"),
                note: string_field(v, "note"),
            })
            .collect(),
        sections: vec![root_section(
            title.as_deref(),
            description.as_deref(),
            &tags,
            fm_end_line,
        )],
        title,
        description,
        tags,
        raw,
    })
}

/// Split off the frontmatter block, returning its YAML text and the 1-based
/// line number of the closing `---`.
fn split_frontmatter(text: &str) -> Option<(String, usize)> {
    let mut lines = text.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut yaml = String::new();
    for (offset, line) in lines.enumerate() {
        if line == "---" {
            return Some((yaml, offset + 2));
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    None
}

/// The frontmatter's own searchable text, so a concept is findable by its
/// title, summary and tags even when no body section matches.
fn root_section(
    title: Option<&str>,
    description: Option<&str>,
    tags: &[String],
    end_line: usize,
) -> Section {
    let mut parts: Vec<String> = Vec::new();
    parts.extend(title.map(str::to_string));
    parts.extend(description.map(str::to_string));
    if !tags.is_empty() {
        parts.push(tags.join(", "));
    }
    Section {
        heading_path: Vec::new(),
        start_line: 1,
        end_line,
        text: parts.join("\n"),
    }
}

/// A string-valued key, or `None` when it is absent or another type.
fn string_field(value: &Value, key: &str) -> Option<String> {
    value[key].as_str().map(str::to_string)
}

/// A sequence-valued key, or an empty slice when it is absent or another
/// type.
fn sequence_field<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_sequence().map_or(&[], Vec::as_slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "---\ntype: Module\nid: planner\ndescription: Pure planning stage.\ntags: [core]\nlinks:\n  - rel: depends-on\n    to: config\n---\n\n# Role\n\nBody text.\n";

    #[test]
    fn parses_frontmatter_fields() {
        let c = parse_concept("planner.md", DOC).unwrap();
        assert_eq!(c.kind, "Module");
        assert_eq!(c.id.as_deref(), Some("planner"));
        assert_eq!(c.tags, vec!["core"]);
        assert_eq!(c.links.len(), 1);
        assert_eq!(c.links[0].rel.as_deref(), Some("depends-on"));
        assert!(matches!(c.status, Status::Stable));
    }

    #[test]
    fn root_section_carries_frontmatter_text_and_lines() {
        let c = parse_concept("planner.md", DOC).unwrap();
        let root = &c.sections[0];
        assert!(root.heading_path.is_empty());
        assert_eq!(root.start_line, 1);
        assert!(root.text.contains("Pure planning stage."));
    }

    #[test]
    fn missing_type_parses_with_empty_kind() {
        let c = parse_concept("x.md", "---\nid: x\n---\nbody\n").unwrap();
        assert_eq!(c.kind, "");
    }

    #[test]
    fn no_frontmatter_is_a_parse_error() {
        assert!(parse_concept("x.md", "just markdown\n").is_err());
    }

    #[test]
    fn wrong_typed_fields_degrade_but_raw_survives() {
        let c = parse_concept("x.md", "---\ntype: T\nlinks: nope\n---\nb\n").unwrap();
        assert!(c.links.is_empty());
        assert_eq!(c.raw["links"].as_str(), Some("nope"));
    }

    #[test]
    fn frontmatter_ends_at_the_closing_fence() {
        let c = parse_concept("planner.md", DOC).unwrap();
        assert_eq!(c.sections[0].end_line, 9);
        assert_eq!(c.sections.len(), 1);
    }

    #[test]
    fn title_resource_and_sources_are_read() {
        let doc = "---\ntype: Module\ntitle: Planner\nresource: /src/planner.rs\nsources:\n  - id: src\n    resource: /src/planner.rs\n    title: Planner source\n---\nb\n";
        let c = parse_concept("planner.md", doc).unwrap();
        assert_eq!(c.title.as_deref(), Some("Planner"));
        assert_eq!(c.resource.as_deref(), Some("/src/planner.rs"));
        assert_eq!(
            c.sources,
            vec![Source {
                id: Some("src".into()),
                resource: Some("/src/planner.rs".into()),
                title: Some("Planner source".into()),
            }]
        );
        assert!(c.sections[0].text.starts_with("Planner"));
    }

    #[test]
    fn status_words_map_and_anything_else_is_stable() {
        let parse_status = |s: &str| {
            parse_concept("x.md", &format!("---\ntype: T\nstatus: {s}\n---\nb\n"))
                .unwrap()
                .status
        };
        assert_eq!(parse_status("draft"), Status::Draft);
        assert_eq!(parse_status("deprecated"), Status::Deprecated);
        assert_eq!(parse_status("nonsense"), Status::Stable);
    }

    #[test]
    fn non_map_link_and_tag_entries_degrade() {
        let c = parse_concept(
            "x.md",
            "---\ntype: T\ntags: [ok, [nested]]\nlinks:\n  - 3\n---\nb\n",
        )
        .unwrap();
        assert_eq!(c.tags, vec!["ok"]);
        assert_eq!(
            c.links,
            vec![Link {
                rel: None,
                to: None,
                note: None
            }]
        );
    }

    #[test]
    fn unterminated_frontmatter_is_a_parse_error() {
        let e = parse_concept("x.md", "---\ntype: T\nbody\n").unwrap_err();
        assert_eq!(e.path, "x.md");
        assert!(e.message.contains("frontmatter"));
    }

    #[test]
    fn invalid_yaml_reports_the_yaml_message() {
        let e = parse_concept("x.md", "---\ntype: [unclosed\n---\nb\n").unwrap_err();
        assert!(!e.message.is_empty());
    }
}
