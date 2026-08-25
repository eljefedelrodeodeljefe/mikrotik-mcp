use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{AddDhcpStaticLeaseParams, SetDhcpLeaseParams};

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
    client.put("ip/dhcp-server/lease", &body).await
}

pub async fn remove_lease(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/dhcp-server/lease", id).await
}

/// Converts an existing dynamic lease into a static one
/// (`/ip/dhcp-server/lease/make-static`).
///
/// Preferred over `add_static_lease` when the client already holds a dynamic
/// lease: RouterOS converts the binding in place, so the address is pinned
/// without creating a second lease entry for the same MAC.
pub async fn make_lease_static(client: &RouterosClient, id: &str) -> anyhow::Result<Value> {
    let body = json!({ "numbers": id });
    client.post("ip/dhcp-server/lease/make-static", &body).await
}

/// Updates properties of an existing lease (`/ip/dhcp-server/lease/set`).
///
/// Only the supplied properties change. Useful for labelling a lease that was
/// created by `make_lease_static`, which carries no comment of its own.
pub async fn set_lease(client: &RouterosClient, p: &SetDhcpLeaseParams) -> anyhow::Result<Value> {
    let mut body = json!({});
    if let Some(v) = &p.address {
        body["address"] = json!(v);
    }
    if let Some(v) = &p.mac_address {
        body["mac-address"] = json!(v);
    }
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    if let Some(v) = &p.server {
        body["server"] = json!(v);
    }
    anyhow::ensure!(
        body.as_object().is_some_and(|o| !o.is_empty()),
        "set_dhcp_lease needs at least one property to change"
    );
    client.patch("ip/dhcp-server/lease", &p.id, &body).await
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
    async fn make_lease_static_posts_to_the_command_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/ip/dhcp-server/lease/make-static"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*2"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = make_lease_static(&client, "*1").await.unwrap();
        assert_eq!(result[".id"], "*2");
    }

    #[tokio::test]
    async fn set_lease_patches_only_the_supplied_properties() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/dhcp-server/lease/*2"))
            .and(body_json(json!({"comment": "reserved"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*2", "comment": "reserved"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetDhcpLeaseParams {
            id: "*2".into(),
            address: None,
            mac_address: None,
            comment: Some("reserved".into()),
            server: None,
        };
        let result = set_lease(&client, &p).await.unwrap();
        assert_eq!(result["comment"], "reserved");
    }

    #[tokio::test]
    async fn set_lease_refuses_an_empty_change() {
        let client = RouterosClient::for_test("http://127.0.0.1:1");
        let p = SetDhcpLeaseParams {
            id: "*1".into(),
            address: None,
            mac_address: None,
            comment: None,
            server: None,
        };
        assert!(set_lease(&client, &p).await.is_err());
    }
}
