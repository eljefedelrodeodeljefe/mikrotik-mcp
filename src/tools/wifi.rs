//! WiFi (`/interface/wifi`, the `wifi-qcom` / `wifi-qcom-ac` packages) and its
//! CAPsMAN at `/interface/wifi/capsman`.
//!
//! This is the counterpart of [`super::capsman`], which covers legacy CAPsMAN
//! (`/caps-man`). On a manager running both, `wcap*` interfaces live here and
//! `cap*` interfaces there. CAPsMAN-managed `/interface/wifi` entries are
//! dynamic: `/interface/wifi disable` refuses them, so re-provisioning the
//! radio is the way to restart one.
//!
//! `/interface/wifi/security` is deliberately not exposed — it returns the
//! passphrase in plaintext.

use serde_json::{Map, Value, json};

use crate::client::RouterosClient;
use crate::params::{ProvisionWifiRadioParams, SetCapsmanChannelParams, WifiMonitorParams};

pub async fn list_wifi_interfaces(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wifi").await
}

pub async fn monitor_wifi_interface(
    client: &RouterosClient,
    p: &WifiMonitorParams,
) -> anyhow::Result<Value> {
    client
        .post(
            "interface/wifi/monitor",
            &json!({"numbers": p.interface, "once": ""}),
        )
        .await
}

pub async fn list_wifi_registrations(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wifi/registration-table").await
}

pub async fn list_wifi_radios(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wifi/radio").await
}

pub async fn list_wifi_channels(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wifi/channel").await
}

pub async fn list_wifi_capsman_remote_caps(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wifi/capsman/remote-cap").await
}

pub async fn provision_wifi_radio(
    client: &RouterosClient,
    p: &ProvisionWifiRadioParams,
) -> anyhow::Result<Value> {
    client
        .post("interface/wifi/radio/provision", &json!({"numbers": p.id}))
        .await
}

/// Sets properties on a legacy `/caps-man/channel` profile. Lives here rather
/// than in `capsman` because that module is read-only by design.
pub async fn set_capsman_channel(
    client: &RouterosClient,
    p: &SetCapsmanChannelParams,
) -> anyhow::Result<Value> {
    let mut body = Map::new();
    if let Some(f) = &p.frequency {
        body.insert("frequency".into(), json!(f));
    }
    if let Some(e) = &p.extension_channel {
        body.insert("extension-channel".into(), json!(e));
    }
    anyhow::ensure!(
        !body.is_empty(),
        "nothing to set — pass frequency and/or extension_channel"
    );
    client.patch("caps-man/channel", &p.id, &body).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn get_mock(server: &MockServer, p: &str) {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{".id": "*1"}])))
            .expect(1)
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn read_tools_call_correct_paths() {
        let server = MockServer::start().await;
        get_mock(&server, "/rest/interface/wifi").await;
        get_mock(&server, "/rest/interface/wifi/registration-table").await;
        get_mock(&server, "/rest/interface/wifi/radio").await;
        get_mock(&server, "/rest/interface/wifi/channel").await;
        get_mock(&server, "/rest/interface/wifi/capsman/remote-cap").await;

        let client = RouterosClient::for_test(&server.uri());
        list_wifi_interfaces(&client).await.unwrap();
        list_wifi_registrations(&client).await.unwrap();
        list_wifi_radios(&client).await.unwrap();
        list_wifi_channels(&client).await.unwrap();
        let caps = list_wifi_capsman_remote_caps(&client).await.unwrap();
        assert_eq!(caps[0][".id"], "*1");
    }

    #[tokio::test]
    async fn monitor_runs_once() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/wifi/monitor"))
            .and(body_json(json!({"numbers": "wcap", "once": ""})))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!([{"state": "cac", "registered-peers": "0"}])),
            )
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = WifiMonitorParams {
            interface: "wcap".into(),
        };
        let result = monitor_wifi_interface(&client, &p).await.unwrap();
        assert_eq!(result[0]["state"], "cac");
    }

    #[tokio::test]
    async fn provision_posts_radio_id() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/wifi/radio/provision"))
            .and(body_json(json!({"numbers": "*5"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = ProvisionWifiRadioParams { id: "*5".into() };
        provision_wifi_radio(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn set_capsman_channel_patches_only_given_fields() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/caps-man/channel/*1"))
            .and(body_json(json!({"frequency": "2412"})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({".id": "*1", "frequency": "2412"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetCapsmanChannelParams {
            id: "*1".into(),
            frequency: Some("2412".into()),
            extension_channel: None,
        };
        let result = set_capsman_channel(&client, &p).await.unwrap();
        assert_eq!(result["frequency"], "2412");
    }

    #[tokio::test]
    async fn set_capsman_channel_rejects_empty_update() {
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        let p = SetCapsmanChannelParams {
            id: "*1".into(),
            frequency: None,
            extension_channel: None,
        };
        assert!(set_capsman_channel(&client, &p).await.is_err());
    }
}
