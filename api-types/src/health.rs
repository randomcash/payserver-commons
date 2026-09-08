//! Health and readiness.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use types::ChainId;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Chain health information for a single chain.
///
/// Two audiences, one shape. Whether a chain is up is something every merchant
/// needs - a dashboard that cannot say "payments are not being detected right
/// now" is worse than no dashboard. How far behind the monitor is, and why a
/// connection failed, is operational detail that belongs to admins.
///
/// So the detail fields are `Option` and simply absent for everyone else,
/// rather than there being two response types to keep in step. See
/// [`Self::redact`].
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainHealthInfo {
    /// Chain ID (EIP-155).
    pub chain_id: ChainId,
    /// Human-readable chain name.
    pub chain_name: String,
    /// Connection status. Public form is one of `connected`, `connecting`,
    /// `disconnected`, `failed`; admins additionally get `failed: {reason}`.
    pub status: String,
    /// Current block number on chain. Admin only.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub current_block: Option<u64>,
    /// Last block processed by the monitor. Admin only.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub last_processed_block: Option<u64>,
    /// Number of addresses being watched. Admin only.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub watched_addresses: Option<usize>,
    /// Overall health status.
    #[serde(default)]
    pub is_healthy: bool,
}

/// Chains health response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainsHealthResponse {
    /// Health information for each monitored chain.
    pub chains: Vec<ChainHealthInfo>,
    /// Whether all chains are healthy.
    #[serde(default)]
    pub all_healthy: bool,
    /// Data freshness - whether health data is recent (updated within 60s).
    #[serde(default)]
    pub data_fresh: bool,
}

/// Health check response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: String,

    /// Service version.
    pub version: String,

    /// Build commit SHA (short) baked in at compile time.
    pub build_sha: String,

    /// Database connectivity.
    pub database: bool,

    /// Redis connectivity (for monitor service).
    /// None if Redis is not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,
}

/// Readiness probe response (returned on 503).
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessResponse {
    /// "ready" or "not_ready".
    pub status: String,
    /// Names of failing dependencies (e.g. "postgres", "redis", "rpc:56").
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failing: Vec<String>,
}

/// Deep health diagnostic response.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepHealthResponse {
    /// Build commit SHA (short) baked in at compile time.
    pub build_sha: String,
    /// Service version from Cargo.toml.
    pub version: String,
    /// Postgres health.
    pub postgres: DependencyHealth,
    /// Redis health.
    pub redis: DependencyHealth,
    /// Per-chain RPC health keyed by chain_id.
    pub rpcs: HashMap<String, RpcHealth>,
    /// EVM monitor liveness.
    pub monitor: MonitorHealth,
}

/// Status of a single dependency in the deep health check.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// "ok" or "error".
    pub status: String,
    /// Latency in milliseconds.
    pub latency_ms: u64,
    /// Error message if status is "error".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Monitor liveness status in the deep health check.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorHealth {
    /// "ok" or "error".
    pub status: String,
    /// Whether chain health data in Redis is fresh (updated within 60 s).
    #[serde(default)]
    pub data_fresh: bool,
}

/// RPC chain status in the deep health check.
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcHealth {
    /// "ok" or "error".
    pub status: String,
    /// Latency in milliseconds (time to read health from Redis, not RPC RTT).
    pub latency_ms: u64,
    /// Last block number reported by the monitor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_block: Option<u64>,
    /// Error message if status is "error".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ChainHealthInfo {
    /// Strip everything a non-admin should not see.
    ///
    /// `is_healthy`, `status` and the chain's identity stay: that is the "on or
    /// off" answer the dashboard exists to show. Block heights, the watched
    /// address count and the failure reason go - the reason especially, since
    /// an RPC error string routinely carries the provider and the endpoint.
    pub fn redact(mut self) -> Self {
        self.current_block = None;
        self.last_processed_block = None;
        self.watched_addresses = None;
        if let Some(bare) = self.status.split(':').next() {
            self.status = bare.trim().to_string();
        }
        self
    }
}
