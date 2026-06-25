use crate::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use gitea_client::events::{GiteaEvent, PullRequestEvent, PushEvent};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

/// Принимает вебхук Gitea, проверяет HMAC-подпись и запускает сборку Jenkins при поддерживаемом событии.
#[tracing::instrument(skip(state, body_bytes, headers))]
pub async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    body_bytes: axum::body::Bytes,
) -> impl IntoResponse {
    // Защита от подделки: при заданном секрете запрос без корректной подписи отклоняем
    if let Some(secret) = &state.webhook_secret {
        let signature = headers
            .get("X-Gitea-Signature")
            .and_then(|v| v.to_str().ok());
        if let Some(sig) = signature {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
            mac.update(&body_bytes);
            let expected = hex::encode(mac.finalize().into_bytes());
            if sig != expected {
                warn!("Invalid webhook signature");
                return StatusCode::UNAUTHORIZED;
            }
        } else {
            warn!("Missing webhook signature");
            return StatusCode::UNAUTHORIZED;
        }
    }

    // По заголовку определяем тип события, чтобы выбрать схему разбора тела
    let event_type_str = headers
        .get("X-Gitea-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    info!("Received event: {}", event_type_str);

    let event: GiteaEvent = match event_type_str {
        "push" => {
            serde_json::from_slice::<PushEvent>(&body_bytes).map(|e| GiteaEvent::Push(Box::new(e)))
        }
        "pull_request" => serde_json::from_slice::<PullRequestEvent>(&body_bytes)
            .map(|e| GiteaEvent::PullRequest(Box::new(e))),
        _ => {
            info!("Ignored event type: {}", event_type_str);
            return StatusCode::OK; // Неподдерживаемые события подтверждаем, чтобы Gitea не повторяла доставку
        }
    }
    .unwrap_or(GiteaEvent::Unknown(serde_json::Value::Null));

    if let GiteaEvent::Unknown(_) = event {
        error!("Failed to parse event: {}", event_type_str);
        return StatusCode::BAD_REQUEST;
    }

    if let Some(trigger) = state.processor.process(event) {
        if let Err(e) = state
            .jenkins_client
            .trigger_build_with_params(&trigger.job_name, &trigger.params)
            .await
        {
            error!("Failed to trigger Jenkins: {:?}", e);
        } else {
            info!("Successfully triggered Jenkins job: {}", trigger.job_name);
        }
    }

    StatusCode::OK
}
