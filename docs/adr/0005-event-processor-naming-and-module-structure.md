# ADR-0005: EventProcessor Naming and Crate Module Structure

**Date**: 2026-06-22
**Status**: accepted
**Deciders**: AI Agent, User

## Context

The architecture documentation (`AGENTS.md`, `README.md`, `PLAN.md`, `docs/C4_ARCHITECTURE.md`) and the actual source tree (`crates/`) had drifted apart. The documents referred to the event transformation component as a "mapper" (e.g. `bridge-logic/src/mapper.rs`, "Push Mapper", "PR Mapper", "Status Mapper", "pass to bridge-logic mapper"), while the implementation has never contained a `mapper` module or file.

The verified state of the code is:

- The event transformation logic lives in `bridge-logic/src/processor.rs` and is exposed by the `EventProcessor` type (with a `process` method). There is no `mapper.rs`.
- The HTTP handlers live in the directory module `webhook-server/src/handlers/` (`mod.rs`, `gitea_webhook.rs`, `jenkins_webhook.rs`), not in a single `handlers.rs` file.
- The `jenkins-client` crate contains only `client.rs` and `lib.rs`; there is no `models.rs`.
- The `gitea-client` crate contains `client.rs`, `events.rs`, `lib.rs`, `models.rs` — and `events.rs` was undocumented.
- Mapping Jenkins build statuses back to Gitea happens in `webhook-server/src/handlers/jenkins_webhook.rs`, not inside `bridge-logic`.

Using "mapper" as a component, module, or file name in the documentation was misleading: it pointed readers and AI agents to files that do not exist and implied an architecture that does not match reality.

## Decision

We fix the canonical naming and module structure as follows, and treat the source tree as the single source of truth.

1. The component that transforms Gitea webhook events into Jenkins build triggers is named **`EventProcessor`** and lives in `bridge-logic/src/processor.rs`. The term "mapper" SHALL NOT be used as the name of any component, module, or file across the documentation.

2. The actual module structure of the four workspace crates is recorded as canonical:

   ```text
   crates/
   ├── gitea-client/src/    → client.rs, events.rs, lib.rs, models.rs
   ├── jenkins-client/src/  → client.rs, lib.rs
   ├── bridge-logic/src/    → lib.rs, processor.rs
   └── webhook-server/src/  → main.rs, handlers/{mod.rs, gitea_webhook.rs, jenkins_webhook.rs}
   ```

3. The key event-transformation types keep the exact spelling used in the code: `EventProcessor`, `JenkinsTriggerRequest`, `BuildParams`.

4. Status mapping (Jenkins → Gitea) is owned by `webhook-server/src/handlers/jenkins_webhook.rs`, and documentation SHALL place it there rather than in `bridge-logic`.

## Alternatives Considered

### Alternative 1: Rename the code to match the documentation ("mapper")
- **Pros**: No documentation changes; preserves the historical "mapper" wording.
- **Cons**: Requires renaming working, tested executable code (`processor.rs`, `EventProcessor`) purely to match stale prose. Higher risk of regressions for zero behavioural benefit.
- **Why not**: The code is the source of truth and is already correct and tested. Documentation must follow the code, not the reverse.

### Alternative 2: Allow both "mapper" and "EventProcessor" as synonyms
- **Pros**: Tolerates existing references without immediate cleanup.
- **Cons**: Two names for one component reintroduce exactly the ambiguity this cleanup removes, and "mapper" continues to point at non-existent files.
- **Why not**: Violates the single-canonical-name principle for terminology and keeps dangling file references alive.

## Consequences

### Positive
- Documentation, doc-comments, and architecture diagrams now reference real files and a single, unambiguous name (`EventProcessor`).
- New contributors and AI agents are no longer directed to non-existent files (`mapper.rs`, `handlers.rs`, `jenkins-client/models.rs`).
- `events.rs` in `gitea-client` is acknowledged as part of the module structure.

### Negative
- Existing external notes or branches that still say "mapper" will be out of sync until updated.

### Risks
- Future code reorganisation could drift from this record again.
- Mitigation: any change to the crate set, the key type names (`EventProcessor`, `JenkinsTriggerRequest`, `BuildParams`), or the data-flow stages must be captured in a follow-up ADR and reflected in `docs/adr/README.md`.
