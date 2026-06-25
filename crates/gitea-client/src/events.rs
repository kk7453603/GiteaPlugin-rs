use crate::models::{Commit, PullRequest, Repository, User};
use serde::{Deserialize, Serialize};

/// Полезная нагрузка вебхук-события push, отправляемого Gitea при добавлении коммитов.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEvent {
    /// Полная ссылка на ветку или тег (JSON-поле `ref`), например `refs/heads/main`.
    #[serde(rename = "ref")]
    pub ref_field: String,
    /// SHA коммита, на который указывала ссылка до push.
    pub before: String,
    /// SHA коммита, на который указывает ссылка после push.
    pub after: String,
    /// URL для сравнения изменений между состояниями `before` и `after`.
    pub compare_url: String,
    /// Список коммитов, вошедших в данный push.
    pub commits: Vec<Commit>,
    /// Головной коммит push, если он присутствует в полезной нагрузке.
    pub head_commit: Option<Commit>,
    /// Репозиторий, в котором произошло событие.
    pub repository: Repository,
    /// Пользователь, выполнивший push.
    pub pusher: User,
    /// Пользователь, инициировавший доставку вебхука.
    pub sender: User,
}

/// Полезная нагрузка вебхук-события pull request, отправляемого Gitea.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestEvent {
    /// Действие над pull request (например, `opened`, `closed`, `reopened`, `synchronized`).
    pub action: String,
    /// Номер pull request в репозитории.
    pub number: i64,
    /// Данные самого pull request.
    pub pull_request: PullRequest,
    /// Репозиторий, в котором произошло событие.
    pub repository: Repository,
    /// Пользователь, инициировавший доставку вебхука.
    pub sender: User,
}

/// Обобщённое вебхук-событие Gitea, объединяющее поддерживаемые и неизвестные типы событий.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GiteaEvent {
    /// Событие push.
    Push(Box<PushEvent>),
    /// Событие pull request.
    PullRequest(Box<PullRequestEvent>),
    /// Нераспознанное событие, сохранённое как сырое JSON-значение.
    Unknown(serde_json::Value),
}
