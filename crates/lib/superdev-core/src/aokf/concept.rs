//! concept.rs — one AOKF concept file: frontmatter plus body sections.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
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

/// A retrievable slice of a concept: the root section (frontmatter, plus any
/// body text before the first heading), or one heading's body.
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
    /// unknown keys, `stamped` fields, wrongly typed values. Always a mapping.
    pub raw: Value,
    /// Everything after the frontmatter, verbatim. The validator reads links
    /// and footnotes from it, so it must carry no frontmatter-derived text.
    pub body: String,
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
/// `sections[0]` is the root section; the rest are one per markdown heading,
/// in document order.
///
/// # Errors
///
/// Returns [`ParseError`] when the file has no frontmatter block, when that
/// block is not valid YAML, or when it is not a mapping.
pub fn parse_concept(path: &str, text: &str) -> Result<Concept, ParseError> {
    let error = |message: String| ParseError {
        path: path.to_string(),
        message,
    };
    let (yaml, fm_end_line) = split_frontmatter(text).ok_or_else(|| {
        error("no frontmatter: expected a `---` line, then a closing `---`".into())
    })?;
    let raw: Value = serde_yaml_ng::from_str(&yaml).map_err(|e| error(e.to_string()))?;
    if !raw.is_mapping() {
        return Err(error("frontmatter is not a mapping".into()));
    }

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
        sections: split_sections(
            text,
            fm_end_line,
            &frontmatter_text(title.as_deref(), description.as_deref(), &tags),
        ),
        body: body_after(text, fm_end_line).to_string(),
        title,
        description,
        tags,
        raw,
    })
}

/// Everything after the frontmatter's closing `---`.
fn body_after(text: &str, fm_end_line: usize) -> &str {
    let starts = line_starts(text);
    &text[starts.get(fm_end_line).copied().unwrap_or(text.len())..]
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
fn frontmatter_text(title: Option<&str>, description: Option<&str>, tags: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.extend(title.map(str::to_string));
    parts.extend(description.map(str::to_string));
    if !tags.is_empty() {
        parts.push(tags.join(", "));
    }
    parts.join("\n")
}

/// Split the document into the root section plus one section per heading.
///
/// Each section runs from its heading line to the line before the next
/// heading, or to the end of the file. The root section covers the
/// frontmatter and any body text preceding the first heading.
fn split_sections(text: &str, fm_end_line: usize, frontmatter_text: &str) -> Vec<Section> {
    let starts = line_starts(text);
    let last_line = starts.len();
    let body_start = starts.get(fm_end_line).copied().unwrap_or(text.len());
    let headings = headings(&text[body_start..], body_start, &starts);

    let root_end = headings.first().map_or(last_line, |(line, _)| line - 1);
    let body = slice_lines(text, &starts, fm_end_line + 1, root_end);
    let mut sections = vec![Section {
        heading_path: Vec::new(),
        start_line: 1,
        end_line: root_end,
        text: join_non_empty(frontmatter_text, body),
    }];

    for (index, (start_line, heading_path)) in headings.iter().enumerate() {
        let end_line = headings
            .get(index + 1)
            .map_or(last_line, |(next, _)| next - 1);
        sections.push(Section {
            heading_path: heading_path.clone(),
            start_line: *start_line,
            end_line,
            text: slice_lines(text, &starts, *start_line, end_line).to_string(),
        });
    }
    sections
}

/// Every heading in `body`, as its 1-based line in the whole document and its
/// full heading path. `offset` is where `body` starts in that document.
fn headings(body: &str, offset: usize, starts: &[usize]) -> Vec<(usize, Vec<String>)> {
    let mut found = Vec::new();
    let mut stack: Vec<(HeadingLevel, String)> = Vec::new();
    // Heading text arrives as separate events between Start and End, so it is
    // accumulated rather than read from one event.
    let mut open: Option<(usize, HeadingLevel, String)> = None;
    for (event, range) in Parser::new_ext(body, Options::empty()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                open = Some((line_of(starts, offset + range.start), level, String::new()));
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, _, title)) = open.as_mut() {
                    title.push_str(&t);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((line, level, title)) = open.take() {
                    // A heading closes every open heading at its level or
                    // deeper, so an h1 resets the path and an h2 nests.
                    while stack.last().is_some_and(|(open, _)| *open >= level) {
                        stack.pop();
                    }
                    stack.push((level, title));
                    found.push((line, stack.iter().map(|(_, t)| t.clone()).collect()));
                }
            }
            _ => {}
        }
    }
    found
}

/// Byte offset of each line, so a parser offset can be turned into a line
/// number. A trailing newline does not start a further line.
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(
        text.bytes()
            .enumerate()
            .filter(|&(i, b)| b == b'\n' && i + 1 < text.len())
            .map(|(i, _)| i + 1),
    );
    starts
}

/// The 1-based line containing `offset`.
fn line_of(starts: &[usize], offset: usize) -> usize {
    starts.partition_point(|&start| start <= offset)
}

/// Lines `first..=last` of `text`, trailing whitespace trimmed; empty when
/// the range is.
fn slice_lines<'a>(text: &'a str, starts: &[usize], first: usize, last: usize) -> &'a str {
    if first > last || first > starts.len() {
        return "";
    }
    let end = starts.get(last).copied().unwrap_or(text.len());
    text[starts[first - 1]..end].trim_end()
}

/// Join two blocks with a blank-line-free newline, dropping empty ones.
fn join_non_empty(first: &str, second: &str) -> String {
    match (first.is_empty(), second.is_empty()) {
        (_, true) => first.to_string(),
        (true, _) => second.to_string(),
        _ => format!("{first}\n{second}"),
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
    fn root_section_ends_before_the_first_heading() {
        let c = parse_concept("planner.md", DOC).unwrap();
        // Frontmatter closes on line 9, `# Role` is line 11.
        assert_eq!(c.sections[0].end_line, 10);
        assert_eq!(c.sections.len(), 2);
    }

    #[test]
    fn a_body_without_headings_is_all_root_section() {
        let c = parse_concept("x.md", "---\ntype: T\n---\nbody\nmore\n").unwrap();
        assert_eq!(c.sections.len(), 1);
        assert_eq!(c.sections[0].end_line, 5);
        assert_eq!(c.sections[0].text, "body\nmore");
    }

    #[test]
    fn a_deeper_heading_after_a_shallower_sibling_reopens_the_path() {
        let doc = "---\ntype: T\n---\n### Deep\n# Top\n### Under\n";
        let c = parse_concept("x.md", doc).unwrap();
        let paths: Vec<&[String]> = c
            .sections
            .iter()
            .map(|s| s.heading_path.as_slice())
            .collect();
        assert_eq!(paths[1], ["Deep".to_string()]);
        assert_eq!(paths[2], ["Top".to_string()]);
        assert_eq!(paths[3], ["Top".to_string(), "Under".to_string()]);
        assert_eq!(c.sections[3].end_line, 6);
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
    fn splits_body_at_headings_with_line_ranges() {
        let doc = "---\ntype: T\n---\nintro ignored? no — pre-heading body joins the root section\n\n# One\n\nalpha\n\n## Sub\n\nbeta\n\n# Two\n\ngamma\n";
        let c = parse_concept("x.md", doc).unwrap();
        let paths: Vec<Vec<String>> = c.sections.iter().map(|s| s.heading_path.clone()).collect();
        assert_eq!(paths[0], Vec::<String>::new());
        assert_eq!(paths[1], vec!["One"]);
        assert_eq!(paths[2], vec!["One", "Sub"]);
        assert_eq!(paths[3], vec!["Two"]);
        let one = &c.sections[1];
        assert!(one.text.contains("alpha"));
        assert!(!one.text.contains("beta"));
        // heading line itself is the section start
        assert_eq!(doc.lines().nth(one.start_line - 1).unwrap(), "# One");
    }

    #[test]
    fn headings_inside_code_fences_are_not_sections() {
        let doc = "---\ntype: T\n---\n\n# Real\n\n```\n# not a heading\n```\n";
        let c = parse_concept("x.md", doc).unwrap();
        assert_eq!(c.sections.len(), 2); // root + Real
    }

    #[test]
    fn frontmatter_that_is_not_a_mapping_is_a_parse_error() {
        for text in ["---\n- one\n---\nb\n", "---\n---\nb\n"] {
            let e = parse_concept("x.md", text).unwrap_err();
            assert_eq!(e.message, "frontmatter is not a mapping");
        }
    }

    #[test]
    fn the_body_is_kept_verbatim() {
        let c = parse_concept("x.md", "---\ntype: T\ntitle: T\n---\n# H\n\n[a](a.md)\n").unwrap();
        assert_eq!(c.body, "# H\n\n[a](a.md)\n");
    }

    #[test]
    fn invalid_yaml_reports_the_yaml_message() {
        let e = parse_concept("x.md", "---\ntype: [unclosed\n---\nb\n").unwrap_err();
        assert!(!e.message.is_empty());
    }
}
