use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{
    AddBridgePortParams, AddInterfaceListMemberParams, GetInterfaceParams, InterfaceNameParams,
};

pub async fn list_interfaces(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface").await
}

pub async fn list_wireless_registrations(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wireless/registration-table").await
}

pub async fn get_interface(
    client: &RouterosClient,
    p: &GetInterfaceParams,
) -> anyhow::Result<Value> {
    client.get(&format!("interface?name={}", p.name)).await
}

pub async fn enable_interface(
    client: &RouterosClient,
    p: &InterfaceNameParams,
) -> anyhow::Result<()> {
    client
        .post_void("interface/enable", &json!({"numbers": p.interface}))
        .await
}

pub async fn disable_interface(
    client: &RouterosClient,
    p: &InterfaceNameParams,
) -> anyhow::Result<()> {
    client
        .post_void("interface/disable", &json!({"numbers": p.interface}))
        .await
}

pub async fn list_interface_list_members(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/list/member").await
}

pub async fn add_interface_list_member(
    client: &RouterosClient,
    p: &AddInterfaceListMemberParams,
) -> anyhow::Result<Value> {
    client
        .put(
            "interface/list/member",
            &json!({"list": p.list, "interface": p.interface}),
        )
        .await
}

pub async fn remove_interface_list_member(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("interface/list/member", id).await
}

/// Lists bridge ports — `/interface/bridge/port`. Shows which interfaces are
/// bridged, their bridge, pvid, and status (e.g. `in-bridge`).
pub async fn list_bridge_ports(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/bridge/port").await
}

/// Adds an interface to a bridge — `/interface/bridge/port add`. Use after
/// creating a virtual AP (`add_wifi_ap`) so the new SSID reaches the LAN.
pub async fn add_bridge_port(
    client: &RouterosClient,
    p: &AddBridgePortParams,
) -> anyhow::Result<Value> {
    let mut body = json!({
        "bridge": p.bridge,
        "interface": p.interface,
    });
    if let Some(pvid) = p.pvid {
        body["pvid"] = json!(pvid);
    }
    if let Some(comment) = &p.comment {
        body["comment"] = json!(comment);
    }
    client.put("interface/bridge/port", &body).await
}

/// Removes an interface from the bridge — `/interface/bridge/port remove`.
/// Resolves by interface name. Required before converting a bridged AP interface
/// to station mode (a WAN interface must not be in the LAN bridge).
pub async fn remove_bridge_port(client: &RouterosClient, interface: &str) -> anyhow::Result<()> {
    let ports: Value = client.get("interface/bridge/port").await?;
    let id = ports
        .as_array()
        .into_iter()
        .flatten()
        .find(|p| p.get("interface").and_then(Value::as_str) == Some(interface))
        .and_then(|p| p.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("'{}' is not a bridge port", interface))?
        .to_string();
    client.delete("interface/bridge/port", &id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_interfaces_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"name": "ether1", "type": "ether"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_interfaces(&client).await.unwrap();
        assert!(result.as_array().unwrap()[0]["name"] == "ether1");
    }

    #[tokio::test]
    async fn list_wireless_registrations_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wireless/registration-table"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_wireless_registrations(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_interface_includes_name_in_query() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name": "ether1"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = GetInterfaceParams {
            name: "ether1".into(),
        };
        let result = get_interface(&client, &p).await.unwrap();
        assert_eq!(result["name"], "ether1");
    }

    #[tokio::test]
    async fn enable_interface_posts_to_enable_command() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/enable"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = InterfaceNameParams {
            interface: "pppoe-out1".into(),
        };
        enable_interface(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn disable_interface_posts_to_disable_command() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/disable"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = InterfaceNameParams {
            interface: "pppoe-out1".into(),
        };
        disable_interface(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn list_interface_list_members_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/list/member"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"list": "WAN", "interface": "lte1"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_interface_list_members(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["interface"], "lte1");
    }

    #[tokio::test]
    async fn add_interface_list_member_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/interface/list/member"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*8", "list": "WAN", "interface": "lte1"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddInterfaceListMemberParams {
            list: "WAN".into(),
            interface: "lte1".into(),
        };
        let result = add_interface_list_member(&client, &p).await.unwrap();
        assert_eq!(result["list"], "WAN");
    }

    #[tokio::test]
    async fn list_bridge_ports_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/bridge/port"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "bridge": "bridge", "interface": "wifi2", "pvid": "1"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_bridge_ports(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["interface"], "wifi2");
    }

    #[tokio::test]
    async fn add_bridge_port_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/interface/bridge/port"))
            .and(body_json(json!({
                "bridge": "bridge",
                "interface": "spine-guest",
                "pvid": 1
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*7", "bridge": "bridge", "interface": "spine-guest"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddBridgePortParams {
            bridge: "bridge".into(),
            interface: "spine-guest".into(),
            pvid: Some(1),
            comment: None,
        };
        let result = add_bridge_port(&client, &p).await.unwrap();
        assert_eq!(result["interface"], "spine-guest");
    }
}
