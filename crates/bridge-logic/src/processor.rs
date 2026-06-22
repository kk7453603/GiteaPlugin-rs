use gitea_client::events::{PushEvent, PullRequestEvent};
use tracing::info;

pub struct JenkinsTriggerRequest {
    pub job_name: String,
    pub params: Vec<(String, String)>,
}

pub struct EventProcessor {
    job_name: String,
}

impl EventProcessor {
    pub fn new(job_name: String) -> Self {
        Self { job_name }
    }

    #[tracing::instrument(skip(self, event))]
    pub fn process_push_event(&self, event: PushEvent) -> Option<JenkinsTriggerRequest> {
        info!("Processing push event for branch: {}", event.ref_field);
        
        let branch_name = event.ref_field.strip_prefix("refs/heads/").unwrap_or(&event.ref_field);
        let commit_sha = if let Some(head) = &event.head_commit {
            head.id.as_str()
        } else {
            &event.after
        };

        // Don't trigger on delete
        if commit_sha == "0000000000000000000000000000000000000000" {
            info!("Branch {} was deleted, ignoring trigger", branch_name);
            return None;
        }

        let params = vec![
            ("BRANCH_NAME".to_string(), branch_name.to_string()),
            ("COMMIT_SHA".to_string(), commit_sha.to_string()),
            ("GITEA_REPO_URL".to_string(), event.repository.html_url.to_string()),
            ("EVENT_TYPE".to_string(), "push".to_string()),
        ];

        Some(JenkinsTriggerRequest {
            job_name: self.job_name.clone(),
            params,
        })
    }

    #[tracing::instrument(skip(self, event))]
    pub fn process_pull_request_event(&self, event: PullRequestEvent) -> Option<JenkinsTriggerRequest> {
        info!("Processing pull request event #{} action: {}", event.number, event.action);

        if event.action == "closed" {
            info!("PR #{} closed, no trigger required", event.number);
            return None;
        }

        let pr_branch = &event.pull_request.head.ref_field;
        let pr_sha = &event.pull_request.head.sha;
        let pr_id_str = event.number.to_string();

        let params = vec![
            ("BRANCH_NAME".to_string(), pr_branch.to_string()),
            ("COMMIT_SHA".to_string(), pr_sha.to_string()),
            ("GITEA_REPO_URL".to_string(), event.repository.html_url.to_string()),
            ("PR_ID".to_string(), pr_id_str),
            ("EVENT_TYPE".to_string(), "pull_request".to_string()),
        ];

        Some(JenkinsTriggerRequest {
            job_name: self.job_name.clone(),
            params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitea_client::events::{PushEvent, PullRequestEvent};
    use gitea_client::models::{User, Repository, Commit, PullRequest, PRBranchInfo};

    fn mock_user() -> User {
        User {
            id: 1,
            login: "user".to_string(),
            full_name: None,
            email: None,
            avatar_url: None,
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
                author: mock_user(),
                committer: mock_user(),
                timestamp: "time".to_string(),
            }),
            repository: mock_repo(),
            pusher: mock_user(),
            sender: mock_user(),
        };

        let result = processor.process_push_event(event).unwrap();
        assert_eq!(result.job_name, "test-job");
        assert_eq!(result.params.len(), 4);
        assert_eq!(result.params[0], ("BRANCH_NAME".to_string(), "main".to_string()));
        assert_eq!(result.params[1], ("COMMIT_SHA".to_string(), "456".to_string()));
        assert_eq!(result.params[2], ("GITEA_REPO_URL".to_string(), "http://gitea/user/repo".to_string()));
        assert_eq!(result.params[3], ("EVENT_TYPE".to_string(), "push".to_string()));
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

        let result = processor.process_push_event(event);
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

        let result = processor.process_pull_request_event(event).unwrap();
        assert_eq!(result.job_name, "test-job");
        assert_eq!(result.params.len(), 5);
        assert_eq!(result.params[0], ("BRANCH_NAME".to_string(), "feature".to_string()));
        assert_eq!(result.params[1], ("COMMIT_SHA".to_string(), "789".to_string()));
        assert_eq!(result.params[2], ("GITEA_REPO_URL".to_string(), "http://gitea/user/repo".to_string()));
        assert_eq!(result.params[3], ("PR_ID".to_string(), "42".to_string()));
        assert_eq!(result.params[4], ("EVENT_TYPE".to_string(), "pull_request".to_string()));
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

        let result = processor.process_pull_request_event(event);
        assert!(result.is_none());
    }
}
