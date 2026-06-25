# Development Guide

This document outlines the workflow, architecture constraints, and testing procedures for developing the `gitea-plugin-rs` middleware.

## Core Principles

We follow several key principles derived from our architectural skills:
1. **Agentless Middleware**: The system lives outside Jenkins and Gitea.
2. **API Design Standards**: REST callback endpoints (e.g., `/jenkins-status`) must return appropriate semantic HTTP status codes (`200 OK`, `400 Bad Request`, `422 Unprocessable Entity`). See the API Design skill documentation.
3. **Spec-Driven**: The business logic in `bridge-logic` MUST maintain parity with the legacy `jenkinsci/gitea-plugin`.
4. **Test-Driven Development (TDD)**: The `bridge-logic` crate is network-agnostic and should be exhaustively covered by unit tests.

## Crate Layout

The project is organized as a Cargo virtual workspace:

* **`gitea-client`**: Handles all interactions with the Gitea REST API.
    * Uses `serde` with strict `PayloadUser` mapping to prevent deserialization errors on sparse models.
* **`jenkins-client`**: Handles all interactions with the Jenkins REST API.
    * Implements CSRF protection by enabling `cookie_store(true)` in `reqwest`.
* **`bridge-logic`**: The pure domain layer.
    * The `EventProcessor` type transforms Gitea Webhook payloads (`PushEvent`, `PullRequestEvent`) into Jenkins job parameters (`BuildParams`), producing a `JenkinsTriggerRequest`.
* **`webhook-server`**: The Axum HTTP server.
    * Entrypoint for Gitea webhooks and Jenkins status callbacks.

## Adding a New Webhook Event

If you need to support a new Gitea event (e.g., `ReleaseEvent`):

1. **Model**: Add the structs to `crates/gitea-client/src/events.rs`. Use `#[serde(rename_all = "camelCase")]` or explicit renames based on the Gitea API documentation.
2. **Handler**: Add a matching pattern in `crates/webhook-server/src/handlers/gitea_webhook.rs`.
3. **Processor**: Add the transformation logic to `crates/bridge-logic/src/processor.rs`.
4. **Test**: Write a unit test in `processor.rs` proving the parameters are mapped correctly.

## API Endpoint Design

When creating new Axum endpoints, strictly adhere to HTTP verb semantics:
* `POST /jenkins-status`: Triggers a state mutation (updates commit status in Gitea). Returns `200 OK` on success, or `400 Bad Request` if payload validation fails.
* `POST /gitea-webhook/post`: Accepts incoming Gitea hooks.

```rust
// Example Axum signature for a properly designed endpoint
pub async fn handle(
    State(state): State<AppState>,
    Json(payload): Json<MyPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Return explicit status codes
}
```

## Running Tests

### Unit Tests
The pure Rust layer (`EventProcessor` and data structs) must pass all unit tests without needing external services:
```bash
cargo test -p bridge-logic -p gitea-client -p jenkins-client
```

### End-to-End Tests (Playwright)
We use a Docker-compose environment to test the full loop:
```bash
# 1. Start the services
docker compose up -d

# 2. Run Playwright E2E tests
cd e2e-tests
npm install
npx playwright test
```
