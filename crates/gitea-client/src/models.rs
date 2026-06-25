use serde::{Deserialize, Serialize};

/// Пользователь Gitea (учётная запись).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Уникальный идентификатор пользователя.
    pub id: i64,
    /// Логин (имя учётной записи) пользователя.
    pub login: String,
    /// Полное отображаемое имя пользователя, если задано.
    pub full_name: Option<String>,
    /// Адрес электронной почты пользователя, если доступен.
    pub email: Option<String>,
    /// URL аватара пользователя, если задан.
    pub avatar_url: Option<String>,
}

/// Репозиторий Gitea и его основные атрибуты.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Уникальный идентификатор репозитория.
    pub id: i64,
    /// Короткое имя репозитория.
    pub name: String,
    /// Полное имя репозитория в формате `владелец/имя`.
    pub full_name: String,
    /// Владелец репозитория.
    pub owner: User,
    /// Признак приватности репозитория.
    pub private: bool,
    /// URL веб-страницы репозитория.
    pub html_url: String,
    /// URL для клонирования по протоколу SSH.
    pub ssh_url: String,
    /// URL для клонирования по протоколу HTTP(S).
    pub clone_url: String,
    /// Имя ветки по умолчанию.
    pub default_branch: String,
}

/// Сведения о пользователе внутри полезной нагрузки коммита (автор или коммиттер).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadUser {
    /// Имя пользователя, указанное в коммите.
    pub name: String,
    /// Адрес электронной почты, указанный в коммите.
    pub email: String,
    /// Логин пользователя Gitea, если он сопоставлен.
    pub username: Option<String>,
}

/// Коммит Git в полезной нагрузке вебхука Gitea.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// SHA-идентификатор коммита.
    pub id: String,
    /// Сообщение коммита.
    pub message: String,
    /// URL веб-страницы коммита.
    pub url: String,
    /// Автор коммита.
    pub author: PayloadUser,
    /// Коммиттер коммита.
    pub committer: PayloadUser,
    /// Временная метка коммита в формате ISO 8601.
    pub timestamp: String,
}

/// Pull request Gitea и его основные атрибуты.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Уникальный идентификатор pull request.
    pub id: i64,
    /// URL API данного pull request.
    pub url: String,
    /// Номер pull request в репозитории.
    pub number: i64,
    /// Состояние pull request (например, `open`, `closed`).
    pub state: String,
    /// Заголовок pull request.
    pub title: String,
    /// Текстовое описание pull request, если задано.
    pub body: Option<String>,
    /// URL веб-страницы pull request.
    pub html_url: String,
    /// Информация об исходной ветке (head) pull request.
    pub head: PRBranchInfo,
    /// Информация о целевой ветке (base) pull request.
    pub base: PRBranchInfo,
}

/// Информация о ветке pull request (исходной или целевой).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRBranchInfo {
    /// Метка ветки в формате `владелец:ветка`.
    pub label: String,
    /// Имя ветки.
    pub ref_name: String,
    /// Полная ссылка на ветку (JSON-поле `ref`), например `refs/heads/main`.
    #[serde(rename = "ref")]
    pub ref_field: String,
    /// SHA вершины ветки.
    pub sha: String,
    /// Репозиторий, которому принадлежит ветка.
    pub repo: Repository,
}

/// Статус коммита, отправляемый в Gitea для отображения результата сборки.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStatus {
    /// Идентификатор статуса, присваиваемый Gitea, если он известен.
    pub id: Option<i64>,
    /// Значение статуса (`pending`, `success`, `error`, `failure`, `warning`).
    pub status: String,
    /// URL, на который ведёт ссылка статуса (например, страница сборки).
    pub target_url: Option<String>,
    /// Человекочитаемое описание статуса.
    pub description: Option<String>,
    /// Контекст статуса, идентифицирующий источник проверки.
    pub context: Option<String>,
}
