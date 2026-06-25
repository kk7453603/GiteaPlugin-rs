# Deployment & Operations Guide (DevOps)

This document provides deployment patterns, configuration options, and troubleshooting steps for running `gitea-plugin-rs` in production.

## Deployment Architecture

The `gitea-plugin-rs` is a lightweight, stateless middleware written in Rust. It acts as an integration layer between Gitea and Jenkins.

**Pattern**: Sidecar or Independent Microservice
- **Stateless**: The server stores no data on disk. All state (like the Jenkins `JSESSIONID` cookie) is kept in memory.
- **Scaling**: Can be horizontally scaled behind a load balancer (e.g., Nginx, Traefik). Gitea webhooks will be distributed across instances.
- **Resource Footprint**: Minimal (typically uses <50MB RAM).

## Docker Containerization

We provide a multi-stage `Dockerfile` based on `debian:bookworm-slim`.
It uses `rustls-tls` to avoid OpenSSL dependencies, resulting in a smaller attack surface and cleaner container image.

```bash
docker build -t gitea-plugin-rs:latest .
docker run -d \
  -p 3000:3000 \
  --env-file .env \
  gitea-plugin-rs:latest
```

## Configuration (Environment Variables)

The application follows the 12-Factor App methodology. All configuration is done via environment variables:

### Server Settings
* `SERVER_PORT`: The port the Axum server listens on (default `3000`).
* `WEBHOOK_SECRET`: (Optional) The secret used by Gitea to sign webhooks. The server validates the `X-Gitea-Signature` HMAC.

### Jenkins Integration
* `JENKINS_URL`: Base URL of the Jenkins instance (default `http://localhost:8080`; e.g., `http://jenkins.internal.local:8080`).
* `JENKINS_USER`: Service account username. Must have `Job/Build` permissions (default `admin`).
* `JENKINS_TOKEN`: API Token for the user. Do not use passwords (default `token`).
* `JENKINS_JOB`: The target parameterized Jenkins job to trigger (default `gitea-trigger-job`; e.g., `gitea-ci-pipeline`).

### Gitea Integration
* `GITEA_URL`: Base URL of the Gitea instance (default `http://localhost:3000`; e.g., `https://git.company.com`).
* `GITEA_TOKEN`: API Token. Used exclusively to post Commit Statuses back to Gitea. Must have `repo:write` permissions (default `token`).

## Security Considerations

* **TLS**: By default, `reqwest` requires valid TLS certificates for external calls. If your internal Jenkins/Gitea use self-signed certificates, you must add the CA to the container's trust store.
* **Secrets**: Do not hardcode tokens. Pass them securely via Docker Secrets, Kubernetes Secrets, or a secure `.env` file mounted at runtime.
* **Network Isolation**: Place `gitea-plugin-rs` in a private subnet. It only needs ingress from Gitea and egress to Jenkins and Gitea.

## Troubleshooting

1. **Jenkins returns `403 Forbidden` on Trigger**
   - **Symptom**: Logs show `Failed to trigger Jenkins: 403 No valid crumb was included in the request`.
   - **Cause**: The `jenkins-client` fetches a CSRF crumb, but Jenkins rejects it.
   - **Fix**: We utilize `cookie_store(true)` to persist `JSESSIONID`. Ensure Jenkins is not configured to strictly validate the client IP against the crumb if a load balancer sits between them. Also, verify `JENKINS_USER` and `JENKINS_TOKEN` are valid.

2. **Gitea Webhooks fail with `422 Unprocessable Entity`**
   - **Symptom**: The webhook server returns `422`.
   - **Cause**: JSON payload mismatch. Usually happens if Gitea adds new fields or removes fields from the webhook payload.
   - **Fix**: Check application logs (`docker logs`). The Axum server will log the exact `serde` parsing error (e.g., "missing field `email`"). Update models in `crates/gitea-client/src/models.rs`.

3. **Jenkins Status not appearing in Gitea**
   - **Cause**: Jenkins must be explicitly configured to send a POST request to `/jenkins-status` at the start and end of the pipeline. Check the Jenkins console output to ensure the `curl` or `HTTP Request` step was successful.
