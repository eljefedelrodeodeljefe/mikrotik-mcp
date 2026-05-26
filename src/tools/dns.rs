use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::AddDnsStaticParams;

pub async fn get_settings(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dns").await
}

pub async fn list_static(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dns/static").await
}

pub async fn add_static(client: &RouterosClient, p: &AddDnsStaticParams) -> anyhow::Result<Value> {
    let mut body = json!({"name": p.name, "address": p.address});
    if let Some(ttl) = p.ttl {
        body["ttl"] = json!(format!("{}s", ttl));
    }
    if let Some(c) = &p.comment {
        body["comment"] = json!(c);
    }
    client.post("ip/dns/static", &body).await
}

pub async fn remove_static(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/dns/static", id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_settings_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dns"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(
                {"servers": "1.1.1.1", "cache-max-ttl": "1w"}
            )))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = get_settings(&client).await.unwrap();
        assert_eq!(result["servers"], "1.1.1.1");
    }

    #[tokio::test]
    async fn list_static_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dns/static"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"name": "nas.home.arpa", "address": "192.168.1.10"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_static(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["name"], "nas.home.arpa");
    }
}
