# PayServer Commons

Shared libraries for the random.cash payment processing platform.

## Crates

| Crate | Description |
|-------|-------------|
| [auth](./auth/) | User authentication: passkeys, Ethereum wallets, BIP39 recovery |
| [crypto](./crypto/) | Cryptographic primitives: Argon2id, AES-256, X25519, Ed25519 |
| [types](./types/) | Common types and traits shared across all payservers |

## Usage

These crates are used by:
- [ethpayserver](https://gitlab.com/random.cash/ethpayserver) - EVM payment processor
- [btcpayserver](https://gitlab.com/random.cash/btcpayserver) - Bitcoin payment processor (planned)

## Development

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Check for issues
cargo clippy
```

## License

MIT License
