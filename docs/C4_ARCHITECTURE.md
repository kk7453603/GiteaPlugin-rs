# Архитектура в нотации C4 (C4 Model)

В данном документе описана архитектура моста `gitea-plugin-rs` с использованием подхода [C4 Model](https://c4model.com/), визуализированная через Mermaid.

## Уровень 1: System Context (Контекст системы)
Показывает высокоуровневое взаимодействие системы со своими внешними зависимостями и пользователями.

```mermaid
graph LR
    classDef person fill:#083F72,color:#fff,stroke:#062b4f,stroke-width:2px,rx:25,ry:25,font-weight:bold,font-size:16px;
    classDef system fill:#1168BD,color:#fff,stroke:#0b4884,stroke-width:2px,rx:5,ry:5,font-weight:bold,font-size:16px;

    dev["🧑‍💻 Developer"]:::person
    
    subgraph "CI/CD Pipeline"
        gitea["Gitea Server"]:::system
        bridge["Gitea-Jenkins Bridge"]:::system
        jenkins["Jenkins CI"]:::system
    end

    dev -- "git push" --> gitea
    gitea -- "Webhook POST" --> bridge
    bridge -- "Trigger build" --> jenkins
    
    jenkins -. "Job Status" .-> bridge
    bridge -. "Commit Status" .-> gitea
```

## Уровень 2: Container (Контейнеры системы)
Раскрывает архитектуру самого моста, показывая его четыре крейта Rust (`webhook-server`, `bridge-logic`, `jenkins-client`, `gitea-client`), запускаемые в одном процессе/контейнере Docker.

```mermaid
graph TD
    classDef external fill:#999999,color:#fff,stroke:#666666,stroke-width:2px,rx:5,ry:5,font-size:14px;
    classDef container fill:#438DD5,color:#fff,stroke:#2b5a88,stroke-width:2px,rx:5,ry:5,font-weight:bold,font-size:14px;

    gitea["Gitea Server"]:::external
    jenkins["Jenkins CI"]:::external

    subgraph "Gitea-Jenkins Bridge (Rust)"
        direction TB
        webhook["webhook-server (Axum)"]:::container
        logic["bridge-logic (Domain)"]:::container
        client_j["jenkins-client (Reqwest)"]:::container
        client_g["gitea-client (Reqwest)"]:::container
        
        webhook ==>|"Разобранное событие → EventProcessor"| logic
        logic -->|"JenkinsTriggerRequest"| client_j
        webhook -->|"CommitStatus"| client_g
    end

    gitea -->|"POST /gitea-webhook/post"| webhook
    jenkins -->|"POST /jenkins-status"| webhook
    
    client_j ==>|"buildWithParameters"| jenkins
    client_g ==>|"POST /statuses/{sha}"| gitea
```

**Последовательность стадий потока данных:**

1. **Приём вебхука Gitea** — `webhook-server` принимает `POST /gitea-webhook/post` (`handlers/gitea_webhook.rs`).
2. **Валидация HMAC-подписи** — проверка заголовка `X-Gitea-Signature` (секрет `WEBHOOK_SECRET`); тип события читается из `X-Gitea-Event`.
3. **Трансформация события** — `EventProcessor` (`bridge-logic/src/processor.rs`) преобразует событие Gitea в `JenkinsTriggerRequest` с параметрами `BuildParams`.
4. **Триггер сборки Jenkins** — `jenkins-client` вызывает `buildWithParameters`.
5. **Обратный колбэк статуса в Gitea** — Jenkins шлёт `POST /jenkins-status`; `webhook-server` (`handlers/jenkins_webhook.rs`) маппит статус сборки в `CommitStatus` и через `gitea-client` отправляет `POST /statuses/{sha}`.

## Уровень 3: Component (Компоненты)
Демонстрирует внутреннюю структуру `EventProcessor` (`bridge-logic/src/processor.rs`) и расположение маппинга статусов сборки.

```mermaid
graph TD
    classDef container fill:#438DD5,color:#fff,stroke:#2b5a88,stroke-width:2px,rx:5,ry:5,font-size:14px;
    classDef comp fill:#85BBF0,color:#000,stroke:#5b80a4,stroke-width:2px,rx:10,ry:10,font-weight:bold,font-size:14px;

    webhook["webhook-server"]:::container
    jenkins_client["jenkins-client"]:::container

    subgraph "bridge-logic (EventProcessor)"
        direction TB
        processor["EventProcessor::process"]:::comp
        push_proc["process_push_event"]:::comp
        pr_proc["process_pull_request_event"]:::comp
    end

    subgraph "webhook-server (handlers/jenkins_webhook.rs)"
        direction TB
        status_map["Маппинг статусов сборки → CommitStatus"]:::comp
    end

    webhook ==>|"GiteaEvent"| processor
    
    processor -->|"PushEvent"| push_proc
    processor -->|"PullRequestEvent"| pr_proc
    
    push_proc -.->|"JenkinsTriggerRequest (BuildParams)"| jenkins_client
    pr_proc -.->|"JenkinsTriggerRequest (BuildParams)"| jenkins_client
```

> **Примечание:** Маппинг статусов сборки Jenkins в `CommitStatus` Gitea выполняется **не** в `bridge-logic`, а в обработчике `webhook-server/src/handlers/jenkins_webhook.rs` (функция `handle`), который вызывается на маршруте `POST /jenkins-status`.

> **Примечание:** Уровень 4 (Code) в C4 обычно не рисуется, так как он слишком детализирован, и его роль выполняют UML диаграммы классов или сам исходный код. В Rust эту роль отлично выполняет `cargo doc`.
