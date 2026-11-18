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
//! passphrase in plaintext. Configuration profiles and interfaces can carry
//! security properties inline (`security.passphrase`, `security.eap-password`),
//! so everything read from or written to either passes through
//! [`super::redact_secrets`].

use serde_json::{Map, Value, json};

use super::{redact_secrets, resolve_profile_id};
use crate::client::RouterosClient;
use crate::params::{
    ProvisionWifiRadioParams, SetCapsmanChannelParams, SetCapsmanConfigurationParams,
    SetWifiConfigurationParams, WifiMonitorParams,
};

fn insert_str(body: &mut Map<String, Value>, key: &str, value: Option<&String>) {
    if let Some(v) = value {
        body.insert(key.into(), json!(v));
    }
}

fn insert_bool(body: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(v) = value {
        body.insert(key.into(), json!(v.to_string()));
    }
}

pub async fn list_wifi_configurations(client: &RouterosClient) -> anyhow::Result<Value> {
    let data = client.get("interface/wifi/configuration").await?;
    Ok(redact_secrets(data))
}

/// Sets properties on a legacy `/caps-man/configuration` profile. Lives here
/// rather than in `capsman` because that module is read-only by design.
pub async fn set_capsman_configuration(
    client: &RouterosClient,
    p: &SetCapsmanConfigurationParams,
) -> anyhow::Result<Value> {
    let mut body = Map::new();
    insert_str(&mut body, "ssid", p.ssid.as_ref());
    insert_str(&mut body, "channel", p.channel.as_ref());
    insert_str(&mut body, "security", p.security.as_ref());
    insert_str(&mut body, "datapath", p.datapath.as_ref());
    insert_str(&mut body, "country", p.country.as_ref());
    insert_str(&mut body, "installation", p.installation.as_ref());
    insert_bool(&mut body, "hide-ssid", p.hide_ssid);
    insert_str(&mut body, "comment", p.comment.as_ref());
    anyhow::ensure!(
        !body.is_empty(),
        "set_capsman_configuration needs at least one property to change"
    );
    let menu = "caps-man/configuration";
    let id = resolve_profile_id(client, menu, p.id.as_ref(), p.name.as_deref()).await?;
    let data = client.patch(menu, &id, &body).await?;
    Ok(redact_secrets(data))
}

/// Sets properties on a `/interface/wifi/configuration` profile.
pub async fn set_wifi_configuration(
    client: &RouterosClient,
    p: &SetWifiConfigurationParams,
) -> anyhow::Result<Value> {
    let mut body = Map::new();
    insert_str(&mut body, "ssid", p.ssid.as_ref());
    insert_str(&mut body, "channel", p.channel.as_ref());
    insert_str(&mut body, "security", p.security.as_ref());
    insert_str(&mut body, "datapath", p.datapath.as_ref());
    insert_str(&mut body, "country", p.country.as_ref());
    insert_str(&mut body, "installation", p.installation.as_ref());
    insert_str(&mut body, "mode", p.mode.as_ref());
    insert_bool(&mut body, "hide-ssid", p.hide_ssid);
    insert_bool(&mut body, "disabled", p.disabled);
    insert_str(&mut body, "comment", p.comment.as_ref());
    anyhow::ensure!(
        !body.is_empty(),
        "set_wifi_configuration needs at least one property to change"
    );
    let menu = "interface/wifi/configuration";
    let id = resolve_profile_id(client, menu, p.id.as_ref(), p.name.as_deref()).await?;
    let data = client.patch(menu, &id, &body).await?;
    Ok(redact_secrets(data))
}

pub async fn list_wifi_interfaces(client: &RouterosClient) -> anyhow::Result<Value> {
    let data = client.get("interface/wifi").await?;
    Ok(redact_secrets(data))
}

pub async fn monitor_wifi_interface(
    client: &RouterosClient,
    p: &WifiMonitorParams,
) -> anyhow::Result<Value> {
    client
        .post_read(
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
    use crate::params::RouterOsId;
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
        get_mock(&server, "/rest/interface/wifi/configuration").await;

        let client = RouterosClient::for_test(&server.uri());
        list_wifi_interfaces(&client).await.unwrap();
        list_wifi_registrations(&client).await.unwrap();
        list_wifi_radios(&client).await.unwrap();
        list_wifi_channels(&client).await.unwrap();
        list_wifi_configurations(&client).await.unwrap();
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
        let p = ProvisionWifiRadioParams {
            id: crate::params::RouterOsId::try_from("*5").unwrap(),
        };
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
            id: crate::params::RouterOsId::try_from("*1").unwrap(),
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
            id: crate::params::RouterOsId::try_from("*1").unwrap(),
            frequency: None,
            extension_channel: None,
        };
        assert!(set_capsman_channel(&client, &p).await.is_err());
    }

    fn capsman_cfg(id: Option<&str>, name: Option<&str>) -> SetCapsmanConfigurationParams {
        SetCapsmanConfigurationParams {
            id: id.map(|i| RouterOsId::try_from(i).unwrap()),
            name: name.map(Into::into),
            ssid: Some("ExampleNet".into()),
            channel: None,
            security: None,
            datapath: None,
            country: None,
            installation: None,
            hide_ssid: None,
            comment: None,
        }
    }

    fn wifi_cfg(id: Option<&str>, name: Option<&str>) -> SetWifiConfigurationParams {
        SetWifiConfigurationParams {
            id: id.map(|i| RouterOsId::try_from(i).unwrap()),
            name: name.map(Into::into),
            ssid: None,
            channel: None,
            security: None,
            datapath: None,
            country: None,
            installation: None,
            mode: None,
            hide_ssid: None,
            disabled: None,
            comment: None,
        }
    }

    #[tokio::test]
    async fn list_wifi_configurations_strips_passphrase() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wifi/configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    ".id": "*1",
                    "name": "cfg-5g",
                    "ssid": "ExampleNet",
                    "security.passphrase": "not-a-real-secret"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_wifi_configurations(&client).await.unwrap();
        assert_eq!(result[0]["ssid"], "ExampleNet");
        assert_eq!(result[0]["security.passphrase-set"], true);
        assert!(!result.to_string().contains("not-a-real-secret"));
    }

    #[tokio::test]
    async fn set_capsman_configuration_by_id_patches_only_given_fields() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/caps-man/configuration/*3"))
            .and(body_json(
                json!({"ssid": "ExampleNet", "hide-ssid": "false"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*3", "ssid": "ExampleNet", "security.passphrase": "not-a-real-secret"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let mut p = capsman_cfg(Some("*3"), None);
        p.hide_ssid = Some(false);
        let result = set_capsman_configuration(&client, &p).await.unwrap();
        assert_eq!(result["ssid"], "ExampleNet");
        assert!(!result.to_string().contains("not-a-real-secret"));
    }

    #[tokio::test]
    async fn set_capsman_configuration_resolves_name_to_id() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "cfg-2g"},
                {".id": "*2", "name": "cfg-5g"}
            ])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/caps-man/configuration/*2"))
            .and(body_json(json!({"ssid": "ExampleNet"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*2"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = capsman_cfg(None, Some("cfg-5g"));
        set_capsman_configuration(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn set_capsman_configuration_unknown_name_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/caps-man/configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "cfg-2g"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = capsman_cfg(None, Some("cfg-missing"));
        let err = set_capsman_configuration(&client, &p).await.unwrap_err();
        assert!(err.to_string().contains("cfg-missing"), "{err}");
    }

    #[tokio::test]
    async fn set_configuration_needs_exactly_one_of_id_or_name() {
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        assert!(
            set_capsman_configuration(&client, &capsman_cfg(None, None))
                .await
                .is_err()
        );
        assert!(
            set_capsman_configuration(&client, &capsman_cfg(Some("*1"), Some("cfg-5g")))
                .await
                .is_err()
        );
        let mut p = wifi_cfg(None, None);
        p.ssid = Some("ExampleNet".into());
        assert!(set_wifi_configuration(&client, &p).await.is_err());
        p.id = Some(RouterOsId::try_from("*1").unwrap());
        p.name = Some("cfg-5g".into());
        assert!(set_wifi_configuration(&client, &p).await.is_err());
    }

    #[tokio::test]
    async fn empty_name_counts_as_absent() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/caps-man/configuration/*1"))
            .and(body_json(json!({"ssid": "ExampleNet"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*1"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        set_capsman_configuration(&client, &capsman_cfg(Some("*1"), Some("")))
            .await
            .unwrap();
        // An empty name alone is no identifier at all, and must not match a
        // nameless entry in the listing.
        assert!(
            set_capsman_configuration(&client, &capsman_cfg(None, Some("")))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn set_wifi_configuration_by_id_skips_lookup() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(0)
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/interface/wifi/configuration/*B"))
            .and(body_json(
                json!({"ssid": "ExampleNet", "installation": "indoor"}),
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({".id": "*B", "ssid": "ExampleNet"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let mut p = wifi_cfg(Some("*B"), None);
        p.ssid = Some("ExampleNet".into());
        p.installation = Some("indoor".into());
        let result = set_wifi_configuration(&client, &p).await.unwrap();
        assert_eq!(result["ssid"], "ExampleNet");
    }

    #[tokio::test]
    async fn set_configuration_rejects_empty_update() {
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        let mut p = capsman_cfg(Some("*1"), None);
        p.ssid = None;
        assert!(set_capsman_configuration(&client, &p).await.is_err());
        assert!(
            set_wifi_configuration(&client, &wifi_cfg(Some("*1"), None))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn set_wifi_configuration_patches_wifi_field_names() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wifi/configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*A", "name": "cfg-5g"}
            ])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/interface/wifi/configuration/*A"))
            .and(body_json(json!({
                "ssid": "ExampleNet",
                "mode": "ap",
                "hide-ssid": "true",
                "disabled": "false"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*A", "ssid": "ExampleNet", "security.passphrase": "not-a-real-secret"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let mut p = wifi_cfg(None, Some("cfg-5g"));
        p.ssid = Some("ExampleNet".into());
        p.mode = Some("ap".into());
        p.hide_ssid = Some(true);
        p.disabled = Some(false);
        let result = set_wifi_configuration(&client, &p).await.unwrap();
        assert_eq!(result["security.passphrase-set"], true);
        assert!(!result.to_string().contains("not-a-real-secret"));
    }

    #[tokio::test]
    async fn list_wifi_interfaces_strips_inline_secrets() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wifi"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    ".id": "*1",
                    "name": "wifi1",
                    "configuration.ssid": "ExampleNet",
                    "security.passphrase": "not-a-real-secret",
                    "security.eap-password": "also-not-a-secret"
                }
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_wifi_interfaces(&client).await.unwrap();
        let text = result.to_string();
        assert!(!text.contains("not-a-real-secret"), "{text}");
        assert!(!text.contains("also-not-a-secret"), "{text}");
        assert_eq!(result[0]["security.passphrase-set"], true);
        assert_eq!(result[0]["security.eap-password-set"], true);
        assert_eq!(result[0]["configuration.ssid"], "ExampleNet");
    }
}
