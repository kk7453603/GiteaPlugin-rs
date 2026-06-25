# Project Guidelines: Gitea-Jenkins Bridge (gitea-plugin-rs)

These are the rules and guidelines for working on the `gitea-plugin-rs` project, a Rust-based middleware replacing the Java-based `jenkinsci/gitea-plugin`.

## Architecture Overview

**Tech Stack:**
- **Language**: Rust (edition 2021)
- **Web Framework**: `axum` (with `tokio` asynchronous runtime)
- **HTTP Client**: `reqwest`
- **Serialization**: `serde` and `serde_json`
- **Security**: HMAC signature validation (`hmac`, `sha2`)

**Services/Crates (Workspace):**
```
┌─────────────────────────────────────────────────────────────┐
│                       webhook-server                        │
│  Axum API, HMAC validation, status callbacks                │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                        bridge-logic                         │
│  Event transformation: Gitea Webhook -> Jenkins SCM Event   │
└─────────────────────────────────────────────────────────────┘
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
        ┌──────────────┐                ┌──────────────┐
        │ jenkins-client│                │ gitea-client │
        │ REST API calls│                │ REST API calls│
        │ CSRF handling │                │ Data Models  │
        └──────────────┘                └──────────────┘
```

---

## File Structure

```
gitea-plugin-rs/
├── Cargo.toml                  # Virtual workspace definition
├── crates/
│   ├── gitea-client/           # Gitea REST API models and client
│   │   ├── src/lib.rs          # Crate root & re-exports
│   │   ├── src/client.rs       # Reqwest-based API client
│   │   ├── src/events.rs       # Gitea webhook event payloads
│   │   └── src/models.rs       # Serde structs for Gitea objects
│   ├── jenkins-client/         # Jenkins REST API client
│   │   ├── src/lib.rs          # Crate root & re-exports
│   │   └── src/client.rs       # Crumb fetching and job triggering
│   ├── bridge-logic/           # Core event transformation logic
│   │   ├── src/lib.rs          # Crate root & re-exports
│   │   └── src/processor.rs    # EventProcessor: transforms Gitea events to Jenkins payloads
│   └── webhook-server/         # Axum server entrypoint
│       ├── src/main.rs         # Setup and routing
│       └── src/handlers/       # Webhook endpoints & HMAC check
│           ├── mod.rs          # Handlers module root
│           ├── gitea_webhook.rs   # Gitea webhook endpoint
│           └── jenkins_webhook.rs # Jenkins status callback endpoint
└── .agents/                    # Agent customizations and rules
```

---

## Code Patterns

### Data Models (Serde)

Use strict typing and `#[serde(rename_all = "...")]` to map JSON fields precisely:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GiteaRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub ssh_url: String,
    pub clone_url: String,
    pub default_branch: String,
    #[serde(default)]
    pub private: bool,
}
```

### Async HTTP Clients (Reqwest)

Use centralized clients that handle authentication and returning standard `Result` types:

```rust
pub async fn trigger_job(&self, job_name: &str, params: &HashMap<String, String>) -> Result<(), reqwest::Error> {
    // 1. Fetch Crumb
    // 2. POST to /job/{job_name}/buildWithParameters
}
```

### Axum Handlers

Handlers should extract state, validate payloads, and delegate to the `bridge-logic` crate:

```rust
pub async fn gitea_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    // 1. Validate HMAC signature using x-gitea-signature
    // 2. Parse x-gitea-event header
    // 3. Deserialize body
    // 4. Transform the event via bridge-logic EventProcessor
    // 5. Trigger Jenkins Client
}
```

---

## Agent Behavioral Rules

1. **Репликация Бизнес-логики (Business Logic Parity)**: The Rust business logic must exactly match the legacy Java `jenkinsci/gitea-plugin`. Never guess the logic; look at the legacy Java code to verify parameter names and edge cases.
2. **Spec Driven Development**: Always exhaustively study the legacy Java code and business logic before creating a full replication plan.
3. **Edge Cases**: Address edge cases as they are discovered during the translation process. 
4. **Модели агентов (Model Routing)**: 
   - Historically: Coordinator is `claude opus 4.6`, Executor is situational (complex: `opus`, easy: `gemini pro`).
   - *Current Fallback Constraint*: When Claude API limits are reached, switch to `gemini 3.1 pro high` for both planning and implementation.
5. **No Visual UI Ports**: UI configuration (like `GiteaServers` or `GiteaAvatar` in Jenkins) should NOT be ported. Use Environment Variables for secrets and configuration.
6. **Artifacts Publication**: `GiteaAssetPublisher` functionality (uploading workspace artifacts) is out-of-scope for the bridge middleware and should be executed natively in the `Jenkinsfile` via `curl` or `HTTP Request Plugin`.

---

## Testing Requirements

- **Unit Testing**: Test the `EventProcessor` in `bridge-logic` thoroughly. The `EventProcessor` should not perform network requests.
- **Integration Testing**: Use mock servers (e.g., `wiremock`) to test `jenkins-client` and `gitea-client`.
- **Run Tests**: `cargo test --all`

---

## Deployment Workflow

- Compile using `cargo build --release`.
- Configure via `.env` or environment variables (имена и значения по умолчанию из `webhook-server/src/main.rs`):
  - `SERVER_PORT` (default `3000`)
  - `JENKINS_URL` (default `http://localhost:8080`)
  - `JENKINS_USER` (default `admin`)
  - `JENKINS_TOKEN` (default `token`)
  - `JENKINS_JOB` (default `gitea-trigger-job`)
  - `GITEA_URL` (default `http://localhost:3000`)
  - `GITEA_TOKEN` (default `token`)
  - `WEBHOOK_SECRET` (опциональная, без значения по умолчанию; если не задана — проверка HMAC-подписи отключена)

---

## Безопасность и защита рабочей станции (DevSecOps)

Так как разработка ведется на корпоративном/локальном ноутбуке, агенты обязаны строго соблюдать границы и защищать хост-систему:

1. **Защита системы (Safety Guard):**
   - Агентам **КАТЕГОРИЧЕСКИ ЗАПРЕЩАЕТСЯ** выполнять деструктивные команды (например, `rm -rf`, `git reset --hard`) на хостовой машине.
   - Использовать навык `safety-guard` для предотвращения нежелательных действий.

2. **Безопасность кода (Security Review):**
   - Проверка входящих вебхуков (HMAC подпись) и работа с секретами (токены Jenkins/Gitea) должны проходить через чек-лист навыка `security-review`.
   - Запрещено хардкодить токены. Все секреты передаются только через переменные окружения.

3. **Защита от вредоносных зависимостей (Supply Chain Security):**
   - Каждая новая зависимость в `Cargo.toml` должна быть обоснована. Обязателен запуск `cargo audit`.

---

## Навыки агентов (Agent Skills)

Все ИИ-агенты, работающие в этом репозитории, **обязаны** использовать следующие скиллы (skills) в зависимости от контекста задачи:

### 🛡️ Безопасность и качество (Security & Quality)
- **`security-review`**: При добавлении валидации вебхуков, работе со входящими HTTP запросами Axum или исходящими запросами Reqwest.
- **`security-scan`**: Для проверки конфигурации агентов.
- **`safety-guard`**: Защита хост-системы от случайных деструктивных команд.
- **`quality-nonconformance`**: Для выявления отклонений в коде от принятых стандартов качества.

### 🦀 Специфично для Rust
- **`rust-patterns`**: Применение паттернов проектирования, специфичных для Rust.
- **`rust-async-patterns`**: Строгие правила работы с `tokio`, написания обработчиков `axum` и использования асинхронного клиента `reqwest`.
- **`rust-testing`**: Следование стандартам написания тестов (unit-тесты для `EventProcessor` в `bridge-logic`, mock-объекты для `wiremock`).

### 📐 Архитектура и процессы
- **`architecture-decision-records`**: Любое значимое изменение в архитектуре моста требует создания ADR.
- **`tdd-workflow`**: При написании бизнес-логики (`bridge-logic`) используйте Test-Driven Development. Сначала тест — потом реализация.
- **`api-design`**: При проектировании REST API (например, callback-эндпоинтов от Jenkins).
- **`git-workflow`**: Соблюдение правил создания коммитов и ведения веток.

### 🧠 ИИ и Память
- **`ck`**: Для сохранения долговременного контекста проекта между сессиями.
- **`agentic-engineering`**: При маршрутизации задач между моделями.

### 🔍 Отладка, QA и Верификация (Debugging & QA)
- **`verification-loop`**: Использование тестов и `cargo clippy` перед коммитами.
- **`troubleshooting`**: Системный подход к решению проблем со сборкой или сетью при интеграции Jenkins/Gitea.

### 🤖 LLM-специфичные Директивы (Qwen3.5 и другие)
- **Защита от галлюцинаций**: Строго сверяйтесь с документацией и избегайте использования несуществующих методов в крейтах.
