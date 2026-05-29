use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{AddInterfaceListMemberParams, GetInterfaceParams, InterfaceNameParams};

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
}
