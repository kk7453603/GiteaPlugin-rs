use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::AppState;
use gitea_client::events::{PushEvent, PullRequestEvent};
use tracing::{info, warn, error};

type HmacSha256 = Hmac<Sha256>;

#[tracing::instrument(skip(state, body_bytes, headers))]
pub async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    body_bytes: axum::body::Bytes,
) -> impl IntoResponse {
    // 1. Verify HMAC if secret is configured
    if let Some(secret) = &state.webhook_secret {
        let signature = headers.get("X-Gitea-Signature").and_then(|v| v.to_str().ok());
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

    // 2. Parse event type
    let event_type = headers.get("X-Gitea-Event").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
    info!("Received event: {}", event_type);

    match event_type {
        "push" => {
            if let Ok(event) = serde_json::from_slice::<PushEvent>(&body_bytes) {
                if let Some(trigger) = state.processor.process_push_event(event) {
                    let params: Vec<(&str, &str)> = trigger.params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                    if let Err(e) = state.jenkins_client.trigger_build_with_params(&trigger.job_name, params).await {
                        error!("Failed to trigger Jenkins: {:?}", e);
                    } else {
                        info!("Successfully triggered Jenkins job: {}", trigger.job_name);
                    }
                }
                StatusCode::OK
            } else {
                error!("Failed to parse PushEvent");
                StatusCode::BAD_REQUEST
            }
        }
        "pull_request" => {
            if let Ok(event) = serde_json::from_slice::<PullRequestEvent>(&body_bytes) {
                if let Some(trigger) = state.processor.process_pull_request_event(event) {
                    let params: Vec<(&str, &str)> = trigger.params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                    if let Err(e) = state.jenkins_client.trigger_build_with_params(&trigger.job_name, params).await {
                        error!("Failed to trigger Jenkins: {:?}", e);
                    } else {
                        info!("Successfully triggered Jenkins job: {}", trigger.job_name);
                    }
                }
                StatusCode::OK
            } else {
                error!("Failed to parse PullRequestEvent");
                StatusCode::BAD_REQUEST
            }
        }
        _ => {
            info!("Ignored event type: {}", event_type);
            StatusCode::OK // Acknowledge unsupported events
        }
    }
}
