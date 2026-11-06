use anyhow::{Context, Result};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use crate::config::{Scheme, Tls};
use crate::params::RouterOsId;

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

/// A failed RouterOS REST call, kept typed so the MCP layer can tell a
/// refusal by the router from a router it could not reach.
#[derive(Debug, thiserror::Error)]
pub enum RouterosError {
    /// RouterOS answered with a non-2xx status. `message` and `detail` come
    /// from its `{"error", "message", "detail"}` body when it sent one.
    #[error("RouterOS returned HTTP {status}{}", describe(message, detail))]
    Status {
        status: StatusCode,
        message: Option<String>,
        detail: Option<String>,
    },
    /// No HTTP response: connection, TLS or timeout failure.
    #[error("cannot reach RouterOS: {0}")]
    Transport(#[source] reqwest::Error),
}

fn describe(message: &Option<String>, detail: &Option<String>) -> String {
    match (message, detail) {
        (Some(m), Some(d)) => format!(": {m}: {d}"),
        (Some(s), None) | (None, Some(s)) => format!(": {s}"),
        (None, None) => String::new(),
    }
}

/// The error body RouterOS sends with a non-2xx status, e.g.
/// `{"error": 400, "message": "Bad Request", "detail": "no such item"}`.
#[derive(serde::Deserialize)]
struct ErrorBody {
    message: Option<String>,
    detail: Option<String>,
}

/// How much of a non-JSON error body to keep.
const MAX_RAW_ERROR_BODY: usize = 200;

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
        scheme: Scheme,
        port: u16,
        username: &str,
        password: &str,
        tls: &Tls,
    ) -> Result<Self> {
        let mut builder = Client::builder();
        match tls {
            Tls::Insecure => builder = builder.danger_accept_invalid_certs(true),
            Tls::Verify { ca_file: None } => {}
            Tls::Verify {
                ca_file: Some(path),
            } => {
                // Trust only what the file holds: a router behind a
                // self-signed certificate is pinned, not merely accepted.
                let pem = std::fs::read(path)
                    .with_context(|| format!("MIKROTIK_CA_FILE: cannot read {}", path.display()))?;
                let certs = reqwest::Certificate::from_pem_bundle(&pem)
                    .with_context(|| format!("MIKROTIK_CA_FILE: {} is not PEM", path.display()))?;
                anyhow::ensure!(
                    !certs.is_empty(),
                    "MIKROTIK_CA_FILE: {} holds no PEM certificate",
                    path.display()
                );
                builder = builder.tls_built_in_root_certs(false);
                for cert in certs {
                    builder = builder.add_root_certificate(cert);
                }
            }
        }
        let client = builder.build().context("failed to build HTTP client")?;

        Ok(Self {
            base_url: format!("{}://{}:{}/rest", scheme.as_str(), host, port),
            host: host.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            client,
        })
    }

    /// Sends `req` with credentials and turns a non-2xx answer into a
    /// [`RouterosError::Status`] that carries RouterOS's own explanation.
    async fn send(&self, req: RequestBuilder) -> Result<Response> {
        let resp = req
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(RouterosError::Transport)?;
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        let text = resp.text().await.unwrap_or_default();
        let (message, detail) = match serde_json::from_str::<ErrorBody>(&text) {
            Ok(body) => (body.message, body.detail),
            Err(_) if text.trim().is_empty() => (None, None),
            Err(_) => (
                None,
                Some(text.trim().chars().take(MAX_RAW_ERROR_BODY).collect()),
            ),
        };
        Err(RouterosError::Status {
            status,
            message,
            detail,
        }
        .into())
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.send(self.client.get(&url))
            .await?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    /// Runs a command that only reads (`monitor once`, `print`), so it is
    /// not logged as a write. Every other `POST` goes through
    /// [`post`](Self::post).
    pub async fn post_read<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.send(self.client.post(&url).json(body))
            .await?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    /// Runs a command (`POST /rest/<menu>/<command>`) that changes state.
    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        log_write("POST", path, body_id(body).as_deref());
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.send(self.client.post(&url).json(body))
            .await?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    /// Adds a new item to a RouterOS menu (`PUT /rest/<menu>`). RouterOS maps
    /// `PUT` to "add"; a bare `POST /rest/<menu>` is a print query and is
    /// rejected for create payloads.
    pub async fn put<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        log_write("PUT", path, None);
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.send(self.client.put(&url).json(body))
            .await?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    /// Updates properties of an existing item (`PATCH /rest/<menu>/<id>`).
    /// RouterOS maps `PATCH` to "set"; only the supplied properties change.
    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        id: &RouterOsId,
        body: &B,
    ) -> Result<T> {
        log_write("PATCH", path, Some(id.as_str()));
        let url = format!(
            "{}/{}/{}",
            self.base_url,
            path.trim_start_matches('/'),
            id.as_str()
        );
        self.send(self.client.patch(&url).json(body))
            .await?
            .json()
            .await
            .context("failed to parse JSON response")
    }

    pub async fn post_void<B: Serialize>(&self, path: &str, body: &B) -> Result<()> {
        log_write("POST", path, body_id(body).as_deref());
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        self.send(self.client.post(&url).json(body)).await?;
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

    /// Pulls `filename` from the router over FTP. FTP is cleartext: the
    /// router password crosses the network unencrypted (see README).
    pub async fn ftp_download(&self, filename: &str) -> Result<Vec<u8>> {
        let (cmd, config) = self.ftp_command(None, filename);
        let output = run_curl(cmd, &config)
            .await
            .context("curl FTP: failed to run")?;

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

    /// Pushes `local_path` to the router over FTP. Cleartext, like
    /// [`ftp_download`](Self::ftp_download).
    pub async fn ftp_upload(&self, local_path: &str, remote_filename: &str) -> Result<()> {
        let (cmd, config) = self.ftp_command(Some(local_path), remote_filename);
        let output = run_curl(cmd, &config)
            .await
            .context("curl FTP upload: failed to run")?;

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

    /// Builds the curl invocation for an FTP transfer. The credentials are
    /// returned separately as a curl config (`--config -`) to be written to
    /// curl's stdin: anything on the command line is readable by every local
    /// user through `ps` or `/proc/<pid>/cmdline`.
    fn ftp_command(
        &self,
        upload_from: Option<&str>,
        remote_filename: &str,
    ) -> (tokio::process::Command, String) {
        let mut cmd = tokio::process::Command::new("curl");
        cmd.args(["--silent", "--fail", "--config", "-"]);
        if let Some(local_path) = upload_from {
            cmd.arg("-T").arg(local_path);
        }
        cmd.arg(format!("ftp://{}:21/{}", self.host, remote_filename));
        let config = format!(
            "user = \"{}\"\n",
            curl_config_quote(&format!("{}:{}", self.username, self.password))
        );
        (cmd, config)
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

    pub async fn delete(&self, path: &str, id: &RouterOsId) -> Result<()> {
        log_write("DELETE", path, Some(id.as_str()));
        let url = format!(
            "{}/{}/{}",
            self.base_url,
            path.trim_start_matches('/'),
            id.as_str()
        );
        self.send(self.client.delete(&url)).await?;
        Ok(())
    }
}

/// Logs one line per request that changes router state. Runs inside the
/// `tool` span opened by the MCP handler, so the line names the tool too.
/// Only the method, menu path and item id are logged, never the body, which
/// can hold passwords.
fn log_write(method: &str, path: &str, id: Option<&str>) {
    tracing::info!(
        method,
        path = path.trim_start_matches('/'),
        id = id.unwrap_or("-"),
        "RouterOS write"
    );
}

/// The item a command body addresses: its `.id`, or `numbers` (an id or a
/// name, as in `/interface enable numbers=ether1`).
fn body_id<B: Serialize>(body: &B) -> Option<String> {
    let value = serde_json::to_value(body).ok()?;
    [".id", "numbers"]
        .iter()
        .find_map(|k| value.get(*k).and_then(|v| v.as_str()).map(str::to_string))
}

/// Runs `cmd`, feeding `config` to its stdin, and collects its output.
async fn run_curl(
    mut cmd: tokio::process::Command,
    config: &str,
) -> std::io::Result<std::process::Output> {
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().expect("stdin is piped");
    stdin.write_all(config.as_bytes()).await?;
    drop(stdin);
    child.wait_with_output().await
}

/// Escapes a value for a double-quoted curl config parameter, where `\`
/// starts an escape sequence.
fn curl_config_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Why ids are parsed: the URL parser resolves `..`, so a raw id is a
    /// path into any other menu.
    #[test]
    fn url_parser_resolves_dot_segments_in_ids() {
        let url =
            reqwest::Url::parse("https://router/rest/ip/dns/static/*1/../../../firewall/filter/*5")
                .unwrap();
        assert_eq!(url.path(), "/rest/ip/firewall/filter/*5");
    }

    #[tokio::test]
    async fn delete_addresses_the_item_under_its_own_menu() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/rest/ip/dns/static/*1A"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let client = RouterosClient::for_test(&server.uri());
        let id = RouterOsId::try_from("1A").unwrap();
        client.delete("ip/dns/static", &id).await.unwrap();
    }

    fn argv(cmd: &tokio::process::Command) -> Vec<String> {
        cmd.as_std()
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn ftp_password_is_never_on_curl_argv() {
        let client = RouterosClient::for_test("http://router");
        for (cmd, config) in [
            client.ftp_command(None, "r.backup"),
            client.ftp_command(Some("/tmp/r.backup"), "r.backup"),
        ] {
            let args = argv(&cmd);
            assert!(
                args.iter().all(|a| !a.contains("test")),
                "password on argv: {args:?}"
            );
            assert!(
                args.iter().any(|a| a == "-"),
                "config not read from stdin: {args:?}"
            );
            assert_eq!(config, "user = \"admin:test\"\n");
        }
    }

    #[test]
    fn curl_config_quote_escapes_quotes_and_backslashes() {
        assert_eq!(curl_config_quote(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(curl_config_quote("a\nb"), r"a\nb");
    }

    /// A throwaway self-signed CA, only ever parsed here.
    const TEST_CA_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBljCCAT2gAwIBAgIUGYsotU+zqFUaUvzKT2bn3ksNjRswCgYIKoZIzj0EAwIw\nGTEXMBUGA1UEAwwOdGVzdC1yb3V0ZXItY2EwHhcNMjYxMTA2MjExMzQ3WhcNMzYx\nMTAzMjExMzQ3WjAZMRcwFQYDVQQDDA50ZXN0LXJvdXRlci1jYTBZMBMGByqGSM49\nAgEGCCqGSM49AwEHA0IABNz/JbCyVDeENSjvDOZyKYSem+9w3hg6/9OHav55Lf28\n1UxTR8RPjpluBno1gqKJl0sUhje7IL4Z6ARoRaf6Fp2jYzBhMB0GA1UdDgQWBBT2\ndymnFHRXT0JESHLoQUvWlOGD+jAfBgNVHSMEGDAWgBT2dymnFHRXT0JESHLoQUvW\nlOGD+jAPBgNVHRMBAf8EBTADAQH/MA4GA1UdDwEB/wQEAwICBDAKBggqhkjOPQQD\nAgNHADBEAiA0AGaupp29/cyo5+ughsx7SJ1KvnMVcZo27lu2VZaQUgIgQQwZ88JB\nDy7uULaSemtlQ47tkOXJm0KRF3LJ9x/JKbk=\n-----END CERTIFICATE-----\n";

    fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("mikrotik-mcp-{}-{name}", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn with_ca(path: std::path::PathBuf) -> Result<RouterosClient> {
        let tls = Tls::Verify {
            ca_file: Some(path),
        };
        RouterosClient::new("192.0.2.1", Scheme::Https, 443, "admin", "pw", &tls)
    }

    #[test]
    fn ca_file_with_a_pem_certificate_builds_a_client() {
        let client = with_ca(write_temp("ca.pem", TEST_CA_PEM)).unwrap();
        assert_eq!(client.base_url, "https://192.0.2.1:443/rest");
    }

    #[test]
    fn ca_file_without_a_certificate_is_an_error() {
        let err = with_ca(write_temp("junk.pem", "not a certificate\n"))
            .err()
            .unwrap();
        assert!(err.to_string().contains("MIKROTIK_CA_FILE"), "{err}");
        let err = with_ca("/nonexistent/ca.pem".into()).err().unwrap();
        assert!(err.to_string().contains("cannot read"), "{err}");
    }

    #[test]
    fn scheme_is_taken_from_config_not_guessed_from_the_port() {
        let tls = Tls::Verify { ca_file: None };
        let client = RouterosClient::new("r", Scheme::Https, 80, "u", "p", &tls).unwrap();
        assert_eq!(client.base_url, "https://r:80/rest");
        let client =
            RouterosClient::new("r", Scheme::Http, 8080, "u", "p", &Tls::Insecure).unwrap();
        assert_eq!(client.base_url, "http://r:8080/rest");
    }
}
