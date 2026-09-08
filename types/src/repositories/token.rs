//! Token repository traits.
//!
//! This module provides generic token storage traits that work across all payserver
//! implementations. Token types and validation are network-specific and should be
//! defined in each payserver's crate (e.g., ERC20/ERC721/ERC1155 in evm crate).

use async_trait::async_trait;

use super::RepositoryResult;
use crate::types::{ChainId, TokenData};

/// Query parameters for listing tokens.
#[derive(Debug, Clone)]
pub struct TokenQueryParams {
    pub token_type: Option<String>,
    pub chain_id: Option<ChainId>,
    pub enabled: Option<bool>,
    pub symbol: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for TokenQueryParams {
    fn default() -> Self {
        Self {
            token_type: None,
            chain_id: None,
            enabled: None,
            symbol: None,
            limit: 100,
            offset: 0,
        }
    }
}

impl TokenQueryParams {
    pub fn with_token_type(mut self, token_type: impl Into<String>) -> Self {
        self.token_type = Some(token_type.into());
        self
    }

    pub fn with_chain(mut self, chain_id: ChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = offset;
        self
    }
}

/// Read operations for tokens.
#[async_trait]
pub trait TokenReader: Send + Sync {
    /// Get a token by ID.
    async fn get(&self, id: i64) -> RepositoryResult<Option<TokenData>>;

    /// Get a token by network and contract address.
    async fn get_by_address(
        &self,
        chain_id: &ChainId,
        address: &str,
    ) -> RepositoryResult<Option<TokenData>>;

    /// Find a token by network and symbol.
    /// Note: If multiple tokens have the same symbol on a network, returns the first match.
    async fn find_by_symbol(
        &self,
        chain_id: &ChainId,
        symbol: &str,
    ) -> RepositoryResult<Option<TokenData>>;

    /// Query tokens with filters and pagination.
    /// Returns (total_count, tokens).
    async fn query(&self, params: &TokenQueryParams) -> RepositoryResult<(i64, Vec<TokenData>)>;

    /// Get all enabled tokens for a specific network.
    async fn get_enabled_for_chain(&self, chain_id: &ChainId) -> RepositoryResult<Vec<TokenData>>;
}

/// Write operations for tokens.
#[async_trait]
pub trait TokenWriter: Send + Sync {
    /// Insert a new token. Returns the assigned ID.
    async fn insert(&self, token: &TokenData) -> RepositoryResult<i64>;

    /// Update an existing token.
    async fn update(&self, token: &TokenData) -> RepositoryResult<()>;

    /// Delete a token by ID.
    async fn delete(&self, id: i64) -> RepositoryResult<()>;

    /// Enable or disable a token.
    async fn set_enabled(&self, id: i64, enabled: bool) -> RepositoryResult<()>;
}

/// Combined token repository with full read/write access.
pub trait TokenRepository: TokenReader + TokenWriter {}

/// Blanket implementation for any type implementing both Reader and Writer.
impl<T: TokenReader + TokenWriter> TokenRepository for T {}
