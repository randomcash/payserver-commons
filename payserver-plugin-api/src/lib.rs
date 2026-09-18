//! The plugin manifest contract: parsing, id validation, and version
//! negotiation primitives shared by the plugin host and, eventually,
//! plugins themselves.
//!
//! This crate has no wasmtime dependency and instantiates nothing: the
//! runtime that instantiates a module, dispatches actions and filters, and
//! bounds and contains a call lives in the host, because only the host knows
//! what it is willing to run. What lives here is
//! everything that can be decided before a plugin's code ever runs: is the
//! manifest well-formed, is its id safe to route to, and does its declared
//! `ethpayserver` requirement match a candidate host version. The host-side
//! load-time gate that uses these types — refusing an incompatible plugin
//! and rejecting a duplicate id — lives in ethpayserver, since only the host
//! knows its own version and which plugins are already loaded.
//!
//! # Version negotiation
//!
//! The host presents itself as a dependency (`ethpayserver:^1.2.0`) the same
//! way one plugin would depend on another, so [`Dependency`] is the single
//! mechanism for both: a semver requirement matched against a semver
//! version, with [`Dependency::is_satisfied_by`] doing the check.

mod dependency;
mod failure_mode;
mod id;
mod kind;
mod manifest;

pub use dependency::{Dependency, InvalidDependency};
pub use failure_mode::FailureMode;
pub use id::{InvalidPluginId, PluginId};
pub use kind::PluginKind;
pub use manifest::{InvalidManifest, Manifest};

pub use semver::{Version, VersionReq};
