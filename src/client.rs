use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

/// Where the router should push an encrypted backup, via RouterOS
/// `/tool fetch upload=yes mode=sftp`. The router initiates the connection
/// (egress only), so this works even when inbound FTP is disabled by a
/// hardened management config.
#[derive(Clone, Debug)]
pub struct SftpTarget {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    /// Remote directory (or empty for the login home dir).
    pub path: String,
}

pub struct RouterosClient {
    base_url: String,
    host: String,
    username: String,
    password: String,
    client: Client,
}

impl RouterosClient {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        tls_verify: bool,
    ) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(!tls_verify)
            .build()
            .context("failed to build HTTP client")?;

        let scheme = if port == 80 { "http" } else { "https" };
        Ok(Self {
            base_url: format!("{}://{}:{}/rest", scheme, host, port),
            host: host.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            client,
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("RouterOS returned error status")?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(body)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("RouterOS returned error status")?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    /// Adds a new item to a RouterOS menu (`PUT /rest/<menu>`). RouterOS maps
    /// `PUT` to "add"; a bare `POST /rest/<menu>` is a print query and is
    /// rejected for create payloads.
    pub async fn put<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(body)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("RouterOS returned error status")?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    /// Updates properties of an existing item (`PATCH /rest/<menu>/<id>`).
    /// RouterOS maps `PATCH` to "set"; only the supplied properties change.
    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        id: &str,
        body: &B,
    ) -> Result<T> {
        let id = normalise_id(id);
        let url = format!("{}/{}/{}", self.base_url, path.trim_start_matches('/'), id);
        self.client
            .patch(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(body)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("RouterOS returned error status")?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    pub async fn post_void<B: Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(body)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("RouterOS returned error status")?;
        Ok(())
    }

    /// Pushes a file already saved on the router out to `target` via
    /// `/tool fetch upload=yes mode=sftp`. Returns the fetch status object.
    /// Unlike [`ftp_download`](Self::ftp_download), nothing is pulled to the
    /// local machine — the file lands on the SFTP host — and the router makes
    /// only an outbound connection, so no inbound service (FTP) is required.
    pub async fn sftp_push(
        &self,
        src_filename: &str,
        remote_name: &str,
        target: &SftpTarget,
    ) -> Result<serde_json::Value> {
        let dst_path = if target.path.trim().is_empty() {
            remote_name.to_string()
        } else {
            format!("{}/{}", target.path.trim_end_matches('/'), remote_name)
        };
        let body = serde_json::json!({
            "upload": "yes",
            "mode": "sftp",
            "address": target.host,
            "port": target.port.to_string(),
            "user": target.user,
            "password": target.password,
            "src-path": src_filename,
            "dst-path": dst_path,
        });
        self.post("tool/fetch", &body).await
    }

    pub async fn ftp_download(&self, filename: &str) -> Result<Vec<u8>> {
        let output = tokio::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--user",
                &format!("{}:{}", self.username, self.password),
                &format!("ftp://{}:21/{}", self.host, filename),
            ])
            .output()
            .await
            .context("curl FTP: failed to spawn")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "curl FTP failed ({}): {}",
                output.status,
                stderr.trim()
            ));
        }

        Ok(output.stdout)
    }

    pub async fn ftp_upload(&self, local_path: &str, remote_filename: &str) -> Result<()> {
        let output = tokio::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--user",
                &format!("{}:{}", self.username, self.password),
                "-T",
                local_path,
                &format!("ftp://{}:21/{}", self.host, remote_filename),
            ])
            .output()
            .await
            .context("curl FTP upload: failed to spawn")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "curl FTP upload failed ({}): {}",
                output.status,
                stderr.trim()
            ));
        }

        Ok(())
    }

    /// Creates a client pointing at an arbitrary base URL for use in tests.
    /// The mock server URI (e.g. from wiremock) is used as the base; `/rest` is appended.
    #[cfg(test)]
    pub fn for_test(server_uri: &str) -> Self {
        Self {
            base_url: format!("{}/rest", server_uri.trim_end_matches('/')),
            host: "localhost".to_string(),
            username: "admin".to_string(),
            password: "test".to_string(),
            client: Client::builder().build().unwrap(),
        }
    }

    pub async fn delete(&self, path: &str, id: &str) -> Result<()> {
        let id = normalise_id(id);
        let url = format!("{}/{}/{}", self.base_url, path.trim_start_matches('/'), id);
        self.client
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("RouterOS returned error status")?;
        Ok(())
    }
}

/// RouterOS item IDs look like `*1`; accept them with or without the leading `*`.
fn normalise_id(id: &str) -> String {
    if id.starts_with('*') {
        id.to_string()
    } else {
        format!("*{id}")
    }
}
