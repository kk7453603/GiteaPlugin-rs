use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum JenkinsClientError {
    #[error("HTTP Request Failed: {0}")]
    RequestError(#[from] ReqwestError),
    #[error("API Error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("Failed to acquire CSRF crumb")]
    CrumbError,
}

#[derive(Deserialize, Debug)]
pub struct CrumbResponse {
    #[serde(rename = "crumbRequestField")]
    pub field: String,
    pub crumb: String,
}

pub struct JenkinsClient {
    client: Client,
    base_url: String,
    user: String,
    token: String,
}

impl JenkinsClient {
    pub fn new(base_url: String, user: String, token: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap(),
            base_url: base_url.trim_end_matches('/').to_string(),
            user,
            token,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_crumb(&self) -> Result<CrumbResponse, JenkinsClientError> {
        let url = format!("{}/crumbIssuer/api/json", self.base_url);
        let res = self.client.get(&url)
            .basic_auth(&self.user, Some(&self.token))
            .send()
            .await?;

        if res.status().is_success() {
            let crumb = res.json::<CrumbResponse>().await?;
            Ok(crumb)
        } else {
            error!("Failed to get crumb: {}", res.status());
            Err(JenkinsClientError::CrumbError)
        }
    }

    #[tracing::instrument(skip(self, params))]
    pub async fn trigger_build_with_params(
        &self,
        job_name: &str,
        params: Vec<(&str, &str)>,
    ) -> Result<(), JenkinsClientError> {
        let crumb = self.get_crumb().await.unwrap_or(CrumbResponse {
            field: "Jenkins-Crumb".to_string(),
            crumb: "dummy".to_string(), // Some Jenkins setups disable CSRF
        });

        let url = format!("{}/job/{}/buildWithParameters", self.base_url, job_name);
        
        let mut req = self.client.post(&url)
            .basic_auth(&self.user, Some(&self.token))
            .header(crumb.field, crumb.crumb);

        if !params.is_empty() {
            req = req.form(&params);
        }

        let res = req.send().await?;

        if res.status().is_success() || res.status().is_redirection() {
            debug!("Successfully triggered job: {}", job_name);
            Ok(())
        } else {
            let status = res.status().as_u16();
            let text = res.text().await.unwrap_or_default();
            error!("Failed to trigger job: {} - {}", status, text);
            Err(JenkinsClientError::ApiError { status, message: text })
        }
    }
}
