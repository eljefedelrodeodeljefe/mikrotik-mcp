use serde_json::Value;

use crate::client::RouterosClient;

pub async fn list_disks(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("disk").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_disks_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/disk"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "slot": "usb1-part1", "type": "partition", "fs": "fat32"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_disks(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["slot"], "usb1-part1");
    }
}
