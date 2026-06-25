use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, error};

/// Ошибки взаимодействия с Jenkins REST API.
#[derive(Error, Debug)]
pub enum JenkinsClientError {
    /// Ошибка выполнения HTTP-запроса (сетевой сбой или ошибка `reqwest`).
    #[error("HTTP Request Failed: {0}")]
    RequestError(#[from] ReqwestError),
    /// Ошибка уровня API: Jenkins вернул неуспешный HTTP-статус.
    #[error("API Error: {status} - {message}")]
    ApiError {
        /// HTTP-код ответа Jenkins.
        status: u16,
        /// Тело ответа с описанием ошибки.
        message: String,
    },
    /// Не удалось получить CSRF-крамб (crumb) для защиты от подделки запросов.
    #[error("Failed to acquire CSRF crumb")]
    CrumbError,
}

/// Ответ эндпоинта `crumbIssuer` с данными CSRF-крамба (crumb) Jenkins.
#[derive(Deserialize, Debug)]
pub struct CrumbResponse {
    /// Имя HTTP-заголовка для передачи крамба; JSON-поле `crumbRequestField`.
    #[serde(rename = "crumbRequestField")]
    pub field: String,
    /// Значение CSRF-крамба; JSON-поле `crumb`.
    pub crumb: String,
}

/// Асинхронный клиент Jenkins, хранящий базовый URL и учётные данные для запросов.
pub struct JenkinsClient {
    client: Client,
    base_url: String,
    user: String,
    token: String,
}

impl JenkinsClient {
    /// Создаёт клиент Jenkins по базовому URL, имени пользователя и API-токену.
    pub fn new(base_url: String, user: String, token: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .cookie_store(true)
                .build()
                .unwrap(),
            base_url: base_url.trim_end_matches('/').to_string(),
            user,
            token,
        }
    }

    /// Запрашивает CSRF-крамб (crumb) у эндпоинта `crumbIssuer` Jenkins.
    #[tracing::instrument(skip(self))]
    pub async fn get_crumb(&self) -> Result<CrumbResponse, JenkinsClientError> {
        let url = format!("{}/crumbIssuer/api/json", self.base_url);
        let res = self
            .client
            .get(&url)
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

    /// Запускает параметризованную сборку задания Jenkins через `buildWithParameters`.
    #[tracing::instrument(skip(self, params))]
    pub async fn trigger_build_with_params<T: serde::Serialize + ?Sized>(
        &self,
        job_name: &str,
        params: &T,
    ) -> Result<(), JenkinsClientError> {
        // Если CSRF-защита в Jenkins отключена, эндпоинт крамба недоступен —
        // подставляем фиктивный крамб, чтобы запрос на сборку всё равно прошёл.
        let crumb = self.get_crumb().await.unwrap_or(CrumbResponse {
            field: "Jenkins-Crumb".to_string(),
            crumb: "dummy".to_string(),
        });

        let url = format!("{}/job/{}/buildWithParameters", self.base_url, job_name);

        let mut req = self
            .client
            .post(&url)
            .basic_auth(&self.user, Some(&self.token))
            .header(crumb.field, crumb.crumb);

        req = req.form(params);

        let res = req.send().await?;

        if res.status().is_success() || res.status().is_redirection() {
            debug!("Successfully triggered job: {}", job_name);
            Ok(())
        } else {
            let status = res.status().as_u16();
            let text = res.text().await.unwrap_or_default();
            error!("Failed to trigger job: {} - {}", status, text);
            Err(JenkinsClientError::ApiError {
                status,
                message: text,
            })
        }
    }
}
