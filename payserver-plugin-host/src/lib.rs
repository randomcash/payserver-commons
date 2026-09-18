//! The wasmtime plugin host, shared by every payserver.
//!
//! A plugin is installed against *a payserver*, not against ETHPayserver
//! specifically: the same artifact should load on an EVM, Solana or Monero
//! instance, because nothing a plugin does — declaring a manifest, being
//! gated on a host version, running bounded wasm, rendering a page — is about
//! chains at all. Keeping this crate free of chain vocabulary is what makes
//! that true rather than aspirational.
//!
//! ## What is here, and what deliberately is not
//!
//! Here: the load-time gate ([`PluginRegistry`]), the wasmtime runtime
//! ([`PluginEngine`], [`PluginInstance`]), dispatch with deadlines and
//! disable-on-repeated-failure ([`PluginHost`]), the artifact store and its
//! digest ([`PluginArtifacts`]), and the static page descriptor and renderer
//! ([`PageElement`], [`PageHost`]).
//!
//! Not here: anything that reaches into a particular server. Creating an
//! invoice, reading the merchant directory and filtering invoice creation are
//! *capabilities* a host offers, and each payserver implements them against
//! its own data layer. They stay on the other side of this boundary
//! deliberately — the moment one of them is implemented in here, this crate
//! knows what an invoice is on one chain and the portability is gone.
//!
//! ## The calling convention
//!
//! Host-defined and not yet part of `payserver-plugin-api`: a plugin exports
//! `memory`, an `alloc(len: i32) -> i32` bump allocator the host uses to
//! place its JSON argument, and one function per call name with signature
//! `(ptr: i32, len: i32) -> i64`, the answer packed as `(ptr << 32) | len`.

mod artifacts;
mod error;
mod host;
pub mod page;
mod pages;
mod registry;
mod runtime;

pub use artifacts::{ArtifactError, PluginArtifacts, digest};
pub use error::PluginLoadError;
pub use host::{FilterOutcome, PluginHost, PluginHostError, PluginStatusSnapshot};
pub use page::{PageElement, Viewer};
pub use pages::{PageError, PageHost, PageRenderer};
pub use registry::{PluginRegistry, host_version};
pub use runtime::{
    HOST_MODULE, PluginCallError, PluginEngine, PluginHostCalls, PluginInstance, PluginWasmError,
};

// Re-exported so a payserver implementing host capabilities does not have to
// depend on `payserver-plugin-api` separately just to name a `PluginId`.
pub use payserver_plugin_api::{
    Dependency, FailureMode, InvalidManifest, InvalidPluginId, Manifest, PluginId, PluginKind,
    Version, VersionReq,
};
