//! Точка входа моста: Axum-сервер, принимающий вебхуки Gitea и колбэки статуса Jenkins.
//!
//! Настраивает разделяемое состояние ([`AppState`]), регистрирует маршруты
//! `POST /gitea-webhook/post` и `POST /jenkins-status` и запускает HTTP-сервер.

use axum::{routing::post, Router};
use bridge_logic::processor::EventProcessor;
use gitea_client::client::GiteaClient;
use jenkins_client::client::JenkinsClient;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

mod handlers;

/// Разделяемое состояние Axum-приложения, передаётся во все обработчики вебхуков.
#[derive(Clone)]
pub struct AppState {
    /// Процессор трансформации событий Gitea в запросы на сборку Jenkins.
    pub processor: Arc<EventProcessor>,
    /// Клиент Gitea API для обратного колбэка статуса коммита.
    pub gitea_client: Arc<GiteaClient>,
    /// Клиент Jenkins API для запуска сборок.
    pub jenkins_client: Arc<JenkinsClient>,
    /// Секрет для проверки HMAC-подписи вебхука; `None`, если проверка отключена.
    pub webhook_secret: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Конфигурация читается из переменных окружения, чтобы секреты не попадали в код
    let jenkins_url =
        std::env::var("JENKINS_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let jenkins_user = std::env::var("JENKINS_USER").unwrap_or_else(|_| "admin".to_string());
    let jenkins_token = std::env::var("JENKINS_TOKEN").unwrap_or_else(|_| "token".to_string());
    let jenkins_job =
        std::env::var("JENKINS_JOB").unwrap_or_else(|_| "gitea-trigger-job".to_string());

    let gitea_url =
        std::env::var("GITEA_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let gitea_token = std::env::var("GITEA_TOKEN").unwrap_or_else(|_| "token".to_string());
    let webhook_secret = std::env::var("WEBHOOK_SECRET").ok();

    let jenkins_client = JenkinsClient::new(jenkins_url, jenkins_user, jenkins_token);
    let gitea_client = GiteaClient::new(gitea_url, gitea_token);

    let processor = Arc::new(EventProcessor::new(jenkins_job));

    let state = AppState {
        processor,
        gitea_client: Arc::new(gitea_client),
        jenkins_client: Arc::new(jenkins_client),
        webhook_secret,
    };

    let app = Router::new()
        .route("/gitea-webhook/post", post(handlers::gitea_webhook::handle))
        .route("/jenkins-status", post(handlers::jenkins_webhook::handle))
        .with_state(state);

    let server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("0.0.0.0:{}", server_port);
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("Starting server on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
