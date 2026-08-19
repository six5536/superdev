//! embed.rs — embedding providers: a local model2vec model by default, an
//! HTTP API when the manifest asks for one.
//!
//! Vectors are L2-normalised, so cosine similarity is a dot product.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Local model downloaded on first use: 32M static embeddings, retrieval-tuned.
pub const LOCAL_MODEL: &str = "minishlab/potion-retrieval-32M";

/// Commit pinned for [`LOCAL_MODEL`]. Bumping it changes [`Embedder::model_id`]
/// and so forces a reindex.
pub const LOCAL_MODEL_REVISION: &str = "6fc8051fab2a1e0ee76689cf08c853792ac285e7";

/// The three files `model2vec-rs` needs on disk to load a model folder.
const MODEL_FILES: [&str; 3] = ["config.json", "tokenizer.json", "model.safetensors"];

/// The only API provider implemented so far.
const OPENAI_PROVIDER: &str = "openai";
const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/embeddings";
const OPENAI_KEY_ENV: &str = "OPENAI_API_KEY";

/// Turns text into vectors. `Send + Sync` because the MCP server shares one
/// embedder across request threads.
pub trait Embedder: Send + Sync {
    /// Stable identifier recorded in the index manifest; a change forces a rebuild.
    fn model_id(&self) -> String;
    /// Embed each text, in order.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

/// Manifest `[<capability>.embeddings]`: which API embeds this capability's text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    /// Provider id; only `openai` is implemented.
    pub provider: String,
    /// Provider-side model name, e.g. `text-embedding-3-small`.
    pub model: String,
}

/// Choose the embedder for a capability: an API when the manifest names one,
/// the local model otherwise.
///
/// A local model that will not load (no network on first use, unwritable
/// cache) yields `Ok(None)` — the caller falls back to lexical-only search
/// rather than failing. An unknown provider is a manifest error, since only
/// the user can fix it.
pub fn embedder_from(config: Option<&EmbeddingsConfig>) -> Result<Option<Box<dyn Embedder>>> {
    match config {
        Some(config) => {
            let embedder = ApiEmbedder::new(&config.provider, &config.model)?;
            Ok(Some(Box::new(embedder)))
        }
        None => Ok(Model2VecEmbedder::load()
            .ok()
            .map(|e| Box::new(e) as Box<dyn Embedder>)),
    }
}

/// The local model2vec model, loaded into memory.
pub struct Model2VecEmbedder {
    model: model2vec_rs::model::StaticModel,
}

impl Model2VecEmbedder {
    /// Load [`LOCAL_MODEL`] from the user cache, downloading it on first use.
    ///
    /// The cache is `<cache root>/superdev/models/<model>/<revision>/`, so a
    /// revision bump downloads afresh instead of overwriting.
    // Needs the network and a 124 MiB download; covered by the manual smoke run.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn load() -> Result<Model2VecEmbedder> {
        let dir = model_cache_dir()?;
        download_model(&dir)?;
        // Force normalisation regardless of what the model config says: the
        // index compares vectors with a dot product.
        let model = model2vec_rs::model::StaticModel::from_pretrained(&dir, None, Some(true), None)
            .map_err(|e| Error::Embedding {
                message: format!("loading {}: {e}", dir.display()),
            })?;
        Ok(Model2VecEmbedder { model })
    }
}

impl Embedder for Model2VecEmbedder {
    fn model_id(&self) -> String {
        format!("model2vec:{LOCAL_MODEL}@{LOCAL_MODEL_REVISION}")
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(self.model.encode(texts))
    }
}

/// Fetch any missing model file from the Hub, at the pinned revision.
// Network again: no unit test reaches this.
#[cfg_attr(coverage_nightly, coverage(off))]
fn download_model(dir: &Path) -> Result<()> {
    let missing: Vec<&str> = MODEL_FILES
        .iter()
        .copied()
        .filter(|f| !dir.join(f).is_file())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(dir).map_err(|e| Error::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;
    for file in missing {
        let url =
            format!("https://huggingface.co/{LOCAL_MODEL}/resolve/{LOCAL_MODEL_REVISION}/{file}");
        let response = ureq::get(&url).call().map_err(|e| Error::Embedding {
            message: format!("downloading {url}: {e}"),
        })?;
        // Write beside the target and rename, so an interrupted download never
        // leaves a truncated file that later runs treat as cached.
        let partial = partial_path(dir, file);
        let mut out = std::fs::File::create(&partial).map_err(|e| Error::Io {
            path: partial.clone(),
            source: e,
        })?;
        std::io::copy(&mut response.into_reader(), &mut out).map_err(|e| Error::Io {
            path: partial.clone(),
            source: e,
        })?;
        std::fs::rename(&partial, dir.join(file)).map_err(|e| Error::Io {
            path: partial,
            source: e,
        })?;
    }
    Ok(())
}

/// Scratch file a download writes before renaming onto `file`.
///
/// The pid suffix keeps two processes that warm the cache at the same time off
/// each other's scratch file. Sharing one would let a half-written copy be
/// renamed into place and cached as good, leaving that machine on lexical-only
/// search for ever.
fn partial_path(dir: &Path, file: &str) -> PathBuf {
    dir.join(format!("{file}.partial.{}", std::process::id()))
}

/// Where [`LOCAL_MODEL`] at [`LOCAL_MODEL_REVISION`] is cached.
fn model_cache_dir() -> Result<PathBuf> {
    Ok(cache_root()?
        .join("superdev")
        .join("models")
        .join(LOCAL_MODEL)
        .join(LOCAL_MODEL_REVISION))
}

fn cache_root() -> Result<PathBuf> {
    cache_root_from(
        std::env::var_os("XDG_CACHE_HOME"),
        std::env::var_os("LOCALAPPDATA"),
        std::env::var_os("HOME"),
    )
}

/// `$XDG_CACHE_HOME`, else `%LOCALAPPDATA%`, else `$HOME/.cache`.
fn cache_root_from(
    xdg: Option<OsString>,
    local_appdata: Option<OsString>,
    home: Option<OsString>,
) -> Result<PathBuf> {
    let set = |v: Option<OsString>| v.filter(|v| !v.is_empty()).map(PathBuf::from);
    set(xdg)
        .or_else(|| set(local_appdata))
        .or_else(|| set(home).map(|h| h.join(".cache")))
        .ok_or_else(|| Error::Embedding {
            message: "no cache directory: set XDG_CACHE_HOME or HOME".into(),
        })
}

/// An HTTP embeddings API. The key is read at call time, so constructing one
/// costs nothing and needs no credentials.
#[derive(Debug)]
pub struct ApiEmbedder {
    provider: String,
    model: String,
    endpoint: String,
    key_env: String,
}

impl ApiEmbedder {
    /// Build an embedder for `provider`; fails when the provider is unknown.
    pub fn new(provider: &str, model: &str) -> Result<ApiEmbedder> {
        if provider != OPENAI_PROVIDER {
            return Err(Error::Manifest {
                message: format!("unknown embeddings provider `{provider}`"),
            });
        }
        Ok(ApiEmbedder {
            provider: provider.to_string(),
            model: model.to_string(),
            endpoint: OPENAI_ENDPOINT.to_string(),
            key_env: OPENAI_KEY_ENV.to_string(),
        })
    }
}

impl Embedder for ApiEmbedder {
    fn model_id(&self) -> String {
        format!("{}:{}", self.provider, self.model)
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let key = std::env::var(&self.key_env).map_err(|_| Error::Embedding {
            message: format!("{} is not set", self.key_env),
        })?;
        self.post(&key, texts)
    }
}

/// The slice of the OpenAI response we read. Vectors come back tagged with the
/// input index, so reordering is the server's prerogative, not a bug.
#[derive(Deserialize)]
struct ApiResponse {
    data: Vec<ApiVector>,
}

#[derive(Deserialize)]
struct ApiVector {
    index: usize,
    embedding: Vec<f32>,
}

impl ApiEmbedder {
    // Every line needs a live endpoint and a key.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn post(&self, key: &str, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = ureq::post(&self.endpoint)
            .set("Authorization", &format!("Bearer {key}"))
            .send_json(serde_json::json!({ "model": self.model, "input": texts }))
            .map_err(|e| Error::Embedding {
                message: format!("{}: {e}", self.endpoint),
            })?;
        let mut body: ApiResponse = response.into_json().map_err(|e| Error::Embedding {
            message: format!("{}: unreadable response: {e}", self.endpoint),
        })?;
        if body.data.len() != texts.len() {
            return Err(Error::Embedding {
                message: format!(
                    "{}: asked for {} vectors, got {}",
                    self.endpoint,
                    texts.len(),
                    body.data.len()
                ),
            });
        }
        body.data.sort_by_key(|v| v.index);
        Ok(body.data.into_iter().map(|v| v.embedding).collect())
    }
}

/// Deterministic stand-in for the real embedders: hashed unit vectors, no
/// model, no network.
#[cfg(test)]
pub(crate) struct FakeEmbedder;

#[cfg(test)]
impl Embedder for FakeEmbedder {
    fn model_id(&self) -> String {
        "fake:8".into()
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| hashed_unit_vector(t)).collect())
    }
}

/// FNV-1a over the text, spread across 8 dimensions and L2-normalised.
#[cfg(test)]
fn hashed_unit_vector(text: &str) -> Vec<f32> {
    const DIMS: u32 = 8;
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut v: Vec<f32> = (0..DIMS)
        .map(|d| {
            let bits = hash.rotate_left(d * 8) as u32;
            f32::from(bits as u16) / f32::from(u16::MAX) - 0.5
        })
        .collect();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    // A hash landing exactly on the origin has no direction; any fixed unit
    // vector will do.
    if norm == 0.0 {
        v[0] = 1.0;
        return v;
    }
    for x in &mut v {
        *x /= norm;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_embedder_is_deterministic_and_normalised() {
        let f = FakeEmbedder;
        let a = f.embed(&["hello".into()]).unwrap();
        let b = f.embed(&["hello".into()]).unwrap();
        assert_eq!(a, b);
        let norm: f32 = a[0].iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fake_embedder_separates_texts_and_keeps_input_order() {
        let f = FakeEmbedder;
        let v = f.embed(&["alpha".into(), "beta".into()]).unwrap();
        assert_eq!(v.len(), 2);
        assert_ne!(v[0], v[1]);
        assert_eq!(v[0], f.embed(&["alpha".into()]).unwrap()[0]);
        assert_eq!(f.model_id(), "fake:8");
        assert!(f.embed(&[]).unwrap().is_empty());
        // Degenerate inputs still produce a unit vector.
        let norm: f32 = f.embed(&["".into()]).unwrap()[0]
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn api_config_selects_api_embedder_and_bad_provider_errors() {
        let cfg = EmbeddingsConfig {
            provider: "openai".into(),
            model: "text-embedding-3-small".into(),
        };
        // Construction must not hit the network or require the key; only embed() needs it.
        assert!(embedder_from(Some(&cfg)).unwrap().is_some());
        let bad = EmbeddingsConfig {
            provider: "nope".into(),
            model: "x".into(),
        };
        assert!(embedder_from(Some(&bad)).is_err());
    }

    #[test]
    fn api_embedder_names_its_model_and_short_circuits_empty_input() {
        let e = ApiEmbedder::new("openai", "text-embedding-3-small").unwrap();
        assert_eq!(e.model_id(), "openai:text-embedding-3-small");
        // No key, no network: an empty batch never leaves the process.
        assert!(e.embed(&[]).unwrap().is_empty());
    }

    #[test]
    fn unknown_provider_names_itself() {
        let err = ApiEmbedder::new("cohere", "embed-v3").unwrap_err();
        assert!(matches!(err, Error::Manifest { .. }));
        assert!(err.to_string().contains("cohere"));
    }

    #[test]
    fn cache_root_prefers_xdg_then_local_appdata_then_home() {
        let p = |s: &str| PathBuf::from(s);
        assert_eq!(
            cache_root_from(
                Some("/xdg".into()),
                Some("/appdata".into()),
                Some("/h".into())
            )
            .unwrap(),
            p("/xdg")
        );
        // An empty variable is an unset variable.
        assert_eq!(
            cache_root_from(Some("".into()), Some("/appdata".into()), Some("/h".into())).unwrap(),
            p("/appdata")
        );
        assert_eq!(
            cache_root_from(None, None, Some("/h".into())).unwrap(),
            p("/h").join(".cache")
        );
        let err = cache_root_from(None, None, None).unwrap_err();
        assert!(err.to_string().contains("XDG_CACHE_HOME"));
    }

    #[test]
    fn partial_path_is_process_scoped_and_per_file() {
        let dir = Path::new("/cache");
        let a = partial_path(dir, "model.safetensors");
        assert_eq!(a.parent().unwrap(), dir);
        let name = a.file_name().unwrap().to_str().unwrap();
        assert_eq!(
            name,
            format!("model.safetensors.partial.{}", std::process::id())
        );
        // Two files downloaded by one process still get separate scratch files.
        assert_ne!(a, partial_path(dir, "config.json"));
    }

    #[test]
    fn model_cache_dir_is_revision_scoped() {
        // Env-dependent, so assert the shape rather than the prefix.
        let dir = model_cache_dir().unwrap();
        assert!(dir.ends_with(PathBuf::from(LOCAL_MODEL).join(LOCAL_MODEL_REVISION)));
        assert!(dir.is_absolute());
    }

    #[test]
    fn manifest_accepts_embeddings_subtable() {
        let m = crate::manifest::Manifest::parse(
            "blueprint = \"0.1.0\"\n[knowledge]\nprovider = \"aokf\"\n[knowledge.embeddings]\nprovider = \"openai\"\nmodel = \"text-embedding-3-small\"\n",
        ).unwrap();
        assert_eq!(
            m.capabilities["knowledge"][0]
                .embeddings
                .as_ref()
                .unwrap()
                .provider,
            "openai"
        );
    }
}
