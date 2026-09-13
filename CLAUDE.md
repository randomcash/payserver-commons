# payserver-commons

Shared crates for the random.cash payservers: types, api-types, auth, crypto,
rates, scrub, ui-kit.

## This repository is public

No session URLs in commits or PR bodies, no secrets, and never a reproduction
for an unfixed vulnerability.

## Nothing here ships until a consumer bumps its pin

Consumers pin this repository **by revision**, not by branch:

```toml
types = { git = "…/payserver-commons.git", rev = "e6c4ddb…" }
```

So merging here changes nothing downstream. A change reaches `ethpayserver` or
`payserver-client` only when that repo bumps the `rev` and runs `cargo update`.
Plan for two PRs, and expect the consumer half to be where the breakage shows.

This is deliberate: a branch pin made rebuilding last week's commit silently pick
up this week's commons, and a breaking change took testnet down the moment it
merged.

## `api-types` and `ui-kit` compile into the browser

They are part of the WASM bundle. Before adding a dependency to either, ask
whether it builds for `wasm32-unknown-unknown` — anything C-linked, anything
needing system entropy, anything with a JS shim is a problem.

**The obvious check is currently blind.** `cargo check -p <crate> --target
wasm32-unknown-unknown` fails on most crates here regardless of your change,
because `uuid` requires an explicit randomness feature for that target and the
real client supplies it through feature unification. So the check cannot
distinguish "your dependency is wasm-hostile" from "uuid". To get a real signal,
temporarily add `"js"` to that crate's `uuid` features, check, then revert.

Pure-Rust, `no_std`-capable crates (the RustCrypto family, for instance) are
safe. Prefer them.

## Reachability is not the same as `pub`

An item can be `pub` in its module, absent from the crate root's re-export list,
and therefore unreachable from outside — while every in-crate test passes,
because those call it by its in-crate path.

This has shipped. When adding a public item, add a test that names it **through
the crate root** exactly as a consumer would.

## The gate

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --lib --all-features
```

Unlike `ethpayserver`, `--all-features` is correct here — CI uses it.

## ui-kit and the client share a vocabulary

ui-kit components emit `ps-` prefixed classes. The client has adopted those names
for buttons and cards, but **ui-kit does not yet own the styling** — the client's
stylesheet still provides it. So a class emitted here with no rule anywhere
renders unstyled, and nothing in either repo's tests notices: layout tests assert
geometry, not colour, and e2e locates elements by class, which still match.

If you add a component, add its rules to `ui-kit/styles/ui-kit.css` in the same
change.

## Conventions that bite

- **`git grep`, not bare `grep`** — `grep` is `ugrep` here and honours
  `.gitignore`, so a recursive grep silently skips files.
- **Never `git add -A`** — check for stray untracked files and add paths
  explicitly.
- **A test that cannot fail is worse than no test.** Break the thing it covers
  and confirm it goes red before trusting it.
