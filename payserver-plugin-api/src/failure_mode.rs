//! What a filter does when it cannot be run.

use serde::{Deserialize, Serialize};

/// What happens to the action a filter was meant to inspect when the filter
/// itself fails to run (traps, times out, or is missing entirely).
///
/// `failure_mode` is required in the manifest for any [`crate::PluginKind`]
/// that filters. Omitting it is not the same as choosing a default —
/// [`crate::Manifest::from_str`] treats an absent value as [`FailureMode::Closed`],
/// since letting an action through that nothing vetted is the worse of the
/// two ways to get this wrong silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureMode {
    /// The action proceeds as if the filter had allowed it.
    Open,
    /// The action is blocked.
    Closed,
}
