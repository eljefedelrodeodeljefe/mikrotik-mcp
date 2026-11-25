use serde_json::{Value, json};

use super::Props;
use crate::client::RouterosClient;
use crate::params::RouterOsId;
use crate::params::{
    AddDhcpClientParams, AddDhcpStaticLeaseParams, SetDhcpClientParams, SetDhcpLeaseParams,
    SetDhcpNetworkParams,
};

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
    let body = Props::new()
        .with("mac-address", &p.mac_address)
        .with("address", &p.address)
        .text("comment", p.comment.as_deref())
        .into_body();
    client.put("ip/dhcp-server/lease", &body).await
}

pub async fn remove_lease(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete("ip/dhcp-server/lease", id).await
}

/// Converts an existing dynamic lease into a static one
/// (`/ip/dhcp-server/lease/make-static`).
///
/// Preferred over `add_static_lease` when the client already holds a dynamic
/// lease: RouterOS converts the binding in place, so the address is pinned
/// without creating a second lease entry for the same MAC.
pub async fn make_lease_static(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<Value> {
    let body = json!({ "numbers": id });
    client.post("ip/dhcp-server/lease/make-static", &body).await
}

/// Updates properties of an existing lease (`/ip/dhcp-server/lease/set`).
///
/// Only the supplied properties change. Useful for labelling a lease that was
/// created by `make_lease_static`, which carries no comment of its own.
pub async fn set_lease(client: &RouterosClient, p: &SetDhcpLeaseParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .text("address", p.address.as_deref())
        .text("mac-address", p.mac_address.as_deref())
        .text("comment", p.comment.as_deref())
        .text("server", p.server.as_deref())
        .into_changes("set_dhcp_lease")?;
    client.patch("ip/dhcp-server/lease", &p.id, &body).await
}

pub async fn list_networks(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dhcp-server/network").await
}

/// Sets the options a DHCP server hands out for one network
/// (`/ip/dhcp-server/network set`).
pub async fn set_network(
    client: &RouterosClient,
    p: &SetDhcpNetworkParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .text("gateway", p.gateway.as_deref())
        .text("dns-server", p.dns_server.as_deref())
        .text("ntp-server", p.ntp_server.as_deref())
        .text("domain", p.domain.as_deref())
        .text("comment", p.comment.as_deref())
        .into_changes("set_dhcp_network")?;
    client.patch("ip/dhcp-server/network", &p.id, &body).await
}

pub async fn list_clients(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/dhcp-client").await
}

pub async fn add_client(client: &RouterosClient, p: &AddDhcpClientParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("interface", &p.interface)
        .text("add-default-route", p.add_default_route.as_deref())
        .number("default-route-distance", p.default_route_distance)
        .flag("use-peer-dns", p.use_peer_dns)
        .flag("use-peer-ntp", p.use_peer_ntp)
        .flag("disabled", p.disabled)
        .text("comment", p.comment.as_deref())
        .into_body();
    client.put("ip/dhcp-client", &body).await
}

pub async fn set_client(client: &RouterosClient, p: &SetDhcpClientParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .text("interface", p.interface.as_deref())
        .text("add-default-route", p.add_default_route.as_deref())
        .number("default-route-distance", p.default_route_distance)
        .flag("use-peer-dns", p.use_peer_dns)
        .flag("use-peer-ntp", p.use_peer_ntp)
        .flag("disabled", p.disabled)
        .text("comment", p.comment.as_deref())
        .into_changes("set_dhcp_client")?;
    client.patch("ip/dhcp-client", &p.id, &body).await
}

pub async fn remove_client(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete("ip/dhcp-client", id).await
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
        let result =
            make_lease_static(&client, &crate::params::RouterOsId::try_from("*1").unwrap())
                .await
                .unwrap();
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
            id: crate::params::RouterOsId::try_from("*2").unwrap(),
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
            id: crate::params::RouterOsId::try_from("*1").unwrap(),
            address: None,
            mac_address: None,
            comment: None,
            server: None,
        };
        assert!(set_lease(&client, &p).await.is_err());
    }

    #[tokio::test]
    async fn dhcp_client_add_set_list_remove() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dhcp-client"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "interface": "ether1", "status": "bound", "address": "192.0.2.10/24"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/dhcp-client"))
            .and(body_json(json!({
                "interface": "wlan1",
                "add-default-route": "yes",
                "default-route-distance": "2",
                "use-peer-dns": "false",
                "use-peer-ntp": "false",
                "comment": "backup uplink"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*2"})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/dhcp-client/*2"))
            .and(body_json(json!({"default-route-distance": "5"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*2"})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("DELETE"))
            .and(path("/rest/ip/dhcp-client/*2"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        assert_eq!(list_clients(&client).await.unwrap()[0]["status"], "bound");
        let add = AddDhcpClientParams {
            interface: "wlan1".into(),
            add_default_route: Some("yes".into()),
            default_route_distance: Some(2),
            use_peer_dns: Some(false),
            use_peer_ntp: Some(false),
            disabled: None,
            comment: Some("backup uplink".into()),
        };
        add_client(&client, &add).await.unwrap();
        let id = RouterOsId::try_from("*2").unwrap();
        let set = SetDhcpClientParams {
            id: id.clone(),
            interface: None,
            add_default_route: None,
            default_route_distance: Some(5),
            use_peer_dns: None,
            use_peer_ntp: None,
            disabled: None,
            comment: None,
        };
        set_client(&client, &set).await.unwrap();
        remove_client(&client, &id).await.unwrap();
    }

    #[tokio::test]
    async fn set_network_patches_ntp_and_dns_server() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/dhcp-server/network"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "address": "192.0.2.0/24", "gateway": "192.0.2.1"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/dhcp-server/network/*1"))
            .and(body_json(
                json!({"dns-server": "192.0.2.1", "ntp-server": "192.0.2.1"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*1"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        assert_eq!(
            list_networks(&client).await.unwrap()[0]["address"],
            "192.0.2.0/24"
        );
        let p = SetDhcpNetworkParams {
            id: RouterOsId::try_from("*1").unwrap(),
            gateway: None,
            dns_server: Some("192.0.2.1".into()),
            ntp_server: Some("192.0.2.1".into()),
            domain: None,
            comment: None,
        };
        set_network(&client, &p).await.unwrap();
        let empty = SetDhcpNetworkParams {
            id: RouterOsId::try_from("*1").unwrap(),
            gateway: None,
            dns_server: None,
            ntp_server: None,
            domain: None,
            comment: None,
        };
        assert!(set_network(&client, &empty).await.is_err());
    }
}
