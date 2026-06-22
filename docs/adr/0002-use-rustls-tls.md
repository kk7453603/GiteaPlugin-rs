# 0002. Architecture: Use rustls-tls over default OpenSSL

## Status
Accepted

## Context
Our Docker multi-stage build using `debian:bookworm-slim` for compiling the Rust application (`reqwest` with `default-tls`) failed due to missing `pkg-config` and `libssl-dev` C-dependencies required by the `openssl-sys` crate. 

## Decision
Instead of adding heavy system-level C dependencies to our Docker build and runtime environments, we configured `reqwest` in our workspace `Cargo.toml` to disable default features and explicitly use the `rustls-tls` feature:
```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

## Consequences
- **Positive:** The project can be built entirely with the standard Rust toolchain. Dockerfiles are simplified, and the resulting binaries are fully statically linked regarding TLS.
- **Positive:** Reduces the attack surface and potential system library conflicts in deployment environments.
- **Negative:** `rustls` does not support all legacy cryptographic algorithms that OpenSSL supports, though this is rarely an issue for modern API communications (Gitea and Jenkins).
