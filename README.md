# PayServer Commons

Shared Rust libraries for self-hosted cryptocurrency payment processing.

## Overview

PayServer Commons provides the foundational crates used across all PayServer implementations. It includes authentication, cryptography, and common types for building payment processors that support multiple blockchain networks.

## Crates

| Crate | Description |
|-------|-------------|
| [types](./types/) | Core types, traits, and repository patterns |
| [api-types](./api-types/) | The PayServer HTTP contract: request/response shapes, shared by every server and the client |
| [auth](./auth/) | Authentication: passkeys, Ethereum wallets, BIP39 recovery |
| [crypto](./crypto/) | Cryptographic primitives: Argon2id, AES-256, X25519, Ed25519 |
| [rates](./rates/) | Exchange rate providers and currency utilities |
| [scrub](./scrub/) | Secret/PII redaction for error reports, shared by every payserver and the client |
| [payserver-plugin-api](./payserver-plugin-api/) | Plugin manifest contract: parsing, id validation, version negotiation |
| [payserver-plugin-host](./payserver-plugin-host/) | The wasmtime plugin host: load-time gate, instantiation, bounded calls, page rendering |
| [ui-kit](./ui-kit/) | Shared Leptos UI components for random.cash frontends |

## Installation

Pin by `rev`, not by branch — see [Which version you build against](https://github.com/randomcash/ethpayserver#which-version-you-build-against)
in ethpayserver's README for why. Add to your `Cargo.toml`:

```toml
[dependencies]
types = { git = "https://github.com/randomcash/payserver-commons.git", rev = "<commit-sha>" }
auth = { git = "https://github.com/randomcash/payserver-commons.git", rev = "<commit-sha>" }
crypto = { git = "https://github.com/randomcash/payserver-commons.git", rev = "<commit-sha>" }
```

## Crate Details

### types

Common types and traits shared across all PayServer implementations.

- **ChainId** - CAIP-2 chain identity (`eip155:1`, `tron:728126428`, ...), open
  by construction so a new chain never requires editing this crate
- **Repository traits** - around twenty, including `InvoiceRepository`, `PaymentRepository`, `TokenRepository`
- **Data types** - `InvoiceData`, `PaymentData`, `TokenData`, `Store`
- **Multi-tenant** - `Store`, `StoreId`, `UserId` for multi-merchant support

```rust
use types::{ChainId, InvoiceStatus, Store, UserId};

// Chain identity
let ethereum = ChainId::evm(1);                        // eip155:1
let tron_mainnet = ChainId::parse("tron:728126428")?;

// Multi-tenant stores
let store = Store::new("My Shop", UserId::new());
```

**Features:**
- `openapi` - Adds `utoipa::ToSchema` derives for OpenAPI documentation

### auth

Secure, passwordless authentication with multiple methods.

- **Passkeys** - WebAuthn/FIDO2 phishing-resistant authentication
- **Ethereum Wallets** - EIP-191 signature-based authentication
- **BIP39 Recovery** - Mnemonic phrase account recovery
- **Role-Based Access** - Server and store-level permissions
- **Zero-Knowledge** - Server stores only encrypted data it cannot decrypt

```rust
use auth::{WebAuthnAuthService, api};
use std::sync::Arc;

let service = Arc::new(WebAuthnAuthService::new(Arc::new(data_service)));
let router = api::router(api::AuthState::new(service));
// Mount at /auth
```

**Store Roles:**
| Role | Description |
|------|-------------|
| Owner | Full store access |
| Manager | Manage settings, view all data |
| Employee | Create and view invoices |
| Guest | Read-only access |

### crypto

Bitwarden-style client-side encryption primitives.

- **KDF** - Argon2id password hashing, HKDF-SHA256 key derivation
- **Symmetric** - AES-256-CBC + HMAC-SHA256 authenticated encryption
- **Asymmetric** - X25519 key exchange, Ed25519 signatures
- **Mnemonic** - BIP39 recovery phrase generation

```rust
use crypto::{kdf, symmetric, types::KdfParams};

// Derive key from password
let params = KdfParams::new_random();
let master_key = kdf::derive_master_key(b"password", "user@example.com", &params)?;
let stretched = kdf::stretch_master_key(&master_key)?;

// Encrypt/decrypt
let encrypted = symmetric::encrypt(b"secret", &stretched)?;
let decrypted = symmetric::decrypt(&encrypted, &stretched)?;
```

## Supported Chain Families

`ChainId` (see [types](./types/)) is a [CAIP-2](https://standards.chainagnostic.org/CAIPs/caip-2)
string rather than a closed list, so a payserver — or a plugin — can support a
new chain without a change here. Five namespaces are modeled today: `eip155`
(Ethereum and other EVM chains), `tron`, `solana`, `monero`, `bip122`
(Bitcoin). Which chains any given payserver actually implements is a separate
question — see that payserver's own README.

## Used By

- [ethpayserver](https://github.com/randomcash/ethpayserver) - EVM (`eip155`) payment processor. Its
  frontend, [payserver-client](https://github.com/randomcash/payserver-client), is built to serve any
  payserver on any of the chain families above, not only this one.

## Development

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

### Requirements

- Rust 1.90+ (edition 2024)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          Payment Server                          │
│                (ethpayserver, and any future payserver)          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────┐ ┌───────────┐ ┌──────┐ ┌───────┐ ┌───────┐ ┌──────┐│
│  │  auth  │ │ api-types │ │ types│ │ crypto│ │ rates │ │ scrub││
│  └────────┘ └───────────┘ └──────┘ └───────┘ └───────┘ └──────┘│
│       │           │           │         │         │        │   │
│       └───────────┴───────────┴─────────┴─────────┴────────┘   │
│                                 │                                │
│                          payserver-commons                       │
│         (also: payserver-plugin-api, payserver-plugin-host,      │
│          ui-kit — see Crates above for the full list)            │
└─────────────────────────────────────────────────────────────────┘
```

## License

MIT License

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
