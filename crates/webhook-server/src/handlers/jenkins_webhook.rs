use crate::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use gitea_client::models::CommitStatus;
use serde::Deserialize;
use tracing::{error, info};

/// Полезная нагрузка колбэка статуса сборки от Jenkins для обновления статуса коммита в Gitea.
#[derive(Deserialize)]
pub struct JenkinsStatusPayload {
    /// Владелец репозитория в Gitea (JSON-поле `repo_owner`).
    pub repo_owner: String,
    /// Имя репозитория в Gitea (JSON-поле `repo_name`).
    pub repo_name: String,
    /// SHA коммита, к которому относится статус сборки (JSON-поле `commit_sha`).
    pub commit_sha: String,
    /// Статус сборки Jenkins: `SUCCESS`, `FAILURE`, `UNSTABLE` и т. п. (JSON-поле `build_status`).
    pub build_status: String,
    /// URL страницы сборки в Jenkins для перехода из Gitea (JSON-поле `target_url`).
    pub target_url: String,
    /// Контекст (имя проверки), отображаемый рядом со статусом коммита (JSON-поле `context`).
    pub context: String,
}

/// Принимает колбэк статуса сборки Jenkins и публикует соответствующий статус коммита в Gitea.
#[tracing::instrument(skip(state, payload))]
pub async fn handle(
    State(state): State<AppState>,
    Json(payload): Json<JenkinsStatusPayload>,
) -> impl IntoResponse {
    info!(
        "Received Jenkins status update for {}/{} @ {}",
        payload.repo_owner, payload.repo_name, payload.commit_sha
    );

    let gitea_status = match payload.build_status.as_str() {
        "SUCCESS" => "success",
        "FAILURE" | "UNSTABLE" | "ABORTED" => "failure",
        "IN_PROGRESS" => "pending",
        _ => "warning",
    };

    let status = CommitStatus {
        id: None,
        status: gitea_status.to_string(),
        target_url: Some(payload.target_url),
        description: Some(format!("Jenkins build: {}", payload.build_status)),
        context: Some(payload.context),
    };

    match state
        .gitea_client
        .create_commit_status(
            &payload.repo_owner,
            &payload.repo_name,
            &payload.commit_sha,
            &status,
        )
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Failed to update Gitea commit status: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
