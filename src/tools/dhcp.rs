use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::AddDhcpStaticLeaseParams;

pub async fn list_servers(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dhcp-server").await
}

pub async fn list_leases(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dhcp-server/lease").await
}

pub async fn add_static_lease(
    client: &RouterosClient,
    p: &AddDhcpStaticLeaseParams,
) -> anyhow::Result<Value> {
    let mut body = json!({"mac-address": p.mac_address, "address": p.address});
    if let Some(c) = &p.comment {
        body["comment"] = json!(c);
    }
    client.post("ip/dhcp-server/lease", &body).await
}

pub async fn remove_lease(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/dhcp-server/lease", id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_servers_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dhcp-server"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"name": "dhcp1", "interface": "bridge"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_servers(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["name"], "dhcp1");
    }

    #[tokio::test]
    async fn list_leases_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dhcp-server/lease"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_leases(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }
}
