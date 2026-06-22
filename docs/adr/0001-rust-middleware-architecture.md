# 0001. Architecture: Rust Middleware replacing Java Plugin

## Status
Accepted

## Context
The previous integration between Gitea and Jenkins relied on the `gitea-plugin` written in Java. This plugin ran inside the Jenkins JVM, which tightly coupled the webhook logic and status reporting to Jenkins's internal extension points. Maintenance was difficult due to Jenkins API changes, and debugging required full Jenkins instances. 

## Decision
We decided to extract the Gitea-Jenkins bridge logic into a standalone middleware written in Rust.
1. The middleware will receive webhooks directly from Gitea.
2. It will transform Gitea payloads into simple Jenkins parameterized build triggers (`JenkinsTriggerRequest`).
3. It will use the Jenkins REST API (`/job/{job_name}/buildWithParameters`) to start jobs.
4. It will provide a callback endpoint (`/jenkins-status`) for Jenkins pipelines to report build statuses back to Gitea via the Gitea REST API.

## Consequences
- **Positive:** Decouples the webhook processing from Jenkins core. Allows independent deployment, scaling, and easier debugging (Agentless model).
- **Positive:** Rust's memory safety and performance (via `tokio` and `axum`) ensure high throughput and low resource usage.
- **Negative:** Jenkins jobs must be parameterized manually or via declarative pipelines to accept `BRANCH_NAME` and `COMMIT_SHA`.
- **Negative:** We lose the native Jenkins UI for configuring Gitea servers. Configuration is now strictly via environment variables (`GITEA_TOKEN`, `JENKINS_URL`, etc.).
