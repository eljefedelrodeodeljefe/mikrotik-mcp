use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

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
        // RouterOS item IDs are like "*1"; accept with or without the leading *.
        let id = if id.starts_with('*') {
            id.to_string()
        } else {
            format!("*{}", id)
        };
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
