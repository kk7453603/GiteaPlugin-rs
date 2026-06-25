use gitea_client::events::{GiteaEvent, PullRequestEvent, PushEvent};
use serde::Serialize;
use tracing::info;

/// Параметры сборки Jenkins, сериализуемые в JSON-тело запроса `buildWithParameters`.
#[derive(Debug, Serialize, PartialEq)]
pub struct BuildParams {
    /// Имя ветки сборки; сериализуется в JSON-поле `BRANCH_NAME`.
    #[serde(rename = "BRANCH_NAME")]
    pub branch_name: String,
    /// SHA коммита, для которого запускается сборка; сериализуется в JSON-поле `COMMIT_SHA`.
    #[serde(rename = "COMMIT_SHA")]
    pub commit_sha: String,
    /// URL репозитория Gitea; сериализуется в JSON-поле `GITEA_REPO_URL`.
    #[serde(rename = "GITEA_REPO_URL")]
    pub gitea_repo_url: String,
    /// Тип события (`push` или `pull_request`); сериализуется в JSON-поле `EVENT_TYPE`.
    #[serde(rename = "EVENT_TYPE")]
    pub event_type: String,
    /// Номер pull request, если событие связано с PR; сериализуется в JSON-поле `PR_ID`.
    #[serde(rename = "PR_ID", skip_serializing_if = "Option::is_none")]
    pub pr_id: Option<String>,
}

/// Запрос на запуск сборки Jenkins: имя job и набор параметров сборки.
pub struct JenkinsTriggerRequest {
    /// Имя job в Jenkins, который требуется запустить.
    pub job_name: String,
    /// Параметры сборки, передаваемые в Jenkins.
    pub params: BuildParams,
}

/// Преобразователь событий Gitea в запросы на запуск сборок Jenkins.
pub struct EventProcessor {
    job_name: String,
}

impl EventProcessor {
    /// Создаёт `EventProcessor`, запускающий указанный job Jenkins.
    pub fn new(job_name: String) -> Self {
        Self { job_name }
    }

    /// Преобразует событие Gitea в запрос на сборку Jenkins, если событие его требует.
    #[tracing::instrument(skip(self, event))]
    pub fn process(&self, event: GiteaEvent) -> Option<JenkinsTriggerRequest> {
        match event {
            GiteaEvent::Push(e) => self.process_push_event(*e),
            GiteaEvent::PullRequest(e) => self.process_pull_request_event(*e),
            _ => None,
        }
    }

    fn process_push_event(&self, event: PushEvent) -> Option<JenkinsTriggerRequest> {
        info!("Processing push event for branch: {}", event.ref_field);

        let branch_name = event
            .ref_field
            .strip_prefix("refs/heads/")
            .unwrap_or(&event.ref_field);
        let commit_sha = event
            .head_commit
            .as_ref()
            .map_or(event.after.as_str(), |h| h.id.as_str());

        if commit_sha == "0000000000000000000000000000000000000000" {
            info!("Branch {} was deleted, ignoring trigger", branch_name);
            return None;
        }

        Some(JenkinsTriggerRequest {
            job_name: self.job_name.clone(),
            params: BuildParams {
                branch_name: branch_name.to_string(),
                commit_sha: commit_sha.to_string(),
                gitea_repo_url: event.repository.html_url,
                event_type: "push".to_string(),
                pr_id: None,
            },
        })
    }

    fn process_pull_request_event(&self, event: PullRequestEvent) -> Option<JenkinsTriggerRequest> {
        info!(
            "Processing pull request event #{} action: {}",
            event.number, event.action
        );

        if event.action == "closed" {
            info!("PR #{} closed, no trigger required", event.number);
            return None;
        }

        Some(JenkinsTriggerRequest {
            job_name: self.job_name.clone(),
            params: BuildParams {
                branch_name: event.pull_request.head.ref_field,
                commit_sha: event.pull_request.head.sha,
                gitea_repo_url: event.repository.html_url,
                event_type: "pull_request".to_string(),
                pr_id: Some(event.number.to_string()),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitea_client::events::{PullRequestEvent, PushEvent};
    use gitea_client::models::{Commit, PRBranchInfo, PayloadUser, PullRequest, Repository, User};

    fn mock_user() -> User {
        User {
            id: 1,
            login: "user".to_string(),
            full_name: None,
            email: None,
            avatar_url: None,
        }
    }

    fn mock_payload_user() -> PayloadUser {
        PayloadUser {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            username: Some("testuser".to_string()),
        }
    }

    fn mock_repo() -> Repository {
        Repository {
            id: 1,
            name: "repo".to_string(),
            full_name: "user/repo".to_string(),
            owner: mock_user(),
            private: false,
            html_url: "http://gitea/user/repo".to_string(),
            ssh_url: "git@gitea:user/repo.git".to_string(),
            clone_url: "http://gitea/user/repo.git".to_string(),
            default_branch: "main".to_string(),
        }
    }

    #[test]
    fn test_process_push_event_normal() {
        let processor = EventProcessor::new("test-job".to_string());
        let event = PushEvent {
            ref_field: "refs/heads/main".to_string(),
            before: "123".to_string(),
            after: "456".to_string(),
            compare_url: "url".to_string(),
            commits: vec![],
            head_commit: Some(Commit {
                id: "456".to_string(),
                message: "msg".to_string(),
                url: "url".to_string(),
                author: mock_payload_user(),
                committer: mock_payload_user(),
                timestamp: "time".to_string(),
            }),
            repository: mock_repo(),
            pusher: mock_user(),
            sender: mock_user(),
        };

        let result = processor
            .process(GiteaEvent::Push(Box::new(event)))
            .unwrap();
        assert_eq!(result.job_name, "test-job");
        assert_eq!(result.params.branch_name, "main");
        assert_eq!(result.params.commit_sha, "456");
        assert_eq!(result.params.gitea_repo_url, "http://gitea/user/repo");
        assert_eq!(result.params.event_type, "push");
        assert_eq!(result.params.pr_id, None);
    }

    #[test]
    fn test_process_push_event_deleted_branch() {
        let processor = EventProcessor::new("test-job".to_string());
        let event = PushEvent {
            ref_field: "refs/heads/feature".to_string(),
            before: "123".to_string(),
            after: "0000000000000000000000000000000000000000".to_string(),
            compare_url: "url".to_string(),
            commits: vec![],
            head_commit: None,
            repository: mock_repo(),
            pusher: mock_user(),
            sender: mock_user(),
        };

        let result = processor.process(GiteaEvent::Push(Box::new(event)));
        assert!(result.is_none());
    }

    #[test]
    fn test_process_pr_event_opened() {
        let processor = EventProcessor::new("test-job".to_string());
        let event = PullRequestEvent {
            action: "opened".to_string(),
            number: 42,
            pull_request: PullRequest {
                id: 100,
                url: "url".to_string(),
                number: 42,
                title: "Fix bug".to_string(),
                body: Some("Fixed it".to_string()),
                state: "open".to_string(),
                html_url: "http://gitea/user/repo/pulls/42".to_string(),
                head: PRBranchInfo {
                    label: "user:feature".to_string(),
                    ref_name: "feature".to_string(),
                    ref_field: "feature".to_string(),
                    sha: "789".to_string(),
                    repo: mock_repo(),
                },
                base: PRBranchInfo {
                    label: "user:main".to_string(),
                    ref_name: "main".to_string(),
                    ref_field: "main".to_string(),
                    sha: "123".to_string(),
                    repo: mock_repo(),
                },
            },
            repository: mock_repo(),
            sender: mock_user(),
        };

        let result = processor
            .process(GiteaEvent::PullRequest(Box::new(event)))
            .unwrap();
        assert_eq!(result.job_name, "test-job");
        assert_eq!(result.params.branch_name, "feature");
        assert_eq!(result.params.commit_sha, "789");
        assert_eq!(result.params.gitea_repo_url, "http://gitea/user/repo");
        assert_eq!(result.params.event_type, "pull_request");
        assert_eq!(result.params.pr_id, Some("42".to_string()));
    }

    #[test]
    fn test_process_pr_event_closed() {
        let processor = EventProcessor::new("test-job".to_string());
        let event = PullRequestEvent {
            action: "closed".to_string(),
            number: 42,
            pull_request: PullRequest {
                id: 100,
                url: "url".to_string(),
                number: 42,
                title: "Fix bug".to_string(),
                body: Some("Fixed it".to_string()),
                state: "closed".to_string(),
                html_url: "http://gitea/user/repo/pulls/42".to_string(),
                head: PRBranchInfo {
                    label: "user:feature".to_string(),
                    ref_name: "feature".to_string(),
                    ref_field: "feature".to_string(),
                    sha: "789".to_string(),
                    repo: mock_repo(),
                },
                base: PRBranchInfo {
                    label: "user:main".to_string(),
                    ref_name: "main".to_string(),
                    ref_field: "main".to_string(),
                    sha: "123".to_string(),
                    repo: mock_repo(),
                },
            },
            repository: mock_repo(),
            sender: mock_user(),
        };

        let result = processor.process(GiteaEvent::PullRequest(Box::new(event)));
        assert!(result.is_none());
    }
}
