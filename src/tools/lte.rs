use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::GetLteInfoParams;

pub async fn get_lte_info(client: &RouterosClient, p: &GetLteInfoParams) -> anyhow::Result<Value> {
    let iface = p.interface.as_deref().unwrap_or("lte1");
    client
        .post("interface/lte/info", &json!({"number": iface}))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_lte_info_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/info"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "connected",
                "pin-status": "verified",
                "operator": "Telekom.de"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = GetLteInfoParams {
            interface: Some("lte1".into()),
        };
        let result = get_lte_info(&client, &p).await.unwrap();
        assert_eq!(result["status"], "connected");
        assert_eq!(result["pin-status"], "verified");
    }
}
