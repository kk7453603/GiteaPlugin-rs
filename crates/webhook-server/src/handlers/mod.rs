//! HTTP-обработчики вебхуков: приём событий Gitea и колбэков статуса Jenkins.

/// Обработчик входящих вебхуков Gitea (`POST /gitea-webhook/post`).
pub mod gitea_webhook;
/// Обработчик колбэков статуса сборки Jenkins (`POST /jenkins-status`).
pub mod jenkins_webhook;
