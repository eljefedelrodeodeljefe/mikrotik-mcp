use serde_json::Value;

use crate::client::RouterosClient;
use crate::params::RemoveFileParams;

/// Lists files on the device — `/file`. Returns name, type, size, and
/// creation time. Use to find backup files (e.g. left by save_backup) or to
/// look up the `.id` needed for removal.
pub async fn list_files(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("file").await
}

/// Removes a file by name — resolves the name to its `.id` then
/// `DELETE /file/<id>`. Primarily for cleaning up backup files left on the
/// device after save_backup has downloaded them.
pub async fn remove_file(client: &RouterosClient, p: &RemoveFileParams) -> anyhow::Result<()> {
    let files: Value = client.get("file").await?;
    let id = files
        .as_array()
        .into_iter()
        .flatten()
        .find(|f| f.get("name").and_then(Value::as_str) == Some(p.name.as_str()))
        .and_then(|f| f.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("no file named '{}'", p.name))?
        .to_string();
    client.delete("file", &id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_files_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/file"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "chateau.backup", "size": "42000"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_files(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["name"], "chateau.backup");
    }

    #[tokio::test]
    async fn remove_file_resolves_name_then_deletes() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/file"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*9", "name": "old.backup"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("DELETE"))
            .and(path("/rest/file/*9"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = RemoveFileParams {
            name: "old.backup".into(),
        };
        remove_file(&client, &p).await.unwrap();
    }
}
