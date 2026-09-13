//! What a plugin is allowed to do.

use serde::{Deserialize, Serialize};

/// Whether a plugin emits actions, filters/vetoes them, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Action,
    Filter,
    Both,
}

impl PluginKind {
    /// Whether this kind acts as a filter, and therefore needs a
    /// `failure_mode` to define what happens when it cannot run.
    pub fn is_filter(self) -> bool {
        matches!(self, PluginKind::Filter | PluginKind::Both)
    }
}
