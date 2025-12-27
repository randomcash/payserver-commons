# PayServer Commons

Shared Rust libraries for self-hosted cryptocurrency payment processing.

## Overview

PayServer Commons provides the foundational crates used across all PayServer implementations. It includes authentication, cryptography, and common types for building payment processors that support multiple blockchain networks.

## Crates

| Crate | Description |
|-------|-------------|
| [types](./types/) | Core types, traits, and repository patterns |
| [auth](./auth/) | Authentication: passkeys, Ethereum wallets, BIP39 recovery |
| [crypto](./crypto/) | Cryptographic primitives: Argon2id, AES-256, X25519, Ed25519 |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
types = { git = "https://gitlab.com/random.cash/payserver-commons.git" }
auth = { git = "https://gitlab.com/random.cash/payserver-commons.git" }
crypto = { git = "https://gitlab.com/random.cash/payserver-commons.git" }
```

## Crate Details

### types

Common types and traits shared across all PayServer implementations.

- **Network** - Enum of supported blockchain networks (EVM, Bitcoin, Lightning)
- **Repository traits** - `InvoiceRepository`, `PaymentRepository`, `TokenRepository`
- **Data types** - `InvoiceData`, `PaymentData`, `TokenData`, `Store`
- **Multi-tenant** - `Store`, `StoreId`, `UserId` for multi-merchant support

```rust
use types::{Network, InvoiceStatus, Store, UserId};

// Supported networks
let network = Network::Ethereum;
let network = Network::Bitcoin;

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
use auth::{AuthService, api};

let service = AuthService::new(data_service);
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

## Supported Networks

### EVM Networks
- Ethereum, Polygon, Arbitrum, Optimism, Base
- Avalanche, BNB Smart Chain, zkSync Era, Linea, Scroll, Fantom, Gnosis

### Bitcoin
- Bitcoin Mainnet, Bitcoin Testnet

### Lightning
- Lightning Network

## Used By

- [ethpayserver](https://gitlab.com/random.cash/ethpayserver) - EVM payment processor
- btcpayserver (planned) - Bitcoin payment processor

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
┌─────────────────────────────────────────────────┐
│              Payment Server                      │
│         (ethpayserver, btcpayserver)            │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐       │
│  │  auth   │   │  types  │   │ crypto  │       │
│  └─────────┘   └─────────┘   └─────────┘       │
│       │             │             │             │
│       └─────────────┼─────────────┘             │
│                     │                           │
│              payserver-commons                  │
└─────────────────────────────────────────────────┘
```

## License

MIT License

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
