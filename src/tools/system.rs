use anyhow::Context;
use serde_json::Value;

use crate::client::RouterosClient;
use crate::params::{GetLogsParams, RestoreBackupParams, SaveBackupParams};

pub async fn get_resources(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/resource").await
}

pub async fn get_identity(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/identity").await
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
    entries.truncate(p.count.unwrap_or(50) as usize);
    Ok(Value::Array(entries))
}

pub async fn save_backup(
    client: &RouterosClient,
    p: &SaveBackupParams,
    password: &str,
    encrypt: bool,
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
    let bytes = client
        .ftp_download(&filename)
        .await
        .context("step 2: FTP download failed")?;

    let path = std::path::Path::new(&p.output_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("step 3: create_dir_all: {}", p.output_path))?;
    }
    std::fs::write(path, &bytes)
        .with_context(|| format!("step 3: write file: {}", p.output_path))?;

    Ok(format!(
        "backup saved to {} ({} bytes, {})",
        p.output_path,
        bytes.len(),
        if encrypt { "encrypted" } else { "unencrypted" },
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

    Ok(format!("backup {} loaded — device is rebooting", remote_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
        let p = GetLogsParams { topics: Some("dhcp".into()), count: None };
        let result = get_logs(&client, &p).await.unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["message"], "assigned");
        assert_eq!(arr[1]["message"], "failed");
    }

    #[tokio::test]
    async fn get_logs_truncates_to_count() {
        let entries: Vec<_> = (0..60u32).map(|i| json!({"n": i})).collect();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/log"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(entries)))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = GetLogsParams { topics: None, count: Some(10) };
        let result = get_logs(&client, &p).await.unwrap();
        assert_eq!(result.as_array().unwrap().len(), 10);
    }
}
