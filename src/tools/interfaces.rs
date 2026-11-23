use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::RouterOsId;
use crate::params::{
    AddInterfaceListMemberParams, GetInterfaceParams, InterfaceNameParams, SetBridgePortParams,
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

pub async fn remove_interface_list_member(
    client: &RouterosClient,
    id: &RouterOsId,
) -> anyhow::Result<()> {
    client.delete("interface/list/member", id).await
}

pub async fn list_bridge_ports(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/bridge/port").await
}

pub async fn list_bridge_hosts(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/bridge/host").await
}

pub async fn set_bridge_port(
    client: &RouterosClient,
    p: &SetBridgePortParams,
) -> anyhow::Result<Value> {
    let mut body = serde_json::Map::new();
    if let Some(b) = &p.bridge {
        body.insert("bridge".into(), json!(b));
    }
    if let Some(c) = &p.comment {
        body.insert("comment".into(), json!(c));
    }
    anyhow::ensure!(
        !body.is_empty(),
        "nothing to set — pass bridge and/or comment"
    );
    client.patch("interface/bridge/port", &p.id, &body).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
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
    async fn list_bridge_hosts_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/bridge/host"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"mac-address": "00:2B:67:0B:72:50", "on-interface": "ether6", "bridge": "bridge"}
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_bridge_hosts(&client).await.unwrap();
        assert_eq!(result[0]["on-interface"], "ether6");
    }

    #[tokio::test]
    async fn list_bridge_ports_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/bridge/port"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*4", "interface": "ether6", "bridge": "bridge"}
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_bridge_ports(&client).await.unwrap();
        assert_eq!(result[0]["interface"], "ether6");
    }

    #[tokio::test]
    async fn set_bridge_port_patches_only_given_fields() {
        use wiremock::matchers::body_json;
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/interface/bridge/port/*4"))
            .and(body_json(json!({"bridge": "bridge-ci"})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    json!({".id": "*4", "interface": "ether6", "bridge": "bridge-ci"}),
                ),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetBridgePortParams {
            id: crate::params::RouterOsId::try_from("*4").unwrap(),
            bridge: Some("bridge-ci".into()),
            comment: None,
        };
        let result = set_bridge_port(&client, &p).await.unwrap();
        assert_eq!(result["bridge"], "bridge-ci");
    }

    #[tokio::test]
    async fn set_bridge_port_rejects_empty_update() {
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        let p = SetBridgePortParams {
            id: crate::params::RouterOsId::try_from("*4").unwrap(),
            bridge: None,
            comment: None,
        };
        assert!(set_bridge_port(&client, &p).await.is_err());
    }
}
