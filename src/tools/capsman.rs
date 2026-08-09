//! CAPsMAN (Controlled Access Point system Manager) — read-only tools for the
//! legacy `/caps-man` menu.
//!
//! Note the RouterOS split: this module covers legacy CAPsMAN (`/caps-man`,
//! provided by the `wireless` package). The newer `wifi` package exposes an
//! unrelated manager at `/interface/wifi/capsman`. Both can be enabled on the
//! same router at once, and a CAP only ever joins the one matching its
//! installed package.
//!
//! `/caps-man/security` is deliberately not exposed — it returns the WPA
//! passphrase in plaintext.

use serde_json::Value;

use crate::client::RouterosClient;

pub async fn get_capsman_manager(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/manager").await
}

pub async fn list_capsman_remote_caps(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/remote-cap").await
}

pub async fn list_capsman_radios(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/radio").await
}

pub async fn list_capsman_interfaces(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/interface").await
}

pub async fn list_capsman_registrations(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/registration-table").await
}

pub async fn list_capsman_configurations(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/configuration").await
}

pub async fn list_capsman_provisioning(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/provisioning").await
}

pub async fn list_capsman_channels(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/channel").await
}

pub async fn list_capsman_datapaths(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("caps-man/datapath").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_capsman_manager_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/manager"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "enabled": "true",
                "certificate": "none",
                "require-peer-certificate": "false"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = get_capsman_manager(&client).await.unwrap();
        assert_eq!(result["enabled"], "true");
    }

    #[tokio::test]
    async fn list_capsman_remote_caps_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/remote-cap"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    ".id": "*2",
                    "base-mac": "AA:BB:CC:00:00:01",
                    "board": "RBcAPGi-5acD2nD",
                    "identity": "cap-1",
                    "radios": "2",
                    "state": "Run",
                    "version": "7.20.7"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_remote_caps(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["state"], "Run");
    }

    #[tokio::test]
    async fn list_capsman_radios_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/radio"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "radio-mac": "AA:BB:CC:00:00:02",
                    "interface": "cap6",
                    "remote-cap-identity": "cap-1",
                    "provisioned": "true"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_radios(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["provisioned"], "true");
    }

    #[tokio::test]
    async fn list_capsman_interfaces_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/interface"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    ".id": "*14",
                    "name": "cap1",
                    "configuration": "cfg-2g",
                    "current-state": "running-ap",
                    "master": "true"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_interfaces(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["current-state"], "running-ap");
    }

    #[tokio::test]
    async fn list_capsman_registrations_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/registration-table"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "interface": "cap1",
                    "mac-address": "AA:BB:CC:00:00:03",
                    "ssid": "example-wifi",
                    "rx-signal": "-43"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_registrations(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["rx-signal"], "-43");
    }

    #[tokio::test]
    async fn list_capsman_configurations_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "cfg-2g", "channel": "ch-2g"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_configurations(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["name"], "cfg-2g");
    }

    #[tokio::test]
    async fn list_capsman_provisioning_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/provisioning"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    ".id": "*1",
                    "action": "create-dynamic-enabled",
                    "hw-supported-modes": "gn",
                    "master-configuration": "cfg-2g"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_provisioning(&client).await.unwrap();
        assert_eq!(
            result.as_array().unwrap()[0]["action"],
            "create-dynamic-enabled"
        );
    }

    #[tokio::test]
    async fn list_capsman_channels_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/channel"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    ".id": "*2",
                    "name": "ch-5g",
                    "band": "5ghz-n/ac",
                    "frequency": "5180",
                    "extension-channel": "Ceee"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_channels(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["frequency"], "5180");
    }

    #[tokio::test]
    async fn list_capsman_datapaths_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/datapath"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*2", "name": "dp-guest", "bridge": "bridge-guest", "local-forwarding": "false"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_capsman_datapaths(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["local-forwarding"], "false");
    }
}
