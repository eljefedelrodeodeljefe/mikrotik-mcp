use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{
    AddFirewallAddressListParams, AddFirewallFilterParams, AddFirewallMangleParams,
    AddFirewallNatParams,
};

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
    client.put("ip/firewall/filter", &body).await
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
    client.put("ip/firewall/nat", &body).await
}

pub async fn remove_nat(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/firewall/nat", id).await
}

pub async fn list_mangle(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/firewall/mangle").await
}

pub async fn add_mangle(
    client: &RouterosClient,
    p: &AddFirewallMangleParams,
) -> anyhow::Result<Value> {
    let mut body = json!({"chain": p.chain, "action": p.action});
    if let Some(v) = &p.new_mss {
        body["new-mss"] = json!(v);
    }
    if let Some(v) = p.passthrough {
        body["passthrough"] = json!(v);
    }
    if let Some(v) = &p.protocol {
        body["protocol"] = json!(v);
    }
    if let Some(v) = &p.tcp_flags {
        body["tcp-flags"] = json!(v);
    }
    if let Some(v) = &p.src_address {
        body["src-address"] = json!(v);
    }
    if let Some(v) = &p.dst_address {
        body["dst-address"] = json!(v);
    }
    if let Some(v) = &p.in_interface {
        body["in-interface"] = json!(v);
    }
    if let Some(v) = &p.out_interface {
        body["out-interface"] = json!(v);
    }
    if let Some(v) = &p.in_interface_list {
        body["in-interface-list"] = json!(v);
    }
    if let Some(v) = &p.out_interface_list {
        body["out-interface-list"] = json!(v);
    }
    if let Some(v) = &p.connection_mark {
        body["connection-mark"] = json!(v);
    }
    if let Some(v) = &p.new_connection_mark {
        body["new-connection-mark"] = json!(v);
    }
    if let Some(v) = &p.new_routing_mark {
        body["new-routing-mark"] = json!(v);
    }
    if let Some(v) = &p.new_packet_mark {
        body["new-packet-mark"] = json!(v);
    }
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    client.put("ip/firewall/mangle", &body).await
}

pub async fn remove_mangle(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/firewall/mangle", id).await
}

pub async fn list_address_list(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/firewall/address-list").await
}

pub async fn add_address_list(
    client: &RouterosClient,
    p: &AddFirewallAddressListParams,
) -> anyhow::Result<Value> {
    let mut body = json!({"list": p.list, "address": p.address});
    if let Some(v) = &p.timeout {
        body["timeout"] = json!(v);
    }
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    client.put("ip/firewall/address-list", &body).await
}

pub async fn remove_address_list(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/firewall/address-list", id).await
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

    #[tokio::test]
    async fn add_nat_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/nat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*5", "chain": "srcnat", "action": "masquerade"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallNatParams {
            chain: "srcnat".into(),
            action: "masquerade".into(),
            src_address: None,
            dst_address: None,
            protocol: None,
            dst_port: None,
            to_addresses: None,
            to_ports: None,
            out_interface: Some("bridge".into()),
            comment: None,
        };
        let result = add_nat(&client, &p).await.unwrap();
        assert_eq!(result["action"], "masquerade");
    }

    #[tokio::test]
    async fn add_filter_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/filter"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*6", "chain": "input", "action": "accept"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallFilterParams {
            chain: "input".into(),
            action: "accept".into(),
            src_address: None,
            dst_address: None,
            protocol: None,
            dst_port: None,
            in_interface: None,
            comment: None,
            disabled: None,
        };
        let result = add_filter(&client, &p).await.unwrap();
        assert_eq!(result["action"], "accept");
    }

    #[tokio::test]
    async fn list_address_list_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/address-list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"list": "local-subnets", "address": "192.168.88.0/24"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_address_list(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["list"], "local-subnets");
    }

    #[tokio::test]
    async fn add_address_list_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/address-list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*7", "list": "local-subnets", "address": "10.0.0.0/8"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallAddressListParams {
            list: "local-subnets".into(),
            address: "10.0.0.0/8".into(),
            timeout: None,
            comment: None,
        };
        let result = add_address_list(&client, &p).await.unwrap();
        assert_eq!(result["list"], "local-subnets");
    }

    #[tokio::test]
    async fn list_mangle_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/mangle"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_mangle(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_mangle_puts_mss_clamp() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/mangle"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*4", "chain": "forward", "action": "change-mss",
                "new-mss": "clamp-to-pmtu", "out-interface-list": "WAN"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallMangleParams {
            chain: "forward".into(),
            action: "change-mss".into(),
            new_mss: Some("clamp-to-pmtu".into()),
            passthrough: Some(true),
            protocol: Some("tcp".into()),
            tcp_flags: Some("syn".into()),
            src_address: None,
            dst_address: None,
            in_interface: None,
            out_interface: None,
            in_interface_list: None,
            out_interface_list: Some("WAN".into()),
            connection_mark: None,
            new_connection_mark: None,
            new_routing_mark: None,
            new_packet_mark: None,
            comment: None,
        };
        let result = add_mangle(&client, &p).await.unwrap();
        assert_eq!(result["new-mss"], "clamp-to-pmtu");
    }
}
