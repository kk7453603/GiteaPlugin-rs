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
Раскрывает архитектуру самого моста, показывая его основные контейнеры (в нашем случае это логические крейты Rust, запускаемые в одном процессе/контейнере Docker).

```mermaid
graph TD
    classDef external fill:#999999,color:#fff,stroke:#666666,stroke-width:2px,rx:5,ry:5,font-size:14px;
    classDef container fill:#438DD5,color:#fff,stroke:#2b5a88,stroke-width:2px,rx:5,ry:5,font-weight:bold,font-size:14px;

    gitea["Gitea Server"]:::external
    jenkins["Jenkins CI"]:::external

    subgraph "Gitea-Jenkins Bridge (Rust)"
        direction TB
        webhook["Webhook Server (Axum)"]:::container
        logic["Bridge Logic (Domain)"]:::container
        client_j["Jenkins Client (Reqwest)"]:::container
        client_g["Gitea Client (Reqwest)"]:::container
        
        webhook ==>|"Parsed Event"| logic
        logic -->|"JenkinsTriggerReq"| client_j
        logic -->|"GiteaStatusReq"| client_g
    end

    gitea -->|"POST /webhook"| webhook
    jenkins -->|"POST /jenkins-status"| webhook
    
    client_j ==>|"buildWithParameters"| jenkins
    client_g ==>|"POST /statuses/{sha}"| gitea
```

## Уровень 3: Component (Компоненты)
Демонстрирует внутреннюю структуру главного слоя бизнес-логики (`bridge-logic`).

```mermaid
graph TD
    classDef container fill:#438DD5,color:#fff,stroke:#2b5a88,stroke-width:2px,rx:5,ry:5,font-size:14px;
    classDef comp fill:#85BBF0,color:#000,stroke:#5b80a4,stroke-width:2px,rx:10,ry:10,font-weight:bold,font-size:14px;

    webhook["Webhook Server"]:::container
    jenkins_client["Jenkins Client"]:::container

    subgraph "Bridge Logic Crate"
        direction TB
        processor["EventProcessor Struct"]:::comp
        push_map["Push Mapper"]:::comp
        pr_map["PR Mapper"]:::comp
        status_map["Status Mapper"]:::comp
    end

    webhook ==>|"Event Payload"| processor
    
    processor -->|"PushEvent"| push_map
    processor -->|"PullRequestEvent"| pr_map
    processor -->|"PipelineStatus"| status_map
    
    push_map -.->|"Map to Jenkins vars"| jenkins_client
    pr_map -.->|"Map to Jenkins vars"| jenkins_client
```

> **Примечание:** Уровень 4 (Code) в C4 обычно не рисуется, так как он слишком детализирован, и его роль выполняют UML диаграммы классов или сам исходный код. В Rust эту роль отлично выполняет `cargo doc`.
