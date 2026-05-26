use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{AddFirewallFilterParams, AddFirewallNatParams};

pub async fn list_filter(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/firewall/filter").await
}

pub async fn add_filter(
    client: &RouterosClient,
    p: &AddFirewallFilterParams,
) -> anyhow::Result<Value> {
    let mut body = json!({"chain": p.chain, "action": p.action});
    if let Some(v) = &p.src_address {
        body["src-address"] = json!(v);
    }
    if let Some(v) = &p.dst_address {
        body["dst-address"] = json!(v);
    }
    if let Some(v) = &p.protocol {
        body["protocol"] = json!(v);
    }
    if let Some(v) = &p.dst_port {
        body["dst-port"] = json!(v);
    }
    if let Some(v) = &p.in_interface {
        body["in-interface"] = json!(v);
    }
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    if let Some(v) = p.disabled {
        body["disabled"] = json!(v);
    }
    client.post("ip/firewall/filter", &body).await
}

pub async fn remove_filter(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/firewall/filter", id).await
}

pub async fn list_nat(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/firewall/nat").await
}

pub async fn add_nat(client: &RouterosClient, p: &AddFirewallNatParams) -> anyhow::Result<Value> {
    let mut body = json!({"chain": p.chain, "action": p.action});
    if let Some(v) = &p.src_address {
        body["src-address"] = json!(v);
    }
    if let Some(v) = &p.dst_address {
        body["dst-address"] = json!(v);
    }
    if let Some(v) = &p.protocol {
        body["protocol"] = json!(v);
    }
    if let Some(v) = &p.dst_port {
        body["dst-port"] = json!(v);
    }
    if let Some(v) = &p.to_addresses {
        body["to-addresses"] = json!(v);
    }
    if let Some(v) = &p.to_ports {
        body["to-ports"] = json!(v);
    }
    if let Some(v) = &p.out_interface {
        body["out-interface"] = json!(v);
    }
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    client.post("ip/firewall/nat", &body).await
}

pub async fn remove_nat(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/firewall/nat", id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_filter_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/filter"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_filter(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_nat_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/nat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_nat(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }
}
