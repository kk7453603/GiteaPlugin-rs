#!/bin/bash
set -e

echo "Setting up real Jenkins job and Gitea repository..."

# Wait for Jenkins
echo "Waiting for Jenkins..."
while ! curl -s http://localhost:8080 > /dev/null; do
    sleep 2
done

# Wait for Gitea
echo "Waiting for Gitea..."
while ! curl -s http://localhost:3001 > /dev/null; do
    sleep 2
done

# We will just print the instructions because automating the initial
# admin setup for both Jenkins and Gitea through curl requires bypassing CSRF,
# creating initial admin users, etc., which is complex for a simple e2e script.

echo "========================================================"
echo "To test the full E2E flow manually with the UI:"
echo "1. Open http://localhost:8080 (Jenkins)"
echo "   Create an admin user and a Freestyle project named 'test-job'."
echo "   Add string parameters: BRANCH_NAME, COMMIT_SHA, GITEA_REPO_URL, EVENT_TYPE"
echo "   (Set JENKINS_USER/JENKINS_TOKEN in docker-compose.yml to match your admin user)"
echo "2. Open http://localhost:3001 (Gitea)"
echo "   Register a new user."
echo "   Clone a repository from GitHub using 'New Migration'."
echo "   Go to Repo Settings -> Webhooks -> Add Webhook -> Gitea."
echo "   Set Target URL to: http://webhook-server:3000/gitea-webhook/post"
echo "   Set HTTP Method to POST, Trigger on: Push & Pull Request."
echo "3. Push a commit or open a PR in Gitea."
echo "4. Observe Jenkins 'test-job' being triggered automatically!"
echo "5. For status reporting, add a build step in Jenkins that curls:"
echo "   http://webhook-server:3000/jenkins-status"
echo "   with a JSON body containing the build status."
echo "========================================================"
