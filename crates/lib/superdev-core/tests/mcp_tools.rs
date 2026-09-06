//! The SOKF MCP server, driven by a real rmcp client over an in-process
//! duplex pipe — the transport is the only thing these tests stub.

use std::path::Path;

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

/// A repo root holding a `knowledge/` bundle and an index directory.
fn fixture() -> TempDir {
    let repo = tempfile::tempdir().unwrap();
    let bundle = repo.path().join("knowledge");
    std::fs::create_dir_all(bundle.join("notes")).unwrap();
    std::fs::write(bundle.join("manifest.sokf.yaml"), MANIFEST).unwrap();
    std::fs::write(bundle.join("spec.md"), SPEC).unwrap();
    std::fs::write(bundle.join("module-a.md"), MODULE).unwrap();
    std::fs::write(bundle.join("notes/draft.md"), DRAFT).unwrap();
    repo
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
async fn read_schema_and_tool_roster_use_familiar_semantics() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;
    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(
        names,
        [
            "sokf_edit",
            "sokf_graph",
            "sokf_read",
            "sokf_search",
            "sokf_write"
        ]
    );
    let read = tools.iter().find(|tool| tool.name == "sokf_read").unwrap();
    let properties = read.input_schema["properties"].as_object().unwrap();
    assert!(properties.contains_key("path"));
    assert!(properties.contains_key("offset"));
    assert!(properties.contains_key("limit"));
    assert!(!properties.contains_key("id"));
    assert!(!properties.contains_key("heading"));
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn read_whole_and_section() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let whole = text_of(
        &call(
            &client,
            "sokf_read",
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
            "sokf_read",
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
            "sokf_read",
            serde_json::json!({"path": "sokf:module-a#(root)"}),
        )
        .await,
    );
    assert!(root.contains("[(root)]"), "{root}");
    assert!(!root.contains("[Role]"), "{root}");

    let unknown = call(
        &client,
        "sokf_read",
        serde_json::json!({"path": "sokf:module"}),
    )
    .await;
    let text = text_of(&unknown);
    assert_eq!(unknown.is_error, Some(true), "{text}");
    assert!(text.contains("module-a"), "{text}");

    let window = text_of(
        &call(
            &client,
            "sokf_read",
            serde_json::json!({"path": "sokf:module-a#Role", "offset": 2, "limit": 2}),
        )
        .await,
    );
    assert_eq!(window.lines().count(), 2, "{window}");
    assert!(!window.contains("module-a —"), "{window}");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn physical_read_returns_even_an_unparseable_file_verbatim() {
    let repo = fixture();
    std::fs::write(
        repo.path().join("knowledge/notes/torn.md"),
        "type: Reference\nid: torn\n",
    )
    .unwrap();
    let client = serve_and_client(repo.path()).await;

    let expected = "type: Reference\nid: torn\n";
    let relative = call(
        &client,
        "sokf_read",
        serde_json::json!({"path": "knowledge/notes/torn.md"}),
    )
    .await;
    assert_ne!(relative.is_error, Some(true));
    assert_eq!(text_of(&relative), expected.trim_end());

    let absolute = repo.path().join("knowledge/notes/torn.md");
    let rooted = text_of(&call(&client, "sokf_read", serde_json::json!({"path": absolute})).await);
    assert_eq!(rooted, expected.trim_end());

    let outside = repo.path().join("outside.md");
    std::fs::write(&outside, "secret\n").unwrap();
    let refused = call(&client, "sokf_read", serde_json::json!({"path": outside})).await;
    assert_eq!(refused.is_error, Some(true));
    assert!(text_of(&refused).contains("outside"));
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn graph_map_and_neighbours() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let map = text_of(&call(&client, "sokf_graph", serde_json::json!({})).await);
    assert!(
        map.contains("module-a --depends-on--> spec-a  (Reads the mappings.)"),
        "{map}"
    );

    let neighbours =
        text_of(&call(&client, "sokf_graph", serde_json::json!({"id": "spec-a"})).await);
    // spec-a declares nothing; the hop is the inverse of module-a's edge.
    assert!(
        neighbours.contains("<--depends-on-- module-a"),
        "{neighbours}"
    );
    assert!(neighbours.contains("Pure planning stage"), "{neighbours}");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn overview_orients_and_warns() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;

    let text = text_of(&call(&client, "sokf_read", serde_json::json!({"path": "sokf:"})).await);
    assert!(text.contains("fixture-knowledge"), "{text}");
    assert!(text.contains("3 concepts"), "{text}");
    assert!(text.contains("notes/"), "{text}");
    assert!(text.contains("draft-c — Open questions"), "{text}");
    assert!(text.contains("warnings:"), "{text}");
    assert!(text.contains("missing.md"), "{text}");
    assert!(text.contains("lexical only"), "{text}");
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
