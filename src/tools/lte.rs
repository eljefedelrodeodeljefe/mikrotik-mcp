use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::GetLteInfoParams;

pub async fn get_lte_info(client: &RouterosClient, p: &GetLteInfoParams) -> anyhow::Result<Value> {
    let iface = p.interface.as_deref().unwrap_or("lte1");
    client
        .post(
            "interface/lte/monitor",
            &json!({"numbers": iface, "once": "yes"}),
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_lte_info_calls_monitor_endpoint() {
        let server = MockServer::start().await;
        // RouterOS exposes modem status via `/interface/lte/monitor` (once),
        // which returns a single-element array — there is no `lte/info` command.
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/monitor"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
                "status": "connected",
                "pin-status": "ok",
                "current-operator": "Telekom.de",
                "data-class": "5G NSA"
            }])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = GetLteInfoParams {
            interface: Some("lte1".into()),
        };
        let result = get_lte_info(&client, &p).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["status"], "connected");
        assert_eq!(result.as_array().unwrap()[0]["data-class"], "5G NSA");
    }
}
