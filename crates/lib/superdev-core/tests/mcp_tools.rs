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

/// A concept carrying both include-block kinds for overview validation.
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
async fn the_server_serves_three_tools_and_no_file_tool() {
    let repo = fixture();
    let client = serve_and_client(repo.path()).await;
    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(names, ["sokf_graph", "sokf_overview", "sokf_search"]);
    // The server answers what a file tool cannot, and serves no file tool:
    // an agent reads and edits knowledge with its own tools on physical
    // paths. No compatibility alias stands behind a removed operation.
    for removed in [
        "sokf_read",
        "sokf_resolve_source",
        "sokf_retrieve",
        "sokf_edit",
        "sokf_write",
    ] {
        assert!(!names.contains(&removed), "{removed} is still served");
    }

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

    // A closed schema is a runtime promise, not only a published one.
    let rejected = call(
        &client,
        "sokf_overview",
        serde_json::json!({"path": "sokf:"}),
    )
    .await;
    assert_eq!(rejected.is_error, Some(true), "{}", text_of(&rejected));

    // A removed operation is not merely absent from the roster: naming one
    // fails at the protocol, rather than reaching a tool that still answers.
    let gone = client
        .call_tool(CallToolRequestParams::new("sokf_retrieve").with_arguments(Map::new()))
        .await;
    assert!(gone.is_err(), "{gone:?}");
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
    for removed in [
        "sokf_read",
        "sokf_resolve_source",
        "sokf_retrieve",
        "sokf_edit",
        "sokf_write",
    ] {
        assert!(!instructions.contains(removed), "{instructions}");
    }
    // No client is directed to read the overview address as a file.
    assert!(!instructions.contains("Read `sokf:`"), "{instructions}");
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

    // Orientation only: no concept body reaches the overview.
    assert!(!text.contains("The planning stage reads"), "{text}");
    assert!(!text.contains("Mappings are pairs of paths."), "{text}");
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
/// different bytes in each checkout, and the server answers with the ones
/// under the checkout it was started in.
#[tokio::test]
async fn a_linked_worktree_serves_its_own_checkout() {
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

    // The graph names the worktree's own file, and the path it reports is
    // relative to this checkout rather than the one that owns the Git
    // directory.
    let neighbours =
        text_of(&call(&client, "sokf_graph", serde_json::json!({"id": "spec-a"})).await);
    assert!(
        neighbours.starts_with("spec-a  knowledge/spec.md"),
        "{neighbours}"
    );
    // The search index is this checkout's too: it finds what only the
    // worktree says, so the bytes indexed are the local ones.
    let found = text_of(
        &call(
            &client,
            "sokf_search",
            serde_json::json!({"query": "Only the worktree says this"}),
        )
        .await,
    );
    assert!(found.contains("Only the worktree says this."), "{found}");
    assert_eq!(
        std::fs::read_to_string(worktree.join("knowledge/spec.md")).unwrap(),
        local
    );

    // The other checkout's bytes are still its own, and were never read.
    let other = std::fs::canonicalize(main.path()).unwrap();
    assert!(
        std::fs::read_to_string(other.join("knowledge/spec.md"))
            .unwrap()
            .contains("Mappings are pairs of paths.")
    );
    // The index this server wrote belongs to the worktree, not the checkout
    // that owns the shared Git directory.
    assert!(worktree.join("index").exists());
    assert!(!other.join("index").exists());
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
