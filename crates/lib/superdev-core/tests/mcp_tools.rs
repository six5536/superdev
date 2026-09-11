//! The SOKF MCP server, driven by a real rmcp client over an in-process
//! duplex pipe — the transport is the only thing these tests stub.

use std::path::Path;
use std::process::Command;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::{RoleClient, RunningService};
use serde_json::{Map, Value};
use superdev_core::sokf::{IndexDir, SokfServer};
use tempfile::TempDir;

const MANIFEST: &str = "sokf: \"0.1\"\nname: fixture-knowledge\n";

const SPEC: &str = r#"---
type: Spec
id: spec-a
title: Spec A
description: The mapping format every module reads.
---

# Format

Mappings are pairs of paths.
"#;

/// The module concept: an outbound `depends-on`, an outbound `references`,
/// two body sections, and one body link that points at nothing.
const MODULE: &str = r#"---
type: Module
id: module-a
title: Module A
description: Pure planning stage; computes actions without touching the filesystem.
tags: [core, planning]
links:
  - rel: depends-on
    to: spec-a
    note: Reads the mappings.
  - rel: references
    to: draft-c
---

# Role

The planning stage reads [spec-a](spec.md) and emits actions. It never writes.

# Caveats

Open questions live in [draft-c](notes/draft.md); [the retired note](missing.md) is gone.
"#;

const DRAFT: &str = r#"---
type: Reference
id: draft-c
title: Draft C
status: draft
description: Open questions, none of them decided.
---

# Questions

Nothing decided yet.
"#;

/// A concept whose body carries both include-block kinds, so a resolver
/// answer has generated regions to report.
const RENDERED: &str = r#"---
type: Reference
id: rendered-d
title: Rendered D
description: Carries generated content a caller must not author by hand.
---

# Definition

<!-- sokf:include /knowledge/spec.md#format -->
```markdown
Mappings are pairs of paths.
```
<!-- /sokf:include -->

# Shared

<!-- sokf:include draft-c -->
Nothing decided yet.
<!-- /sokf:include -->
"#;

/// A repo root holding a `knowledge/` bundle and an index directory.
fn fixture() -> TempDir {
    let repo = tempfile::tempdir().unwrap();
    let bundle = repo.path().join("knowledge");
    std::fs::create_dir_all(bundle.join("notes")).unwrap();
    std::fs::write(bundle.join("manifest.sokf.yaml"), MANIFEST).unwrap();
    std::fs::write(bundle.join("spec.md"), SPEC).unwrap();
    std::fs::write(bundle.join("module-a.md"), MODULE).unwrap();
    std::fs::write(bundle.join("notes/draft.md"), DRAFT).unwrap();
    std::fs::write(bundle.join("rendered.md"), RENDERED).unwrap();
    repo
}

/// The structured half of a successful machine result, with its mirrored
/// text item proven to carry the same JSON and nothing else.
fn structured_of(result: &CallToolResult) -> Value {
    assert_ne!(result.is_error, Some(true), "{}", text_of(result));
    let structured = result
        .structured_content
        .as_ref()
        .expect("a machine result carries structured content")
        .clone();
    assert_eq!(result.content.len(), 1, "{:?}", result.content);
    assert_eq!(
        text_of(result),
        serde_json::to_string_pretty(&structured).unwrap()
    );
    structured
}

/// Serve the fixture bundle over `tokio::io::duplex` and return a connected
/// client. The server task ends when the client disconnects.
async fn serve_and_client(repo: &Path) -> RunningService<RoleClient, ()> {
    let (server_transport, client_transport) = tokio::io::duplex(4096);
    let server = SokfServer::new(
        repo.join("knowledge"),
        repo.to_path_buf(),
        IndexDir(repo.join("index")),
        None,
    );
    tokio::spawn(async move {
        let running = server.serve(server_transport).await.unwrap();
        let _ = running.waiting().await;
    });
    ().serve(client_transport).await.unwrap()
}

/// Call one tool with a JSON object of arguments.
async fn call(
    client: &RunningService<RoleClient, ()>,
    name: &'static str,
    arguments: Value,
) -> CallToolResult {
    let arguments: Map<String, Value> = match arguments {
        Value::Object(map) => map,
        other => panic!("arguments must be an object, got {other}"),
    };
    client
        .call_tool(CallToolRequestParams::new(name).with_arguments(arguments))
        .await
        .unwrap()
}

/// Every text block of a result, joined.
fn text_of(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|block| block.as_text().map(|text| text.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A `path:start-end` locator for `file` appears in `text`.
fn has_locator(text: &str, file: &str) -> bool {
    text.lines().any(|line| {
        let Some(rest) = line.trim_start().strip_prefix(file) else {
            return false;
        };
        let Some(range) = rest.strip_prefix(':') else {
            return false;
        };
        let range: String = range
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        let (start, end) = match range.split_once('-') {
            Some(parts) => parts,
            None => return false,
        };
        !start.is_empty() && !end.is_empty()
    })
}

#[tokio::test]
async fn search_returns_locators() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let result = call(
        &client,
        "sokf_search",
        serde_json::json!({"query": "planning stage"}),
    )
    .await;
    let text = text_of(&result);

    assert_ne!(result.is_error, Some(true), "{text}");
    assert!(text.contains("module-a.md:"), "{text}");
    assert!(has_locator(&text, "module-a.md"), "{text}");
    // The concept line leads its group.
    assert!(text.contains("module-a — Pure planning stage"), "{text}");
    // No embedder was passed, so the caller is told search is lexical.
    assert!(
        text.contains("note: semantic search unavailable (lexical only)"),
        "{text}"
    );
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn an_absurd_limit_is_answered_not_aborted() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    // Retrieval widens the limit before tantivy allocates against it, so an
    // unbounded one takes the process down with it.
    let result = call(
        &client,
        "sokf_search",
        serde_json::json!({"query": "planning stage", "limit": u32::MAX}),
    )
    .await;
    let text = text_of(&result);
    assert_ne!(result.is_error, Some(true), "{text}");
    assert!(text.contains("module-a.md:"), "{text}");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn the_tool_roster_splits_source_resolution_from_semantic_retrieval() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;
    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(
        names,
        [
            "sokf_edit",
            "sokf_graph",
            "sokf_overview",
            "sokf_resolve_source",
            "sokf_retrieve",
            "sokf_search",
            "sokf_write"
        ]
    );
    // The mixed operation is gone, with no compatibility alias behind it.
    assert!(!names.contains(&"sokf_read"));

    let schema = |name: &str| {
        Value::Object(
            tools
                .iter()
                .find(|tool| tool.name == name)
                .unwrap()
                .input_schema
                .as_ref()
                .clone(),
        )
    };

    let resolve = schema("sokf_resolve_source");
    assert_eq!(resolve["additionalProperties"], false);
    assert_eq!(resolve["required"], serde_json::json!(["path"]));
    let properties = resolve["properties"].as_object().unwrap();
    assert_eq!(properties.len(), 1);
    assert_eq!(properties["path"]["type"], "string");

    // The knowledge is the whole subject, so the overview narrows by nothing
    // and its request has no property to supply.
    let overview = schema("sokf_overview");
    assert_eq!(overview["additionalProperties"], false);
    assert!(
        overview
            .get("properties")
            .is_none_or(|p| p.as_object().unwrap().is_empty()),
        "{overview}"
    );
    assert!(
        overview
            .get("required")
            .is_none_or(|r| r.as_array().unwrap().is_empty()),
        "{overview}"
    );

    let retrieve = schema("sokf_retrieve");
    assert_eq!(retrieve["additionalProperties"], false);
    assert_eq!(retrieve["required"], serde_json::json!(["path"]));
    let properties = retrieve["properties"].as_object().unwrap();
    assert_eq!(properties.len(), 3);
    assert_eq!(properties["path"]["type"], "string");
    for optional in ["offset", "limit"] {
        assert_eq!(properties[optional]["minimum"], 1, "{optional}");
        assert_eq!(
            properties[optional]["type"],
            serde_json::json!(["integer", "null"]),
            "{optional}"
        );
    }

    // A closed schema is a runtime promise, not only a published one.
    for rejected in [
        (
            "sokf_resolve_source",
            serde_json::json!({"path": "sokf:spec-a", "offset": 1}),
        ),
        ("sokf_resolve_source", serde_json::json!({})),
        (
            "sokf_retrieve",
            serde_json::json!({"path": "sokf:spec-a", "heading": "Format"}),
        ),
        ("sokf_retrieve", serde_json::json!({"offset": 1})),
        ("sokf_overview", serde_json::json!({"path": "sokf:"})),
    ] {
        let result = call(&client, rejected.0, rejected.1.clone()).await;
        assert_eq!(
            result.is_error,
            Some(true),
            "{}: {}",
            rejected.1,
            text_of(&result)
        );
    }
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn the_server_instructions_name_only_the_tools_it_serves() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;
    let served = client.list_all_tools().await.unwrap();
    // Read the value the server sends, not a copy of it.
    let instructions = client
        .peer_info()
        .and_then(|info| info.instructions.clone())
        .expect("the server sends initialization instructions");

    for tool in &served {
        assert!(
            instructions.contains(tool.name.as_ref()),
            "{} is served but unnamed: {instructions}",
            tool.name
        );
    }
    assert!(!instructions.contains("sokf_read"), "{instructions}");
    // No client is directed to read the overview address as a file.
    assert!(!instructions.contains("Read `sokf:`"), "{instructions}");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn retrieve_renders_overview_concept_and_section() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let whole = text_of(
        &call(
            &client,
            "sokf_retrieve",
            serde_json::json!({"path": "sokf:module-a"}),
        )
        .await,
    );
    assert!(whole.contains("type: Module"), "{whole}");
    assert!(whole.contains("depends-on -> spec-a"), "{whole}");
    assert!(whole.contains("[Role]"), "{whole}");
    assert!(whole.contains("[Caveats]"), "{whole}");
    assert!(has_locator(&whole, "module-a.md"), "{whole}");

    let section = text_of(
        &call(
            &client,
            "sokf_retrieve",
            serde_json::json!({"path": "sokf:module-a#Role"}),
        )
        .await,
    );
    assert!(section.contains("[Role]"), "{section}");
    assert!(!section.contains("Caveats"), "{section}");

    // `(root)` is the label every locator line shows, so it has to resolve.
    let root = text_of(
        &call(
            &client,
            "sokf_retrieve",
            serde_json::json!({"path": "sokf:module-a#(root)"}),
        )
        .await,
    );
    assert!(root.contains("[(root)]"), "{root}");
    assert!(!root.contains("[Role]"), "{root}");

    let unknown = call(
        &client,
        "sokf_retrieve",
        serde_json::json!({"path": "sokf:module"}),
    )
    .await;
    let text = text_of(&unknown);
    assert_eq!(unknown.is_error, Some(true), "{text}");
    assert!(text.contains("module-a"), "{text}");

    for malformed in ["sokf:#Role", "sokf:module-a#", "sokf:../../outside"] {
        let result = call(
            &client,
            "sokf_retrieve",
            serde_json::json!({"path": malformed}),
        )
        .await;
        assert_eq!(
            result.is_error,
            Some(true),
            "{malformed}: {}",
            text_of(&result)
        );
    }

    let window = text_of(
        &call(
            &client,
            "sokf_retrieve",
            serde_json::json!({"path": "sokf:module-a#Role", "offset": 2, "limit": 2}),
        )
        .await,
    );
    assert_eq!(window.lines().count(), 2, "{window}");
    assert!(!window.contains("module-a —"), "{window}");

    // A file the parser rejected is quoted, not guessed at.
    std::fs::write(
        repo.path().join("knowledge/notes/torn.md"),
        "type: Reference\r\nid: torn\r\n",
    )
    .unwrap();
    let torn = call(
        &client,
        "sokf_retrieve",
        serde_json::json!({"path": "sokf:notes/torn.md"}),
    )
    .await;
    assert_eq!(torn.is_error, Some(true));
    assert!(
        text_of(&torn).contains("does not parse"),
        "{}",
        text_of(&torn)
    );
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn retrieve_refuses_physical_paths_and_points_at_the_resolver() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    for physical in [
        serde_json::json!({"path": "knowledge/spec.md"}),
        serde_json::json!({"path": repo.path().join("knowledge/spec.md")}),
        serde_json::json!({"path": "spec.md"}),
    ] {
        let result = call(&client, "sokf_retrieve", physical.clone()).await;
        let text = text_of(&result);
        assert_eq!(result.is_error, Some(true), "{physical}: {text}");
        assert!(text.contains("sokf_resolve_source"), "{physical}: {text}");
    }
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn resolve_source_answers_existing_missing_and_generated_targets() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    // An identity and its physical spelling name one canonical target.
    for argument in [
        serde_json::json!({"path": "sokf:module-a"}),
        serde_json::json!({"path": "knowledge/module-a.md"}),
        serde_json::json!({"path": repo.path().join("knowledge/module-a.md")}),
        serde_json::json!({"path": "knowledge/notes/../module-a.md"}),
    ] {
        let resolved = structured_of(&call(&client, "sokf_resolve_source", argument.clone()).await);
        assert_eq!(
            resolved,
            serde_json::json!({
                "ingressPath": "knowledge/module-a.md",
                "canonicalPath": "knowledge/module-a.md",
                "exists": true,
                "generatedRegions": []
            }),
            "{argument}"
        );
    }

    // A missing destination below an existing contained ancestor resolves,
    // reports its absence, and creates nothing.
    let missing = structured_of(
        &call(
            &client,
            "sokf_resolve_source",
            serde_json::json!({"path": "knowledge/notes/new.md"}),
        )
        .await,
    );
    assert_eq!(
        missing,
        serde_json::json!({
            "ingressPath": "knowledge/notes/new.md",
            "canonicalPath": "knowledge/notes/new.md",
            "exists": false,
            "generatedRegions": []
        })
    );
    assert!(!repo.path().join("knowledge/notes/new.md").exists());

    // Generated spans name their authority: a source region by path and
    // region, a shared concept by the file that authors it.
    let rendered = structured_of(
        &call(
            &client,
            "sokf_resolve_source",
            serde_json::json!({"path": "sokf:rendered-d"}),
        )
        .await,
    );
    assert_eq!(
        rendered["generatedRegions"],
        serde_json::json!([
            {
                "startLine": 11,
                "endLine": 13,
                "authoritativePath": "knowledge/spec.md",
                "authoritativeRegion": "format"
            },
            {
                "startLine": 19,
                "endLine": 19,
                "authoritativePath": "knowledge/notes/draft.md"
            }
        ]),
        "{rendered}"
    );
    let source = std::fs::read_to_string(repo.path().join("knowledge/rendered.md")).unwrap();
    let lines: Vec<&str> = source.lines().collect();
    assert_eq!(lines[10], "```markdown");
    assert_eq!(lines[12], "```");
    assert_eq!(lines[18], "Nothing decided yet.");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn resolve_source_refuses_semantic_addresses_escapes_and_directories() {
    let repo = fixture();
    std::fs::write(repo.path().join("outside.md"), "secret\n").unwrap();
    let client = serve_and_client(repo.path()).await;

    let cases = [
        // Rendered knowledge is the retrieval operation's question.
        ("sokf:", "sokf_retrieve"),
        ("sokf:module-a#Role", "sokf_retrieve"),
        // A direct physical argument must enter through `knowledge/`.
        ("README.md", "outside"),
        ("knowledge/../outside.md", "outside"),
        // Directories are not source files.
        ("knowledge/notes", "directory"),
        // An unknown identity names its near misses.
        ("sokf:module", "module-a"),
    ];
    for (argument, expected) in cases {
        let result = call(
            &client,
            "sokf_resolve_source",
            serde_json::json!({"path": argument}),
        )
        .await;
        let text = text_of(&result);
        assert_eq!(result.is_error, Some(true), "{argument}: {text}");
        assert!(text.contains(expected), "{argument}: {text}");
    }

    let absolute = repo.path().join("outside.md");
    let refused = call(
        &client,
        "sokf_resolve_source",
        serde_json::json!({"path": absolute}),
    )
    .await;
    assert_eq!(refused.is_error, Some(true));
    assert_eq!(
        std::fs::read_to_string(repo.path().join("outside.md")).unwrap(),
        "secret\n"
    );
    client.cancel().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn resolve_source_accepts_contained_symlinks_and_refuses_escaping_ones() {
    use std::os::unix::fs::symlink;

    let repo = fixture();
    // A contained target outside `knowledge/` is still inside the checkout.
    std::fs::create_dir(repo.path().join("vendor")).unwrap();
    std::fs::write(repo.path().join("vendor/shared.md"), "shared\n").unwrap();
    symlink(
        repo.path().join("vendor/shared.md"),
        repo.path().join("knowledge/linked.md"),
    )
    .unwrap();
    symlink(
        repo.path().join("vendor"),
        repo.path().join("knowledge/vendored"),
    )
    .unwrap();

    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("escaped.md"), "escaped\n").unwrap();
    symlink(
        outside.path().join("escaped.md"),
        repo.path().join("knowledge/escaping.md"),
    )
    .unwrap();
    symlink(outside.path(), repo.path().join("knowledge/elsewhere")).unwrap();

    let client = serve_and_client(repo.path()).await;

    // Contained: the canonical target wins over the ingress spelling, and a
    // missing suffix below a contained directory link still resolves.
    for (argument, canonical, exists) in [
        ("knowledge/linked.md", "vendor/shared.md", true),
        ("knowledge/vendored/shared.md", "vendor/shared.md", true),
        ("knowledge/vendored/new.md", "vendor/new.md", false),
    ] {
        let resolved = structured_of(
            &call(
                &client,
                "sokf_resolve_source",
                serde_json::json!({"path": argument}),
            )
            .await,
        );
        assert_eq!(resolved["ingressPath"], argument, "{argument}");
        assert_eq!(resolved["canonicalPath"], canonical, "{argument}");
        assert_eq!(resolved["exists"], exists, "{argument}");
    }

    // Escaping: refused before the target is opened, including below a
    // missing suffix.
    for argument in [
        "knowledge/escaping.md",
        "knowledge/elsewhere/escaped.md",
        "knowledge/elsewhere/new.md",
    ] {
        let result = call(
            &client,
            "sokf_resolve_source",
            serde_json::json!({"path": argument}),
        )
        .await;
        let text = text_of(&result);
        assert_eq!(result.is_error, Some(true), "{argument}: {text}");
        assert!(
            text.contains("outside the active checkout"),
            "{argument}: {text}"
        );
    }
    assert!(!outside.path().join("new.md").exists());
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn graph_map_and_neighbours() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let map = text_of(&call(&client, "sokf_graph", serde_json::json!({})).await);
    assert!(
        map.contains("--depends-on--> spec-a  knowledge/spec.md"),
        "{map}"
    );
    assert!(map.contains("(Reads the mappings.)"), "{map}");
    // The source is named once, with the path a reader opens next.
    assert!(map.contains("module-a  knowledge/module-a.md"), "{map}");

    let neighbours =
        text_of(&call(&client, "sokf_graph", serde_json::json!({"id": "spec-a"})).await);
    // spec-a declares nothing; the hop is the inverse of module-a's edge.
    assert!(
        neighbours.contains("<--depends-on-- module-a  knowledge/module-a.md"),
        "{neighbours}"
    );
    assert!(neighbours.contains("Pure planning stage"), "{neighbours}");
    assert!(
        neighbours.starts_with("spec-a  knowledge/spec.md"),
        "{neighbours}"
    );

    // Every path a traversal reports names a file it can open, spelled
    // repository-relative from components rather than by substring luck.
    let expected = Path::new("knowledge").join("module-a.md");
    let expected = expected.to_string_lossy().replace('\\', "/");
    assert!(map.contains(&expected), "{map}");
    for rendered in [&map, &neighbours] {
        for token in rendered.split_whitespace().filter(|t| t.ends_with(".md")) {
            assert!(repo.path().join(token).is_file(), "{token}");
        }
    }
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn overview_orients_and_warns() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let text = text_of(&call(&client, "sokf_overview", serde_json::json!({})).await);
    assert!(text.contains("fixture-knowledge"), "{text}");
    assert!(text.contains("4 concepts"), "{text}");
    assert!(text.contains("notes/"), "{text}");
    assert!(text.contains("draft-c — Open questions"), "{text}");
    assert!(text.contains("warnings:"), "{text}");
    assert!(text.contains("missing.md"), "{text}");
    assert!(text.contains("lexical only"), "{text}");

    // One rendering, not two: the dedicated tool answers with what the
    // retrieval address answered with. The `synced:` line reports what that
    // call's sync did and is absent once the index is warm, so it is dropped
    // from both sides rather than asserted equal across a warm and a cold
    // call.
    let through_retrieve = text_of(
        &call(
            &client,
            "sokf_retrieve",
            serde_json::json!({"path": "sokf:"}),
        )
        .await,
    );
    let without_sync = |rendered: &str| {
        rendered
            .lines()
            .filter(|line| !line.starts_with("synced: "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(without_sync(&text), without_sync(&through_retrieve));

    // Orientation only: no concept body reaches the overview.
    assert!(!text.contains("Pure planning stage in prose"), "{text}");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn mutations_use_agent_safe_policy_and_return_structured_results() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let edited = call(
        &client,
        "sokf_edit",
        serde_json::json!({
            "path": "sokf:module-a",
            "edits": [{
                "oldText": "It never writes.",
                "newText": "It only plans."
            }]
        }),
    )
    .await;
    assert_ne!(edited.is_error, Some(true), "{}", text_of(&edited));
    let details = edited.structured_content.as_ref().unwrap();
    assert_eq!(details["applied"], true);
    // The fixture deliberately carries a body link to a missing file. Applied
    // invalid is a successful result, not an error that invites a retry.
    assert_eq!(details["validation"], "invalid");
    assert_eq!(details["resolvedPath"], "knowledge/module-a.md");
    assert!(
        std::fs::read_to_string(repo.path().join("knowledge/module-a.md"))
            .unwrap()
            .contains("It only plans.")
    );

    let before = std::fs::read(repo.path().join("knowledge/module-a.md")).unwrap();
    let rejected = call(
        &client,
        "sokf_edit",
        serde_json::json!({
            "path": "module-a",
            "edits": [{"oldText": "id: module-a", "newText": "id: changed"}]
        }),
    )
    .await;
    assert_eq!(rejected.is_error, Some(true));
    assert!(text_of(&rejected).contains("must preserve `id`"));
    assert_eq!(
        std::fs::read(repo.path().join("knowledge/module-a.md")).unwrap(),
        before
    );

    let written = call(
        &client,
        "sokf_write",
        serde_json::json!({
            "path": "knowledge/new.md",
            "content": "---\ntype: Note\nid: new-note\n---\nNew.\n"
        }),
    )
    .await;
    assert_ne!(written.is_error, Some(true), "{}", text_of(&written));
    assert_eq!(
        written.structured_content.as_ref().unwrap()["finalPath"],
        "knowledge/new.md"
    );
    client.cancel().await.unwrap();
}

/// Run one git command in `dir`, failing the test with git's own words.
///
/// The fixture is hermetic: system and global configuration are cut off, so
/// a developer's signing key or templates cannot reach it — an inherited
/// `commit.gpgsign` otherwise blocks the commit on an agent that never
/// answers. The sentinel is a path that does not exist rather than
/// `/dev/null`, which git reads as an empty configuration on every platform
/// this suite runs on, Windows included.
fn git(dir: &Path, args: &[&str]) {
    let absent = dir.join("no-such-git-config");
    let output = Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", &absent)
        .env("GIT_CONFIG_SYSTEM", &absent)
        .env("GIT_AUTHOR_NAME", "fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A linked worktree is its own repository: the same identity carries
/// different bytes in each checkout, and neither may reach the other.
#[tokio::test]
async fn a_linked_worktree_serves_its_own_checkout_and_refuses_the_other() {
    let main = fixture();
    git(main.path(), &["init", "--initial-branch=main"]);
    git(main.path(), &["add", "-A"]);
    git(main.path(), &["commit", "-m", "fixture"]);

    // The worktree lives in its own temporary directory, so concurrent runs
    // neither collide nor leave a checkout behind.
    let elsewhere = tempfile::tempdir().unwrap();
    let worktree = elsewhere.path().join("linked");
    git(
        main.path(),
        &[
            "worktree",
            "add",
            "-b",
            "linked",
            worktree.to_str().unwrap(),
        ],
    );
    // The worktree diverges: one identity, two different bodies.
    let worktree = std::fs::canonicalize(&worktree).unwrap();
    let local = std::fs::read_to_string(worktree.join("knowledge/spec.md"))
        .unwrap()
        .replace(
            "Mappings are pairs of paths.",
            "Only the worktree says this.",
        );
    std::fs::write(worktree.join("knowledge/spec.md"), &local).unwrap();
    // `.git` here is a pointer file, not a directory.
    assert!(worktree.join(".git").is_file());

    let client = serve_and_client(&worktree).await;

    // The worktree's own bytes answer, by identity and by path.
    let resolved = structured_of(
        &call(
            &client,
            "sokf_resolve_source",
            serde_json::json!({"path": "sokf:spec-a"}),
        )
        .await,
    );
    assert_eq!(resolved["canonicalPath"], "knowledge/spec.md");
    assert_eq!(
        std::fs::read_to_string(worktree.join(resolved["canonicalPath"].as_str().unwrap()))
            .unwrap(),
        local
    );
    let retrieved = text_of(
        &call(
            &client,
            "sokf_retrieve",
            serde_json::json!({"path": "sokf:spec-a"}),
        )
        .await,
    );
    assert!(
        retrieved.contains("Only the worktree says this."),
        "{retrieved}"
    );

    // The main checkout is another repository, and off limits — by its
    // absolute path, and by a relative one climbing out of this checkout.
    // Both temporary directories come from the same system root, so the climb
    // is spelled with components rather than a formatted string and reads the
    // same on every platform.
    let other = std::fs::canonicalize(main.path()).unwrap();
    let climbing = Path::new("knowledge")
        .join("..")
        .join("..")
        .join("..")
        .join(other.file_name().unwrap())
        .join("knowledge")
        .join("spec.md");
    // Naming the reason keeps this from passing for the wrong one: a
    // mis-spelled path that simply does not exist would also be an error.
    for argument in [
        serde_json::json!({"path": other.join("knowledge/spec.md")}),
        serde_json::json!({"path": climbing}),
    ] {
        let refused = call(&client, "sokf_resolve_source", argument.clone()).await;
        let text = text_of(&refused);
        assert_eq!(refused.is_error, Some(true), "{argument}: {text}");
        assert!(text.contains("is outside"), "{argument}: {text}");
    }
    // The other checkout's bytes are still its own, and were never read.
    assert!(
        std::fs::read_to_string(other.join("knowledge/spec.md"))
            .unwrap()
            .contains("Mappings are pairs of paths.")
    );
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn stale_index_refreshes_between_calls() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let before = text_of(
        &call(
            &client,
            "sokf_search",
            serde_json::json!({"query": "cadence knob"}),
        )
        .await,
    );
    assert!(!before.contains("spec.md:"), "{before}");

    let spec = repo.path().join("knowledge/spec.md");
    let grown =
        std::fs::read_to_string(&spec).unwrap() + "\n# Cadence\n\nThe cadence knob is quarterly.\n";
    std::fs::write(&spec, grown).unwrap();

    let after = text_of(
        &call(
            &client,
            "sokf_search",
            serde_json::json!({"query": "cadence knob"}),
        )
        .await,
    );
    assert!(has_locator(&after, "spec.md"), "{after}");
    assert!(after.contains("cadence knob"), "{after}");
    client.cancel().await.unwrap();
}
