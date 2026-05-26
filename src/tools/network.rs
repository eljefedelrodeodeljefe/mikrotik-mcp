use serde_json::Value;

use crate::client::RouterosClient;

pub async fn list_routes(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/route").await
}

pub async fn list_neighbors(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/neighbor").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_routes_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/route"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"dst-address": "0.0.0.0/0", "gateway": "192.168.1.1"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_routes(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["gateway"], "192.168.1.1");
    }

    #[tokio::test]
    async fn list_neighbors_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/neighbor"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_neighbors(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }
}
