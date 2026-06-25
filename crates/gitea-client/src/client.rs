use crate::models::CommitStatus;
use reqwest::{Client, Error as ReqwestError};
use thiserror::Error;

/// Ошибки, возникающие при работе клиента Gitea.
#[derive(Error, Debug)]
pub enum GiteaClientError {
    /// Сбой выполнения HTTP-запроса на уровне транспорта (обёртка над ошибкой `reqwest`).
    #[error("HTTP Request Failed: {0}")]
    RequestError(#[from] ReqwestError),
    /// Ошибка на уровне API Gitea: HTTP-статус и текст ответа сервера.
    #[error("API Error: {status} - {message}")]
    ApiError {
        /// HTTP-код статуса ответа, полученного от API Gitea.
        status: u16,
        /// Текст тела ответа с описанием ошибки от API Gitea.
        message: String,
    },
}

/// Асинхронный клиент REST API Gitea, инкапсулирующий базовый URL и токен авторизации.
pub struct GiteaClient {
    /// Внутренний HTTP-клиент `reqwest`, используемый для отправки запросов.
    client: Client,
    /// Базовый URL экземпляра Gitea без завершающего слеша.
    base_url: String,
    /// Токен доступа для аутентификации в API Gitea.
    token: String,
}

impl GiteaClient {
    /// Создаёт новый клиент Gitea по базовому URL и токену доступа.
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            client: reqwest::Client::builder().no_proxy().build().unwrap(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        }
    }

    /// Создаёт статус коммита в Gitea для указанного владельца, репозитория и SHA.
    pub async fn create_commit_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        status: &CommitStatus,
    ) -> Result<(), GiteaClientError> {
        let url = format!(
            "{}/api/v1/repos/{}/{}/statuses/{}",
            self.base_url, owner, repo, sha
        );

        let response = self
            .client
            .post(&url)
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
