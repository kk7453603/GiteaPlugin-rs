use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use crate::AppState;
use gitea_client::models::CommitStatus;
use tracing::{info, error};

#[derive(Deserialize)]
pub struct JenkinsStatusPayload {
    pub repo_owner: String,
    pub repo_name: String,
    pub commit_sha: String,
    pub build_status: String, // SUCCESS, FAILURE, UNSTABLE, etc.
    pub target_url: String,
    pub context: String,
}

#[tracing::instrument(skip(state, payload))]
pub async fn handle(
    State(state): State<AppState>,
    Json(payload): Json<JenkinsStatusPayload>,
) -> impl IntoResponse {
    info!("Received Jenkins status update for {}/{} @ {}", payload.repo_owner, payload.repo_name, payload.commit_sha);

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

    match state.gitea_client.create_commit_status(
        &payload.repo_owner,
        &payload.repo_name,
        &payload.commit_sha,
        &status,
    ).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Failed to update Gitea commit status: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
