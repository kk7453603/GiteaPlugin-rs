use gitea_client::client::GiteaClient;
use gitea_client::models::CommitStatus;
use wiremock::matchers::{method, path, header, body_json_string};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_commit_status_success() {
    let mock_server = MockServer::start().await;

    let expected_status = CommitStatus {
        id: None,
        status: "success".to_string(),
        target_url: Some("http://jenkins/job/test/1".to_string()),
        description: Some("Build succeeded".to_string()),
        context: Some("jenkins/build".to_string()),
    };

    let expected_body = serde_json::to_string(&expected_status).unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/repos/myorg/myrepo/statuses/abcdef123456"))
        .and(header("Authorization", "token mytoken"))
        .and(body_json_string(expected_body))
        .respond_with(ResponseTemplate::new(201)) // Gitea returns 201 Created
        .mount(&mock_server)
        .await;

    let client = GiteaClient::new(mock_server.uri(), "mytoken".to_string());
    
    let result = client.create_commit_status("myorg", "myrepo", "abcdef123456", &expected_status).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_commit_status_failure() {
    let mock_server = MockServer::start().await;

    let status = CommitStatus {
        id: None,
        status: "success".to_string(),
        target_url: None,
        description: None,
        context: None,
    };

    Mock::given(method("POST"))
        .and(path("/api/v1/repos/myorg/myrepo/statuses/abcdef123456"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&mock_server)
        .await;

    let client = GiteaClient::new(mock_server.uri(), "mytoken".to_string());
    
    let result = client.create_commit_status("myorg", "myrepo", "abcdef123456", &status).await;
    
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("API Error: 404 - Not Found"));
}
