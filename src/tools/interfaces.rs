use serde_json::Value;

use crate::client::RouterosClient;
use crate::params::GetInterfaceParams;

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
        let p = GetInterfaceParams { name: "ether1".into() };
        let result = get_interface(&client, &p).await.unwrap();
        assert_eq!(result["name"], "ether1");
    }
}
