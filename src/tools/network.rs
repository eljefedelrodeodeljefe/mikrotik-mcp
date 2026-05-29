use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::AddRouteParams;

pub async fn list_routes(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/route").await
}

pub async fn add_route(client: &RouterosClient, p: &AddRouteParams) -> anyhow::Result<Value> {
    let mut body =
        json!({"dst-address": p.dst_address, "gateway": p.gateway, "distance": p.distance});
    if let Some(v) = &p.check_gateway {
        body["check-gateway"] = json!(v);
    }
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    client.put("ip/route", &body).await
}

pub async fn remove_route(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/route", id).await
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
    async fn add_route_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/route"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*2",
                "dst-address": "0.0.0.0/0",
                "gateway": "192.168.188.1",
                "distance": 2
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddRouteParams {
            dst_address: "0.0.0.0/0".into(),
            gateway: "192.168.188.1".into(),
            distance: 2,
            check_gateway: Some("ping".into()),
            comment: None,
        };
        let result = add_route(&client, &p).await.unwrap();
        assert_eq!(result["gateway"], "192.168.188.1");
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
