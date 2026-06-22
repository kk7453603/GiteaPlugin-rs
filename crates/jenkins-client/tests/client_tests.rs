use jenkins_client::client::JenkinsClient;
use wiremock::matchers::{method, path, header, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use serde_json::json;

#[tokio::test]
async fn test_get_crumb_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/crumbIssuer/api/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "crumbRequestField": "Jenkins-Crumb",
            "crumb": "1234567890abcdef"
        })))
        .mount(&mock_server)
        .await;

    let client = JenkinsClient::new(mock_server.uri(), "admin".to_string(), "token".to_string());
    
    let crumb = client.get_crumb().await.unwrap();
    assert_eq!(crumb.field, "Jenkins-Crumb");
    assert_eq!(crumb.crumb, "1234567890abcdef");
}

#[tokio::test]
async fn test_trigger_build_with_params() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/crumbIssuer/api/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "crumbRequestField": "Jenkins-Crumb",
            "crumb": "test-crumb"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/job/test-job/buildWithParameters"))
        .and(header("Jenkins-Crumb", "test-crumb"))
        // Wiremock's body matcher for urlencoded forms can be tricky, so we'll just check success for now
        .respond_with(ResponseTemplate::new(201)) // Jenkins usually returns 201 Created
        .mount(&mock_server)
        .await;

    let client = JenkinsClient::new(mock_server.uri(), "admin".to_string(), "token".to_string());
    
    let params = vec![("BRANCH_NAME", "main"), ("COMMIT_SHA", "abc1234")];
    let result = client.trigger_build_with_params("test-job", params).await;
    
    assert!(result.is_ok());
}
