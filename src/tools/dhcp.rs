use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{AddDhcpClientParams, AddDhcpStaticLeaseParams, SetDhcpClientParams};

pub async fn list_servers(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dhcp-server").await
}

/// Lists DHCP clients (`/ip/dhcp-client`) — the per-interface WAN DHCP config
/// incl. `add-default-route`, `default-route-distance`, and the assigned address.
pub async fn list_clients(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dhcp-client").await
}

/// Updates a DHCP client's settings, located by interface — `/ip dhcp-client set`.
/// Setting `add-default-route=no` lets you manage the WAN default route
/// statically (e.g. a `check-gateway` primary + LTE backup for failover).
pub async fn set_client(
    client: &RouterosClient,
    p: &SetDhcpClientParams,
) -> anyhow::Result<String> {
    let clients = list_clients(client).await?;
    let id = clients
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| c.get("interface").and_then(Value::as_str) == Some(p.interface.as_str()))
        .and_then(|c| c.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("no DHCP client found on interface '{}'", p.interface))?
        .to_string();

    let mut body = json!({ "numbers": id });
    if let Some(b) = p.add_default_route {
        body["add-default-route"] = json!(yes_no(b));
    }
    if let Some(d) = p.default_route_distance {
        body["default-route-distance"] = json!(d.to_string());
    }
    if let Some(b) = p.use_peer_dns {
        body["use-peer-dns"] = json!(yes_no(b));
    }
    client.post_text("ip/dhcp-client/set", &body).await?;
    Ok(format!("dhcp-client on {} updated", p.interface))
}

pub(crate) fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
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
    client.put("ip/dhcp-server/lease", &body).await
}

pub async fn remove_lease(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/dhcp-server/lease", id).await
}

/// Removes a DHCP client, located by interface name — `/ip dhcp-client remove`.
pub async fn remove_client(client: &RouterosClient, interface: &str) -> anyhow::Result<()> {
    let clients = list_clients(client).await?;
    let id = clients
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| c.get("interface").and_then(Value::as_str) == Some(interface))
        .and_then(|c| c.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("no DHCP client on interface '{}'", interface))?
        .to_string();
    client.delete("ip/dhcp-client", &id).await
}

/// Adds a new DHCP client on `interface` — `/ip dhcp-client add`.
/// Set `add_default_route=true` and `default_route_distance` to integrate it
/// into the failover routing stack.
pub async fn add_client(client: &RouterosClient, p: &AddDhcpClientParams) -> anyhow::Result<Value> {
    let mut body = json!({ "interface": p.interface });
    if let Some(b) = p.add_default_route {
        body["add-default-route"] = json!(yes_no(b));
    }
    if let Some(d) = p.default_route_distance {
        body["default-route-distance"] = json!(d.to_string());
    }
    if let Some(b) = p.use_peer_dns {
        body["use-peer-dns"] = json!(yes_no(b));
    }
    client.put("ip/dhcp-client", &body).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
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

    #[tokio::test]
    async fn add_static_lease_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/dhcp-server/lease"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*4", "mac-address": "AA:BB:CC:DD:EE:FF", "address": "192.168.88.50"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddDhcpStaticLeaseParams {
            mac_address: "AA:BB:CC:DD:EE:FF".into(),
            address: "192.168.88.50".into(),
            comment: None,
        };
        let result = add_static_lease(&client, &p).await.unwrap();
        assert_eq!(result["address"], "192.168.88.50");
    }

    #[tokio::test]
    async fn list_clients_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dhcp-client"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "interface": "ether1", "add-default-route": "yes"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_clients(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["interface"], "ether1");
    }

    #[tokio::test]
    async fn set_client_resolves_id_by_interface_and_sets() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dhcp-client"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "interface": "ether1"},
                {".id": "*2", "interface": "lte1"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/ip/dhcp-client/set"))
            .and(body_json(
                json!({"numbers": "*1", "add-default-route": "no"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetDhcpClientParams {
            interface: "ether1".into(),
            add_default_route: Some(false),
            default_route_distance: None,
            use_peer_dns: None,
        };
        let msg = set_client(&client, &p).await.unwrap();
        assert!(msg.contains("ether1"), "got: {msg}");
    }

    #[tokio::test]
    async fn set_client_errors_when_interface_absent() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dhcp-client"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "interface": "ether1"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetDhcpClientParams {
            interface: "ether9".into(),
            add_default_route: Some(false),
            default_route_distance: None,
            use_peer_dns: None,
        };
        let err = set_client(&client, &p).await.unwrap_err().to_string();
        assert!(err.contains("ether9"), "got: {err}");
    }
}
