use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::AddIpAddressParams;

pub async fn list_addresses(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/address").await
}

pub async fn add_address(client: &RouterosClient, p: &AddIpAddressParams) -> anyhow::Result<Value> {
    let mut body = json!({"address": p.address, "interface": p.interface});
    if let Some(c) = &p.comment {
        body["comment"] = json!(c);
    }
    client.post("ip/address", &body).await
}

pub async fn remove_address(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/address", id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_addresses_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/address"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"address": "192.168.1.1/24", "interface": "bridge"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_addresses(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["address"], "192.168.1.1/24");
    }
}
