use serde::{Deserialize, Serialize};
use crate::models::{User, Repository, Commit, PullRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub before: String,
    pub after: String,
    pub compare_url: String,
    pub commits: Vec<Commit>,
    pub head_commit: Option<Commit>,
    pub repository: Repository,
    pub pusher: User,
    pub sender: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestEvent {
    pub action: String, // opened, closed, reopened, edited, assigned, unassigned, label_updated, label_cleared, milestoned, demilestoned, review_requested, review_request_removed, synchronized
    pub number: i64,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvent {
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub ref_type: String, // branch, tag
    pub default_branch: String,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEvent {
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub ref_type: String, // branch, tag
    pub repository: Repository,
    pub sender: User,
}

// Additional events can be added later (Release, Repository, etc.)
