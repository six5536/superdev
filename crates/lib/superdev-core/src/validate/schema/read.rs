//! read.rs — turning a governed file into the pieces the checks read.
//!
//! A behavioural port of the Node reference's readers. Two of them carry bugs
//! that were found and fixed once already, so the port keeps their shape
//! rather than reaching for something tidier:
//!
//! - [`prose_only`] blanks fenced blocks and inline code spans *in place*, so
//!   every later index still points where it did. A code span may wrap across
//!   a line, and the blanking must not stop at the newline — a `<name>` inside
//!   one is a CLI placeholder, not an element.
//! - [`unfenced`] keeps inline code spans, because a backticked tool name has
//!   to stay visible to the load-hoist check. Reading comparables from the
//!   code-stripped text instead produced a false duplication finding.

use std::sync::LazyLock;

use regex::Regex;

/// Collapse every run of whitespace to one space, and trim.
#[must_use]
pub fn norm(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// True for every line inside — or delimiting — a fenced code block.
#[must_use]
pub fn fence_map(lines: &[&str]) -> Vec<bool> {
    static OPEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(`{3,}|~{3,})").unwrap());
    let mut map = vec![false; lines.len()];
    // The marker that opened the block we are inside, if any.
    let mut fence: Option<String> = None;
    for (i, line) in lines.iter().enumerate() {
        let found = OPEN.captures(line).map(|c| c[1].to_string());
        match &fence {
            Some(open) => {
                map[i] = true;
                if let Some(mark) = &found
                    && mark.as_bytes()[0] == open.as_bytes()[0]
                    && mark.len() >= open.len()
                    && line.trim() == mark
                {
                    fence = None;
                }
            }
            None => {
                if let Some(mark) = found {
                    fence = Some(mark);
                    map[i] = true;
                }
            }
        }
    }
    map
}

/// A file's frontmatter, and where its body starts.
pub struct Split<'a> {
    /// The lines between the `---` delimiters.
    pub fm: Vec<&'a str>,
    /// Index of the first body line.
    pub body_start: usize,
}

/// Split leading `---` frontmatter from the body; `None` when there is none.
#[must_use]
pub fn split_frontmatter<'a>(lines: &[&'a str]) -> Option<Split<'a>> {
    if lines.first() != Some(&"---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    Some(Split {
        fm: lines[1..end].to_vec(),
        body_start: end + 1,
    })
}

/// One frontmatter key and whatever followed it.
#[derive(Debug, Clone)]
pub struct FmEntry {
    /// The key.
    pub key: String,
    /// Its one-based line within the frontmatter.
    pub line: usize,
    /// The value on the key's own line, unquoted.
    pub scalar: Option<String>,
    /// The indented lines under it, when there are any.
    pub block: Option<Vec<String>>,
    /// Whether every block line is a `- ` item.
    pub is_list: bool,
    /// Whether the scalar opened a `|` or `>` block.
    pub is_folded: bool,
}

/// Frontmatter as ordered entries.
///
/// Enough YAML for a unit's frontmatter: a scalar on the key's own line, or a
/// block — list, map, or folded string — carried by the lines under it. Not a
/// YAML parser, and not trying to be: a unit's frontmatter belongs to its
/// host, and this reads exactly what the host's field table describes.
#[must_use]
pub fn parse_frontmatter(fm: &[&str]) -> Vec<FmEntry> {
    static KEY: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([A-Za-z_][\w-]*):[ \t]*(.*)$").unwrap());
    static NEXT_KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][\w-]*:").unwrap());
    static QUOTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^(["'])(.*)["']$"#).unwrap());

    let mut entries = Vec::new();
    for (i, line) in fm.iter().enumerate() {
        let Some(caps) = KEY.captures(line) else {
            continue;
        };
        let key = caps[1].to_string();
        let rest = caps[2].trim().to_string();

        let mut block = Vec::new();
        for next in fm.iter().skip(i + 1) {
            if NEXT_KEY.is_match(next) {
                break;
            }
            if !next.trim().is_empty() {
                block.push((*next).to_string());
            }
        }

        let scalar = QUOTED
            .captures(&rest)
            .map_or(rest.clone(), |c| c[2].to_string());
        let is_list = !block.is_empty() && block.iter().all(|l| l.trim_start().starts_with("- "));
        entries.push(FmEntry {
            key,
            line: i + 1,
            scalar: (!scalar.is_empty()).then_some(scalar),
            block: (!block.is_empty()).then_some(block),
            is_list,
            is_folded: rest.starts_with('|') || rest.starts_with('>'),
        });
    }
    entries
}

/// Whether `key` is present with a value.
#[must_use]
pub fn fm_has(fm: &[&str], key: &str) -> bool {
    parse_frontmatter(fm)
        .iter()
        .any(|e| e.key == key && (e.scalar.is_some() || e.block.is_some()))
}

/// The scalar written under `key`, if any.
#[must_use]
pub fn fm_value(fm: &[&str], key: &str) -> Option<String> {
    parse_frontmatter(fm)
        .into_iter()
        .find(|e| e.key == key)
        .and_then(|e| e.scalar)
}

/// The text with fenced blocks blanked and inline code spans kept.
#[must_use]
pub fn unfenced(lines: &[&str], fenced: &[bool]) -> String {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| if fenced[i] { "" } else { *l })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// The text with fenced blocks *and* inline code spans blanked, for scanning
/// tags.
///
/// Blanking pads each character with as many spaces as it occupies in UTF-8,
/// so every byte offset into the result still points where it did. The padding
/// is invisible downstream: bodies are read through [`norm`], which collapses
/// it.
#[must_use]
pub fn prose_only(lines: &[&str], fenced: &[bool]) -> String {
    let text = unfenced(lines, fenced);
    let mut out = String::with_capacity(text.len());
    let mut in_span = false;
    for ch in text.chars() {
        if ch == '`' {
            // The backticks go too, so a span leaves nothing tag-shaped.
            in_span = !in_span;
            out.push(' ');
        } else if in_span && ch != '\n' {
            for _ in 0..ch.len_utf8() {
                out.push(' ');
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// One `yaml` fence found in a schema file.
pub struct YamlFence {
    /// The lines between the markers.
    pub text: String,
    /// The marker that opened it, so its length can be checked.
    pub marker: String,
    /// The one-based line the marker sits on.
    pub line: usize,
}

/// Every ```` ```yaml ```` fence in a schema file, in order.
#[must_use]
pub fn extract_yaml(text: &str) -> Vec<YamlFence> {
    static MARK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(`{3,}|~{3,})\s*(\S*)\s*$").unwrap());
    let lines: Vec<&str> = crate::validate::lines(text);
    let Some(split) = split_frontmatter(&lines) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let mut open: Option<(String, usize)> = None;
    let mut body: Option<Vec<&str>> = None;
    for (i, line) in lines.iter().enumerate().skip(split.body_start) {
        let found = MARK.captures(line);
        match (&open, &found) {
            (None, Some(c)) => {
                open = Some((c[1].to_string(), i + 1));
                body = (&c[2] == "yaml").then(Vec::new);
            }
            (Some((marker, at)), Some(c))
                if c[1].as_bytes()[0] == marker.as_bytes()[0]
                    && c[1].len() >= marker.len()
                    && c[2].is_empty() =>
            {
                if let Some(collected) = body.take() {
                    out.push(YamlFence {
                        text: collected.join("\n"),
                        marker: marker.clone(),
                        line: *at,
                    });
                }
                open = None;
            }
            (Some(_), _) => {
                if let Some(collected) = body.as_mut() {
                    collected.push(line);
                }
            }
            (None, None) => {}
        }
    }
    out
}

/// One element found in a unit's prose.
#[derive(Debug, Clone)]
pub struct Node {
    /// The tag name.
    pub name: String,
    /// Its attributes, in written order.
    pub attrs: Vec<(String, String)>,
    /// Whether it closed itself.
    pub self_closing: bool,
    /// Everything between the open and close tags, blanked as [`prose_only`]
    /// leaves it.
    pub body: String,
    /// The element it sits directly inside, or `None` at the file root.
    pub parent: Option<String>,
    /// Byte offset of the opening tag, which fixes document order.
    pub index: usize,
    /// The opening tag verbatim, for quoting in a finding.
    pub raw: String,
}

impl Node {
    /// The value written for `name`, if any.
    #[must_use]
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Scan prose into element nodes: attributes, immediate parent, and body.
///
/// Anything tag-shaped the strict scanner does not consume is malformed — an
/// unquoted attribute value, a stray angle bracket, a broken close tag — and
/// is reported rather than passed over, because a silently-dropped tag is a
/// rule that stops being checked.
pub fn parse_elements(prose: &str, errs: &mut Vec<String>) -> Vec<Node> {
    static TAG: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"<(/)?([A-Za-z_][\w-]*)((?:\s+[\w-]+\s*=\s*"[^"]*")*)\s*(/)?>"#).unwrap()
    });
    static ATTR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"([\w-]+)\s*=\s*"([^"]*)""#).unwrap());
    static SHAPED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"</?[A-Za-z_][\w-]*").unwrap());

    let mut nodes: Vec<Node> = Vec::new();
    // Indices into `nodes` of the elements still open, outermost first.
    let mut stack: Vec<usize> = Vec::new();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    // Where each open element's body began.
    let mut body_start: Vec<usize> = Vec::new();

    for m in TAG.captures_iter(prose) {
        let whole = m.get(0).unwrap();
        spans.push((whole.start(), whole.end()));
        let name = m[2].to_string();

        if m.get(1).is_some() {
            let Some(depth) = stack.iter().rposition(|&i| nodes[i].name == name) else {
                errs.push(format!("unbalanced </{name}>"));
                continue;
            };
            if depth != stack.len() - 1 {
                let inner = nodes[*stack.last().unwrap()].name.clone();
                errs.push(format!("</{name}> closes across an unclosed <{inner}>"));
            }
            let node = stack[depth];
            nodes[node].body = prose[body_start[depth]..whole.start()].to_string();
            stack.truncate(depth);
            body_start.truncate(depth);
            continue;
        }

        let attrs = m.get(3).map_or_else(Vec::new, |a| {
            ATTR.captures_iter(a.as_str())
                .map(|c| (c[1].to_string(), c[2].to_string()))
                .collect()
        });
        let self_closing = m.get(4).is_some();
        nodes.push(Node {
            name,
            attrs,
            self_closing,
            body: String::new(),
            parent: stack.last().map(|&i| nodes[i].name.clone()),
            index: whole.start(),
            raw: whole.as_str().to_string(),
        });
        if !self_closing {
            stack.push(nodes.len() - 1);
            body_start.push(whole.end());
        }
    }

    for &i in &stack {
        errs.push(format!("unclosed <{}>", nodes[i].name));
    }
    for m in SHAPED.find_iter(prose) {
        if !spans.iter().any(|&(a, b)| m.start() >= a && m.start() < b) {
            let tail: String = prose[m.start()..].chars().take(60).collect();
            let line = tail.split('\n').next().unwrap_or_default();
            errs.push(format!("malformed tag near \"{line}\""));
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fence_is_closed_only_by_its_own_marker_or_longer() {
        let lines = vec!["````yaml", "```", "still inside", "````", "out"];
        assert_eq!(fence_map(&lines), vec![true, true, true, true, false]);
    }

    /// The bug this exists to keep fixed: a code span may wrap across a line,
    /// and blanking that stopped at the newline left the second half of the
    /// span visible, so `<name>` in a CLI placeholder parsed as an element.
    #[test]
    fn a_code_span_is_blanked_across_a_newline() {
        let lines = vec!["a `span <tag>", "still spanning` b", "<real />"];
        let fenced = fence_map(&lines);
        let prose = prose_only(&lines, &fenced);
        assert!(
            !prose.contains("<tag>"),
            "the span must not leave a tag: {prose:?}"
        );
        assert!(prose.contains("<real />"), "prose outside a span survives");
        assert_eq!(prose.lines().count(), 3, "the newline is kept");
    }

    /// Blanking must not move anything: a multi-byte character before a tag
    /// would otherwise shift every later offset.
    #[test]
    fn blanking_preserves_byte_offsets() {
        let lines = vec!["an em—dash and `a span` then <tag />"];
        let fenced = fence_map(&lines);
        let prose = prose_only(&lines, &fenced);
        assert_eq!(prose.len(), lines[0].len(), "byte length is unchanged");
        assert_eq!(prose.find("<tag />"), lines[0].find("<tag />"));
    }

    #[test]
    fn frontmatter_reads_scalars_lists_and_folded_blocks() {
        let fm = vec![
            "name: sd-thing",
            "description: >",
            "  folded over",
            "  two lines",
            "allowed-tools:",
            "  - Read",
            "  - Write",
        ];
        let e = parse_frontmatter(&fm);
        assert_eq!(e.len(), 3);
        assert_eq!(e[0].scalar.as_deref(), Some("sd-thing"));
        assert!(e[1].is_folded && !e[1].is_list);
        assert!(e[2].is_list);
        assert_eq!(fm_value(&fm, "name").as_deref(), Some("sd-thing"));
        assert!(fm_has(&fm, "allowed-tools"));
        assert!(!fm_has(&fm, "model"));
    }

    #[test]
    fn a_yaml_fence_is_found_with_the_marker_that_opened_it() {
        let text = "---\ntype: Schema\n---\n\n# T\n\n````yaml\nkey: value\n````\n";
        let found = extract_yaml(text);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "key: value");
        assert_eq!(found[0].marker, "````");
        assert_eq!(found[0].line, 7);
    }

    #[test]
    fn elements_carry_their_parent_attributes_and_body() {
        let mut errs = Vec::new();
        let nodes = parse_elements(
            "<skill name=\"x\">\n<rules>\n<rule level=\"MUST\">stay</rule>\n</rules>\n</skill>",
            &mut errs,
        );
        assert!(errs.is_empty(), "{errs:?}");
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, ["skill", "rules", "rule"]);
        assert_eq!(nodes[2].parent.as_deref(), Some("rules"));
        assert_eq!(nodes[2].attr("level"), Some("MUST"));
        assert_eq!(norm(&nodes[2].body), "stay");
        assert_eq!(nodes[0].attr("name"), Some("x"));
    }

    #[test]
    fn an_unclosed_element_and_a_crossed_close_are_both_reported() {
        let mut errs = Vec::new();
        parse_elements(
            "<rules>\n<rule level=\"MUST\">no close\n</rules>",
            &mut errs,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("closes across an unclosed <rule>")),
            "{errs:?}"
        );
    }

    #[test]
    fn a_tag_shaped_fragment_the_scanner_cannot_read_is_reported() {
        let mut errs = Vec::new();
        parse_elements("<step name=unquoted />", &mut errs);
        assert!(
            errs.iter().any(|e| e.starts_with("malformed tag near")),
            "{errs:?}"
        );
    }
}
