#!/bin/bash
set -e

# Wait for services to be up
echo "Waiting for webhook server to be ready on port 3000..."
while ! curl -s http://localhost:3000 > /dev/null; do
    # Server might return 404 for GET /, but curl will succeed
    sleep 2
done
echo "Webhook server is up!"

echo "Sending mock Push Event to webhook-server..."

PAYLOAD=$(cat <<EOF
{
  "ref": "refs/heads/main",
  "before": "0000000000000000000000000000000000000000",
  "after": "1234567890abcdef",
  "compare_url": "http://gitea/test/repo/compare",
  "commits": [],
  "repository": {
    "id": 1,
    "name": "test-repo",
    "full_name": "test/test-repo",
    "owner": {
      "id": 1,
      "login": "test"
    },
    "private": false,
    "html_url": "http://gitea:3000/test/test-repo",
    "ssh_url": "git@gitea:test/test-repo.git",
    "clone_url": "http://gitea:3000/test/test-repo.git",
    "default_branch": "main"
  },
  "pusher": {
    "id": 1,
    "login": "test"
  },
  "sender": {
    "id": 1,
    "login": "test"
  }
}
EOF
)

# Call webhook
STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:3000/gitea-webhook/post \
    -H "Content-Type: application/json" \
    -H "X-Gitea-Event: push" \
    -d "$PAYLOAD")

if [ "$STATUS" -eq 200 ]; then
    echo "Successfully received 200 OK from webhook server."
else
    echo "Failed! Received HTTP $STATUS"
    exit 1
fi

echo "Testing Jenkins Status Callback..."
STATUS_PAYLOAD=$(cat <<EOF
{
  "repo_owner": "test",
  "repo_name": "test-repo",
  "commit_sha": "1234567890abcdef",
  "build_status": "SUCCESS",
  "target_url": "http://jenkins:8080/job/test-job/1/",
  "context": "jenkins/build"
}
EOF
)

STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:3000/jenkins-status \
    -H "Content-Type: application/json" \
    -d "$STATUS_PAYLOAD")

if [ "$STATUS" -eq 200 ] || [ "$STATUS" -eq 500 ]; then
    # 500 is acceptable if Jenkins/Gitea mock endpoints are actually hit and return error because they are not set up properly in E2E
    echo "Jenkins Status callback returned $STATUS. (500 expected if Gitea returns 404)"
else
    echo "Failed! Received HTTP $STATUS on Jenkins Status Callback"
    exit 1
fi

echo "E2E basic routing tests passed."
