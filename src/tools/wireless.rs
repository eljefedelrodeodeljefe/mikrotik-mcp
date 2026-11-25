//! Legacy wireless (`/interface/wireless`, the `wireless` package), used by
//! 802.11ac and older radios that do not run `wifi-qcom`. Its newer
//! counterpart, `/interface/wifi`, is in [`super::wifi`]; the registration
//! table is `list_wireless_registrations` in [`super::interfaces`].
//!
//! Security profiles hold their keys in plaintext (`wpa2-pre-shared-key`,
//! `mschapv2-password`, the WEP `static-key-N`), so everything read from or
//! written to them passes through [`super::redact_secrets`]: keys can be set
//! but are never returned.

use anyhow::Context;
use serde_json::Value;

use super::{Props, redact_secrets, resolve_profile_id};
use crate::client::RouterosClient;
use crate::params::{
    AddWirelessConnectListParams, AddWirelessSecurityProfileParams, GetWirelessInterfaceParams,
    RouterOsId, SetWirelessInterfaceParams, SetWirelessSecurityProfileParams,
};

const INTERFACES: &str = "interface/wireless";
const SECURITY_PROFILES: &str = "interface/wireless/security-profiles";
const CONNECT_LIST: &str = "interface/wireless/connect-list";

pub async fn list_interfaces(client: &RouterosClient) -> anyhow::Result<Value> {
    let data = client.get(INTERFACES).await?;
    Ok(redact_secrets(data))
}

/// One interface, looked up by name in the listing (the name never reaches
/// the request path or query).
pub async fn get_interface(
    client: &RouterosClient,
    p: &GetWirelessInterfaceParams,
) -> anyhow::Result<Value> {
    let items: Vec<Value> = client.get(INTERFACES).await?;
    let item = items
        .into_iter()
        .find(|i| i["name"] == p.name.as_str())
        .with_context(|| format!("no wireless interface named {:?}", p.name))?;
    Ok(redact_secrets(item))
}

pub async fn set_interface(
    client: &RouterosClient,
    p: &SetWirelessInterfaceParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .text("mode", p.mode.as_deref())
        .text("ssid", p.ssid.as_deref())
        .text("security-profile", p.security_profile.as_deref())
        .flag("default-authentication", p.default_authentication)
        .text("wps-mode", p.wps_mode.as_deref())
        .text("band", p.band.as_deref())
        .text("frequency", p.frequency.as_deref())
        .text("channel-width", p.channel_width.as_deref())
        .text("country", p.country.as_deref())
        .flag("disabled", p.disabled)
        .text("comment", p.comment.as_deref())
        .into_changes("set_wireless_interface")?;
    let id = resolve_profile_id(client, INTERFACES, p.id.as_ref(), p.name.as_deref()).await?;
    let data = client.patch(INTERFACES, &id, &body).await?;
    Ok(redact_secrets(data))
}

pub async fn list_security_profiles(client: &RouterosClient) -> anyhow::Result<Value> {
    let data = client.get(SECURITY_PROFILES).await?;
    Ok(redact_secrets(data))
}

pub async fn add_security_profile(
    client: &RouterosClient,
    p: &AddWirelessSecurityProfileParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("name", &p.name)
        .text("mode", p.mode.as_deref())
        .text("authentication-types", p.authentication_types.as_deref())
        .text("unicast-ciphers", p.unicast_ciphers.as_deref())
        .text("group-ciphers", p.group_ciphers.as_deref())
        .text("wpa-pre-shared-key", p.wpa_pre_shared_key.as_deref())
        .text("wpa2-pre-shared-key", p.wpa2_pre_shared_key.as_deref())
        .text("comment", p.comment.as_deref())
        .into_body();
    let data = client.put(SECURITY_PROFILES, &body).await?;
    Ok(redact_secrets(data))
}

pub async fn set_security_profile(
    client: &RouterosClient,
    p: &SetWirelessSecurityProfileParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .text("mode", p.mode.as_deref())
        .text("authentication-types", p.authentication_types.as_deref())
        .text("unicast-ciphers", p.unicast_ciphers.as_deref())
        .text("group-ciphers", p.group_ciphers.as_deref())
        .text("wpa-pre-shared-key", p.wpa_pre_shared_key.as_deref())
        .text("wpa2-pre-shared-key", p.wpa2_pre_shared_key.as_deref())
        .text("comment", p.comment.as_deref())
        .into_changes("set_wireless_security_profile")?;
    let id =
        resolve_profile_id(client, SECURITY_PROFILES, p.id.as_ref(), p.name.as_deref()).await?;
    let data = client.patch(SECURITY_PROFILES, &id, &body).await?;
    Ok(redact_secrets(data))
}

pub async fn remove_security_profile(
    client: &RouterosClient,
    id: &RouterOsId,
) -> anyhow::Result<()> {
    client.delete(SECURITY_PROFILES, id).await
}

pub async fn list_connect_list(client: &RouterosClient) -> anyhow::Result<Value> {
    let data = client.get(CONNECT_LIST).await?;
    Ok(redact_secrets(data))
}

pub async fn add_connect_list(
    client: &RouterosClient,
    p: &AddWirelessConnectListParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("interface", &p.interface)
        .text("ssid", p.ssid.as_deref())
        .text("mac-address", p.mac_address.as_deref())
        .text("security-profile", p.security_profile.as_deref())
        .flag("connect", p.connect)
        .text("signal-range", p.signal_range.as_deref())
        .text(
            "place-before",
            p.place_before.as_ref().map(RouterOsId::as_str),
        )
        .text("comment", p.comment.as_deref())
        .into_body();
    client.put(CONNECT_LIST, &body).await
}

pub async fn remove_connect_list(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete(CONNECT_LIST, id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn id(raw: &str) -> RouterOsId {
        RouterOsId::try_from(raw).unwrap()
    }

    fn set_interface_params() -> SetWirelessInterfaceParams {
        SetWirelessInterfaceParams {
            id: None,
            name: None,
            mode: None,
            ssid: None,
            security_profile: None,
            default_authentication: None,
            wps_mode: None,
            band: None,
            frequency: None,
            channel_width: None,
            country: None,
            disabled: None,
            comment: None,
        }
    }

    fn set_profile_params() -> SetWirelessSecurityProfileParams {
        SetWirelessSecurityProfileParams {
            id: None,
            name: None,
            mode: None,
            authentication_types: None,
            unicast_ciphers: None,
            group_ciphers: None,
            wpa_pre_shared_key: None,
            wpa2_pre_shared_key: None,
            comment: None,
        }
    }

    #[tokio::test]
    async fn get_interface_finds_the_named_entry() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wireless"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*5", "name": "wlan1", "mode": "station", "ssid": "ExampleNet"},
                {".id": "*6", "name": "wlan2", "mode": "ap-bridge"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = GetWirelessInterfaceParams {
            name: "wlan2".into(),
        };
        let item = get_interface(&client, &p).await.unwrap();
        assert_eq!(item[".id"], "*6");
        let p = GetWirelessInterfaceParams {
            name: "wlan9".into(),
        };
        let err = get_interface(&client, &p).await.unwrap_err();
        assert!(err.to_string().contains("wlan9"), "{err}");
    }

    #[tokio::test]
    async fn set_interface_by_name_patches_only_given_fields() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wireless"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!([{".id": "*5", "name": "wlan1"}])),
            )
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/interface/wireless/*5"))
            .and(body_json(json!({
                "mode": "station",
                "ssid": "ExampleNet",
                "security-profile": "example-wpa2",
                "default-authentication": "false",
                "wps-mode": "disabled",
                "disabled": "false"
            })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({".id": "*5", "mode": "station"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetWirelessInterfaceParams {
            name: Some("wlan1".into()),
            mode: Some("station".into()),
            ssid: Some("ExampleNet".into()),
            security_profile: Some("example-wpa2".into()),
            default_authentication: Some(false),
            wps_mode: Some("disabled".into()),
            disabled: Some(false),
            ..set_interface_params()
        };
        let data = set_interface(&client, &p).await.unwrap();
        assert_eq!(data["mode"], "station");
    }

    #[tokio::test]
    async fn set_interface_refuses_an_empty_change() {
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        let p = SetWirelessInterfaceParams {
            id: Some(id("*5")),
            ..set_interface_params()
        };
        let err = set_interface(&client, &p).await.unwrap_err();
        assert!(err.to_string().contains("at least one"), "{err}");
    }

    #[tokio::test]
    async fn security_profiles_never_return_their_keys() {
        let server = MockServer::start().await;
        let stored = json!({
            ".id": "*2",
            "name": "example-wpa2",
            "authentication-types": "wpa2-psk",
            "wpa2-pre-shared-key": "not-a-real-secret",
            "wpa-pre-shared-key": "",
            "mschapv2-password": "not-a-real-secret",
            "static-key-0": "not-a-real-secret"
        });
        Mock::given(method("GET"))
            .and(path("/rest/interface/wireless/security-profiles"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([stored.clone()])))
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/rest/interface/wireless/security-profiles"))
            .and(body_json(json!({
                "name": "example-wpa2",
                "mode": "dynamic-keys",
                "authentication-types": "wpa2-psk",
                "wpa2-pre-shared-key": "not-a-real-secret"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(stored.clone()))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/interface/wireless/security-profiles/*2"))
            .and(body_json(
                json!({"wpa2-pre-shared-key": "not-a-real-secret"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(stored))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let listed = list_security_profiles(&client).await.unwrap();
        let added = add_security_profile(
            &client,
            &AddWirelessSecurityProfileParams {
                name: "example-wpa2".into(),
                mode: Some("dynamic-keys".into()),
                authentication_types: Some("wpa2-psk".into()),
                unicast_ciphers: None,
                group_ciphers: None,
                wpa_pre_shared_key: None,
                wpa2_pre_shared_key: Some("not-a-real-secret".into()),
                comment: None,
            },
        )
        .await
        .unwrap();
        let set = set_security_profile(
            &client,
            &SetWirelessSecurityProfileParams {
                name: Some("example-wpa2".into()),
                wpa2_pre_shared_key: Some("not-a-real-secret".into()),
                ..set_profile_params()
            },
        )
        .await
        .unwrap();
        for out in [&listed[0], &added, &set] {
            assert!(!out.to_string().contains("not-a-real-secret"), "{out}");
            assert_eq!(out["wpa2-pre-shared-key-set"], true);
            assert_eq!(out["wpa-pre-shared-key-set"], false);
            assert_eq!(out["authentication-types"], "wpa2-psk");
        }
    }

    #[tokio::test]
    async fn remove_security_profile_deletes_by_id() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/rest/interface/wireless/security-profiles/*2"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let client = RouterosClient::for_test(&server.uri());
        remove_security_profile(&client, &id("*2")).await.unwrap();
    }

    #[tokio::test]
    async fn connect_list_add_list_and_remove() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wireless/connect-list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "interface": "wlan1", "ssid": "ExampleNet", "connect": "true"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/rest/interface/wireless/connect-list"))
            .and(body_json(json!({
                "interface": "wlan1",
                "ssid": "ExampleNet",
                "mac-address": "02:00:00:00:00:01",
                "security-profile": "example-wpa2",
                "connect": "true",
                "place-before": "*1"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*2"})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("DELETE"))
            .and(path("/rest/interface/wireless/connect-list/*2"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let listed = list_connect_list(&client).await.unwrap();
        assert_eq!(listed[0]["ssid"], "ExampleNet");
        let p = AddWirelessConnectListParams {
            interface: "wlan1".into(),
            ssid: Some("ExampleNet".into()),
            mac_address: Some("02:00:00:00:00:01".into()),
            security_profile: Some("example-wpa2".into()),
            connect: Some(true),
            signal_range: None,
            place_before: Some(id("*1")),
            comment: None,
        };
        let added = add_connect_list(&client, &p).await.unwrap();
        assert_eq!(added[".id"], "*2");
        remove_connect_list(&client, &id("*2")).await.unwrap();
    }
}
