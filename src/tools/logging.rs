//! Logging rules (`/system/logging`) and actions (`/system/logging/action`).
//! The log itself is `get_logs` in [`super::system`].

use serde_json::Value;

use super::Props;
use crate::client::RouterosClient;
use crate::params::{AddLoggingRuleParams, RouterOsId};

pub async fn list_rules(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/logging").await
}

pub async fn add_rule(client: &RouterosClient, p: &AddLoggingRuleParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("topics", &p.topics)
        .with("action", &p.action)
        .text("prefix", p.prefix.as_deref())
        .flag("disabled", p.disabled)
        .into_body();
    client.put("system/logging", &body).await
}

pub async fn remove_rule(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete("system/logging", id).await
}

pub async fn list_actions(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("system/logging/action").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn logging_rules_and_actions() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/system/logging"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*0", "topics": "info", "action": "memory"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/system/logging/action"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*0", "name": "memory", "target": "memory"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/rest/system/logging"))
            .and(body_json(
                json!({"topics": "wireless,!debug", "action": "memory"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*5"})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("DELETE"))
            .and(path("/rest/system/logging/*5"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        assert_eq!(list_rules(&client).await.unwrap()[0]["action"], "memory");
        assert_eq!(list_actions(&client).await.unwrap()[0]["target"], "memory");
        let p = AddLoggingRuleParams {
            topics: "wireless,!debug".into(),
            action: "memory".into(),
            prefix: None,
            disabled: None,
        };
        add_rule(&client, &p).await.unwrap();
        remove_rule(&client, &RouterOsId::try_from("*5").unwrap())
            .await
            .unwrap();
    }
}
