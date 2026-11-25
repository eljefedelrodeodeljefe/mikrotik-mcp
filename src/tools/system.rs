use anyhow::Context;
use serde_json::{Value, json};

use super::Props;
use crate::client::{RouterosClient, RouterosError, SftpTarget};
use crate::params::{
    GetLogsParams, GetPackageUpdateParams, RestoreBackupParams, SaveBackupParams, SetClockParams,
    SetNtpClientParams, SetNtpServerParams, SetSystemIdentityParams,
};

pub async fn get_resources(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/resource").await
}

pub async fn get_identity(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/identity").await
}

pub async fn set_identity(
    client: &RouterosClient,
    p: &SetSystemIdentityParams,
) -> anyhow::Result<Value> {
    client
        .post("system/identity/set", &json!({"name": p.name}))
        .await
}

pub async fn get_logs(client: &RouterosClient, p: &GetLogsParams) -> anyhow::Result<Value> {
    let mut entries: Vec<Value> = client.get("log").await?;
    if let Some(topic) = &p.topics {
        entries.retain(|e| {
            e.get("topics")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t.contains(topic.as_str()))
        });
    }
    // RouterOS returns the log oldest first; keep the most recent `count`.
    let count = p.count.unwrap_or(50) as usize;
    entries.drain(..entries.len().saturating_sub(count));
    Ok(Value::Array(entries))
}

pub async fn save_backup(
    client: &RouterosClient,
    p: &SaveBackupParams,
    password: &str,
    encrypt: bool,
    export: Option<&SftpTarget>,
) -> anyhow::Result<String> {
    let mut body = serde_json::json!({"name": p.name});
    if encrypt {
        let pw = p.password.as_deref().unwrap_or(password);
        body["password"] = serde_json::json!(pw);
    }

    client
        .post_void("system/backup/save", &body)
        .await
        .context("step 1: POST system/backup/save failed")?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let filename = format!("{}.backup", p.name);
    let encryption = if encrypt { "encrypted" } else { "unencrypted" };

    // Preferred, hardening-compatible path: the router pushes the backup out
    // over SFTP (egress only). Works even when inbound FTP is disabled.
    if let Some(target) = export {
        client
            .sftp_push(&filename, &filename, target)
            .await
            .context("step 2: SFTP push failed")?;
        return Ok(format!(
            "backup {filename} saved on device and pushed via SFTP to {}:{}{} ({encryption})",
            target.host,
            target.port,
            if target.path.trim().is_empty() {
                String::new()
            } else {
                format!("/{}", target.path.trim_matches('/'))
            },
        ));
    }

    // Legacy path: pull the file to the local machine over FTP. Requires FTP to
    // be enabled on the device and an output_path to write to.
    let output_path = p.output_path.as_deref().context(
        "output_path is required when no SFTP export destination (MIKROTIK_BACKUP_SFTP_HOST) is configured",
    )?;
    let bytes = client
        .ftp_download(&filename)
        .await
        .context("step 2: FTP download failed (FTP may be disabled on the device — set MIKROTIK_BACKUP_SFTP_HOST to push via SFTP instead)")?;

    let path = std::path::Path::new(output_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("step 3: create_dir_all: {output_path}"))?;
    }
    std::fs::write(path, &bytes).with_context(|| format!("step 3: write file: {output_path}"))?;

    Ok(format!(
        "backup saved to {output_path} ({} bytes, {encryption})",
        bytes.len(),
    ))
}

pub async fn restore_backup(
    client: &RouterosClient,
    p: &RestoreBackupParams,
    password: &str,
    encrypt: bool,
) -> anyhow::Result<String> {
    let path = std::path::Path::new(&p.input_path);
    let remote_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("invalid input_path")?
        .to_string();

    client
        .ftp_upload(&p.input_path, &remote_name)
        .await
        .context("step 1: FTP upload failed")?;

    let name_without_ext = remote_name.strip_suffix(".backup").unwrap_or(&remote_name);
    let mut body = serde_json::json!({"name": name_without_ext});
    if encrypt {
        let pw = p.password.as_deref().unwrap_or(password);
        body["password"] = serde_json::json!(pw);
    }

    client
        .post_void("system/backup/load", &body)
        .await
        .context("step 2: POST system/backup/load failed")?;

    Ok(format!(
        "backup {} loaded — device is rebooting",
        remote_name
    ))
}

/// NTP client and server settings together, plus the client's sync state.
pub async fn get_ntp(client: &RouterosClient) -> anyhow::Result<Value> {
    let ntp_client: Value = client.get("system/ntp/client").await?;
    let ntp_server: Value = client.get("system/ntp/server").await?;
    Ok(json!({"client": ntp_client, "server": ntp_server}))
}

/// Copies the given optional fields into a `set` body; at least one must be set.
fn set_body(fields: &[(&str, &Option<String>)]) -> anyhow::Result<Value> {
    let mut body = json!({});
    for (key, value) in fields {
        if let Some(v) = value {
            body[*key] = json!(v);
        }
    }
    anyhow::ensure!(
        body.as_object().is_some_and(|o| !o.is_empty()),
        "nothing to set: pass at least one field"
    );
    Ok(body)
}

pub async fn set_ntp_client(
    client: &RouterosClient,
    p: &SetNtpClientParams,
) -> anyhow::Result<Value> {
    let body = set_body(&[
        ("enabled", &p.enabled),
        ("servers", &p.servers),
        ("mode", &p.mode),
    ])?;
    client.post("system/ntp/client/set", &body).await
}

pub async fn set_ntp_server(
    client: &RouterosClient,
    p: &SetNtpServerParams,
) -> anyhow::Result<Value> {
    let body = set_body(&[("enabled", &p.enabled), ("broadcast", &p.broadcast)])?;
    client.post("system/ntp/server/set", &body).await
}

pub async fn get_clock(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/clock").await
}

pub async fn set_clock(client: &RouterosClient, p: &SetClockParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .text("date", p.date.as_deref())
        .text("time", p.time.as_deref())
        .flag("time-zone-autodetect", p.time_zone_autodetect)
        .text("time-zone-name", p.time_zone_name.as_deref())
        .into_changes("set_clock")?;
    client.post("system/clock/set", &body).await
}

pub async fn list_packages(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/package").await
}

/// The update channel, installed and latest version and status of
/// `/system/package/update`, after an optional `check-for-updates` whose
/// final status is added as `check-status`.
pub async fn get_package_update(
    client: &RouterosClient,
    p: &GetPackageUpdateParams,
) -> anyhow::Result<Value> {
    let check_status = if p.check_for_updates.unwrap_or(false) {
        // Changes no configuration, only asks the download server. With a
        // duration the command returns its progress entries after that time;
        // the last one holds the final status.
        let progress: Vec<Value> = client
            .post_read(
                "system/package/update/check-for-updates",
                &json!({"duration": CHECK_FOR_UPDATES_DURATION}),
            )
            .await
            .context("check-for-updates failed")?;
        progress.last().and_then(|e| e.get("status")).cloned()
    } else {
        None
    };
    let mut update: Value = client.get("system/package/update").await?;
    if let (Some(status), Some(obj)) = (check_status, update.as_object_mut()) {
        obj.insert("check-status".into(), status);
    }
    Ok(update)
}

/// How long `check-for-updates` runs before the REST call returns.
const CHECK_FOR_UPDATES_DURATION: &str = "15s";

pub async fn get_device_mode(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/device-mode").await
}

/// Runs a command after which the router restarts. The router may close the
/// connection before it answers; that counts as success. A connection that
/// never opened does not.
async fn run_restarting(client: &RouterosClient, path: &str, done: &str) -> anyhow::Result<String> {
    match client.post_void(path, &json!({})).await {
        Ok(()) => Ok(done.to_string()),
        Err(e) => match e.downcast_ref::<RouterosError>() {
            Some(RouterosError::Transport(t)) if !t.is_connect() => Ok(format!(
                "{done} (the router closed the connection without answering, as it does when it \
                    restarts)"
            )),
            _ => Err(e),
        },
    }
}

pub async fn reboot(client: &RouterosClient) -> anyhow::Result<String> {
    run_restarting(
        client,
        "system/reboot",
        "reboot started: the router is unreachable until it is back up",
    )
    .await
}

pub async fn install_package_update(client: &RouterosClient) -> anyhow::Result<String> {
    run_restarting(
        client,
        "system/package/update/install",
        "update install started: the router downloads the packages, then reboots into the new \
            version",
    )
    .await
}

pub async fn upgrade_routerboard(client: &RouterosClient) -> anyhow::Result<Value> {
    client.post("system/routerboard/upgrade", &json!({})).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn save_params(output_path: Option<&str>) -> SaveBackupParams {
        SaveBackupParams {
            name: "chateau-test".into(),
            output_path: output_path.map(Into::into),
            password: None,
        }
    }

    #[tokio::test]
    async fn clock_get_and_set() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/system/clock"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"date": "2026-01-31", "time": "13:05:00", "time-zone-name": "UTC"}),
            ))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/system/clock/set"))
            .and(body_json(json!({
                "time-zone-autodetect": "false",
                "time-zone-name": "Europe/Berlin"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        assert_eq!(get_clock(&client).await.unwrap()["date"], "2026-01-31");
        let p = SetClockParams {
            date: None,
            time: None,
            time_zone_name: Some("Europe/Berlin".into()),
            time_zone_autodetect: Some(false),
        };
        set_clock(&client, &p).await.unwrap();
        let empty = SetClockParams {
            date: None,
            time: None,
            time_zone_name: None,
            time_zone_autodetect: None,
        };
        assert!(set_clock(&client, &empty).await.is_err());
    }

    #[tokio::test]
    async fn package_update_checks_only_when_asked() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/package/update/check-for-updates"))
            .and(body_json(json!({"duration": "15s"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".section": "0", "channel": "stable", "status": "finding out latest version..."},
                {".section": "1", "channel": "stable", "status": "System is already up to date"}
            ])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/system/package/update"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "channel": "stable",
                "installed-version": "7.20",
                "latest-version": "7.21",
                "status": "New version is available"
            })))
            .expect(2)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/system/package"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!([{"name": "routeros", "version": "7.20"}])),
            )
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let listed = list_packages(&client).await.unwrap();
        assert_eq!(listed[0]["name"], "routeros");
        let checked = GetPackageUpdateParams {
            check_for_updates: Some(true),
        };
        let out = get_package_update(&client, &checked).await.unwrap();
        assert_eq!(out["latest-version"], "7.21");
        assert_eq!(out["check-status"], "System is already up to date");
        let read_only = GetPackageUpdateParams {
            check_for_updates: None,
        };
        let out = get_package_update(&client, &read_only).await.unwrap();
        assert!(out.get("check-status").is_none());
    }

    #[tokio::test]
    async fn reboot_counts_a_dropped_connection_as_started() {
        use tokio::io::AsyncReadExt;
        // A peer that reads the request and hangs up without answering,
        // as a rebooting router does.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = sock.read(&mut buf).await;
        });
        let client = RouterosClient::for_test(&format!("http://{addr}"));
        let msg = reboot(&client).await.unwrap();
        assert!(msg.contains("closed the connection"), "{msg}");
    }

    #[tokio::test]
    async fn reboot_reports_an_unreachable_router() {
        // Nothing listens on port 9 (discard) here.
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        assert!(reboot(&client).await.is_err());
    }

    #[tokio::test]
    async fn install_and_firmware_upgrade_post_their_commands() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/package/update/install"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/system/routerboard/upgrade"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/system/device-mode"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"mode": "advanced", "container": "false"})),
            )
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let msg = install_package_update(&client).await.unwrap();
        assert!(msg.contains("reboots"), "{msg}");
        upgrade_routerboard(&client).await.unwrap();
        assert_eq!(get_device_mode(&client).await.unwrap()["mode"], "advanced");
    }

    #[tokio::test]
    async fn save_backup_pushes_via_sftp_when_export_configured() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/backup/save"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/tool/fetch"))
            .and(wiremock::matchers::body_partial_json(
                json!({"upload": "yes", "mode": "sftp", "address": "backup-host"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "finished"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let target = SftpTarget {
            host: "backup-host".into(),
            port: 22,
            user: "svc".into(),
            password: "pw".into(),
            path: "mikrotik".into(),
        };
        // output_path is irrelevant in SFTP mode.
        let msg = save_backup(
            &client,
            &save_params(None),
            "fallback-pw",
            true,
            Some(&target),
        )
        .await
        .unwrap();
        assert!(msg.contains("SFTP"), "got: {msg}");
        assert!(msg.contains("backup-host"), "got: {msg}");
    }

    #[tokio::test]
    async fn save_backup_errors_without_output_path_or_export() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/backup/save"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let err = save_backup(&client, &save_params(None), "pw", true, None)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("output_path is required"),
            "got: {err}"
        );
    }

    #[tokio::test]
    async fn get_resources_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/system/resource"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"cpu-load": 5})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = get_resources(&client).await.unwrap();
        assert_eq!(result["cpu-load"], 5);
    }

    #[tokio::test]
    async fn get_identity_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/system/identity"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name": "r-ap-1"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = get_identity(&client).await.unwrap();
        assert_eq!(result["name"], "r-ap-1");
    }

    #[tokio::test]
    async fn set_identity_posts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/identity/set"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = crate::params::SetSystemIdentityParams {
            name: "r-atl-5g".into(),
        };
        let result = set_identity(&client, &p).await.unwrap();
        assert!(result.is_object());
    }

    #[tokio::test]
    async fn get_logs_filters_by_topic() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/log"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"topics": "dhcp,info",     "message": "assigned"},
                {"topics": "firewall,info", "message": "blocked"},
                {"topics": "dhcp,error",    "message": "failed"},
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = GetLogsParams {
            topics: Some("dhcp".into()),
            count: None,
        };
        let result = get_logs(&client, &p).await.unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["message"], "assigned");
        assert_eq!(arr[1]["message"], "failed");
    }

    #[tokio::test]
    async fn get_logs_keeps_the_most_recent_count() {
        let entries: Vec<_> = (0..60u32).map(|i| json!({"n": i})).collect();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/log"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(entries)))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = GetLogsParams {
            topics: None,
            count: Some(10),
        };
        let result = get_logs(&client, &p).await.unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 10);
        assert_eq!(arr[0]["n"], 50, "should keep the most recent entries");
        assert_eq!(arr[9]["n"], 59);
    }

    #[tokio::test]
    async fn get_ntp_reads_client_and_server() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/system/ntp/client"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"enabled": "true", "servers": "0.pool.ntp.org", "status": "synchronized"}),
            ))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/system/ntp/server"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"enabled": "false"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = get_ntp(&client).await.unwrap();
        assert_eq!(result["client"]["status"], "synchronized");
        assert_eq!(result["server"]["enabled"], "false");
    }

    #[tokio::test]
    async fn set_ntp_client_posts_only_given_fields() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/ntp/client/set"))
            .and(body_json(
                json!({"enabled": "yes", "servers": "0.pool.ntp.org"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetNtpClientParams {
            enabled: Some("yes".into()),
            servers: Some("0.pool.ntp.org".into()),
            mode: None,
        };
        set_ntp_client(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn set_ntp_server_posts_to_set_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/ntp/server/set"))
            .and(body_json(json!({"enabled": "yes"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetNtpServerParams {
            enabled: Some("yes".into()),
            broadcast: None,
        };
        set_ntp_server(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn set_ntp_with_no_fields_is_refused() {
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        let p = SetNtpServerParams {
            enabled: None,
            broadcast: None,
        };
        let err = set_ntp_server(&client, &p).await.unwrap_err();
        assert!(err.to_string().contains("nothing to set"), "{err}");
    }
}
