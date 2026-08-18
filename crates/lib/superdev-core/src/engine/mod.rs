//! engine/mod.rs — plan every component, apply the result, roll back on
//! failure. The appliers here compute content and make policy calls (skip,
//! drift guard, backup); every actual side effect goes through `tx::Tx`,
//! which journals it so the first failure can unwind the run instead of
//! leaving the repo half-changed. The mise pin phase lives in `pins`, the
//! skill materialiser in `materialise`.

mod apply;
mod materialise;
mod pins;
mod tx;

pub use apply::{ActionOutcome, ApplyResult, ComponentReport, Planned, apply, plan};
