use reqwest::{Client, Error as ReqwestError};
use thiserror::Error;
use crate::models::CommitStatus;

#[derive(Error, Debug)]
pub enum GiteaClientError {
    #[error("HTTP Request Failed: {0}")]
    RequestError(#[from] ReqwestError),
    #[error("API Error: {status} - {message}")]
    ApiError { status: u16, message: String },
}

pub struct GiteaClient {
    client: Client,
    base_url: String,
    token: String,
}

impl GiteaClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        }
    }

    pub async fn create_commit_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        status: &CommitStatus,
    ) -> Result<(), GiteaClientError> {
        let url = format!("{}/api/v1/repos/{}/{}/statuses/{}", self.base_url, owner, repo, sha);
        
        let response = self.client.post(&url)
            .header("Authorization", format!("token {}", self.token))
            .json(status)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(GiteaClientError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            })
        }
    }
}
