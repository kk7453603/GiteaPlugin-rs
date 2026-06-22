use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub login: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    pub private: bool,
    pub html_url: String,
    pub ssh_url: String,
    pub clone_url: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub url: String,
    pub author: User,
    pub committer: User,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub url: String,
    pub number: i64,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,
    pub head: PRBranchInfo,
    pub base: PRBranchInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRBranchInfo {
    pub label: String,
    pub ref_name: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
    pub repo: Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStatus {
    pub id: Option<i64>,
    pub status: String, // pending, success, error, failure, warning
    pub target_url: Option<String>,
    pub description: Option<String>,
    pub context: Option<String>,
}
