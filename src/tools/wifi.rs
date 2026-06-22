use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{
    AddWifiApParams, AddWifiSecurityParams, AddWifiStationParams, ScanWifiParams,
    SetWifiInterfaceParams, WifiNameParams,
};

pub async fn list_wifi_interfaces(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wifi").await
}

pub async fn list_wifi_security(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/wifi/security").await
}

/// Creates a named wifi security profile — `/interface/wifi/security add`.
/// Reference the returned profile name in `add_wifi_station` → `security`.
pub async fn add_wifi_security(
    client: &RouterosClient,
    p: &AddWifiSecurityParams,
) -> anyhow::Result<Value> {
    let auth_types = p.authentication_types.as_deref().unwrap_or("wpa2-psk");
    client
        .put(
            "interface/wifi/security",
            &json!({
                "name": p.name,
                "authentication-types": auth_types,
                "passphrase": p.passphrase,
            }),
        )
        .await
}

/// Adds a virtual station interface on top of an existing radio (master-interface)
/// — `/interface/wifi add mode=station`. The master continues to serve as an AP
/// while this VIF connects upstream to the named SSID.
pub async fn add_wifi_station(
    client: &RouterosClient,
    p: &AddWifiStationParams,
) -> anyhow::Result<Value> {
    // RouterOS 7 wifi uses flat dot-notation keys for sub-objects:
    // "configuration.mode" and "configuration.ssid" (not top-level "mode"/"ssid").
    let mut body = json!({
        "name": p.name,
        "master-interface": p.master_interface,
        "configuration.mode": "station",
        "configuration.ssid": p.ssid,
    });
    if let Some(sec) = &p.security {
        body["security"] = json!(sec);
    }
    client.put("interface/wifi", &body).await
}

/// Adds a virtual AP interface on top of an existing radio (master-interface)
/// — `/interface/wifi add mode=ap`. The master keeps its own SSID; this VIF
/// broadcasts an additional SSID (second BSSID) on the same radio. Reference the
/// master's security profile to make it a true alias. Add it to the LAN bridge
/// with `add_bridge_port` for the SSID to pass traffic.
pub async fn add_wifi_ap(client: &RouterosClient, p: &AddWifiApParams) -> anyhow::Result<Value> {
    let mut body = json!({
        "name": p.name,
        "master-interface": p.master_interface,
        "configuration.mode": "ap",
        "configuration.ssid": p.ssid,
    });
    if let Some(sec) = &p.security {
        body["security"] = json!(sec);
    }
    if let Some(comment) = &p.comment {
        body["comment"] = json!(comment);
    }
    client.put("interface/wifi", &body).await
}

/// Runs a passive wifi scan on the given interface — `/interface/wifi/scan`.
/// Returns a JSON array of visible APs with SSID, BSSID, channel, signal, and security.
/// The interface must be enabled; station-mode interfaces may briefly interrupt
/// scanning while trying to associate. Duration defaults to 5 s.
pub async fn scan_wifi(client: &RouterosClient, p: &ScanWifiParams) -> anyhow::Result<Value> {
    let interfaces = list_wifi_interfaces(client).await?;
    let id = interfaces
        .as_array()
        .into_iter()
        .flatten()
        .find(|i| i.get("name").and_then(Value::as_str) == Some(p.interface.as_str()))
        .and_then(|i| i.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("no wifi interface named '{}'", p.interface))?
        .to_string();

    let duration = p.duration.unwrap_or(5);
    let body = json!({ "numbers": id, "duration": duration });
    client.post("interface/wifi/scan", &body).await
}

/// Modifies an existing wifi interface — `/interface/wifi/set`. Resolves the
/// interface by name. Use to convert an AP to station mode (set `mode=station`,
/// `ssid=<upstream>`, `security=<profile>`), or to update an existing station.
pub async fn set_wifi_interface(
    client: &RouterosClient,
    p: &SetWifiInterfaceParams,
) -> anyhow::Result<String> {
    let interfaces = list_wifi_interfaces(client).await?;
    let id = interfaces
        .as_array()
        .into_iter()
        .flatten()
        .find(|i| i.get("name").and_then(Value::as_str) == Some(p.name.as_str()))
        .and_then(|i| i.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("no wifi interface named '{}'", p.name))?
        .to_string();

    let mut body = json!({ "numbers": id });
    if let Some(mode) = &p.mode {
        body["configuration.mode"] = json!(mode);
    }
    if let Some(ssid) = &p.ssid {
        body["configuration.ssid"] = json!(ssid);
    }
    if let Some(sec) = &p.security {
        body["security"] = json!(sec);
    }
    if let Some(master) = &p.master_interface {
        body["master-interface"] = json!(master);
    }
    if let Some(pw) = &p.passphrase {
        body["security.passphrase"] = json!(pw);
        // Station mode doesn't use Fast Transition (AP-to-AP roaming feature);
        // leave it set and it breaks the WPA handshake.
        body["security.ft"] = json!("no");
        body["security.ft-over-ds"] = json!("no");
        body["security.authentication-types"] = json!("wpa2-psk");
    }
    client.post_text("interface/wifi/set", &body).await?;
    Ok(format!("wifi interface '{}' updated", p.name))
}

/// Removes a wifi interface by name — `/interface/wifi remove`.
pub async fn remove_wifi_interface(
    client: &RouterosClient,
    p: &WifiNameParams,
) -> anyhow::Result<()> {
    let interfaces = list_wifi_interfaces(client).await?;
    let id = interfaces
        .as_array()
        .into_iter()
        .flatten()
        .find(|i| i.get("name").and_then(Value::as_str) == Some(p.name.as_str()))
        .and_then(|i| i.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("no wifi interface named '{}'", p.name))?
        .to_string();
    client.delete("interface/wifi", &id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_wifi_interfaces_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wifi"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*3", "name": "wifi1", "mode": "ap", "master-interface": ""}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_wifi_interfaces(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["name"], "wifi1");
    }

    #[tokio::test]
    async fn add_wifi_security_puts_profile() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/interface/wifi/security"))
            .and(body_json(json!({
                "name": "upstream-sec",
                "authentication-types": "wpa2-psk",
                "passphrase": "testpassword"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*1", "name": "upstream-sec"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddWifiSecurityParams {
            name: "upstream-sec".into(),
            passphrase: "testpassword".into(),
            authentication_types: None,
        };
        let result = add_wifi_security(&client, &p).await.unwrap();
        assert_eq!(result["name"], "upstream-sec");
    }

    #[tokio::test]
    async fn add_wifi_station_puts_station_interface() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/interface/wifi"))
            .and(body_json(json!({
                "name": "wifi3",
                "master-interface": "wifi1",
                "configuration.mode": "station",
                "configuration.ssid": "MyHotspot",
                "security": "upstream-sec"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*B", "name": "wifi3", "configuration.mode": "station"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddWifiStationParams {
            name: "wifi3".into(),
            master_interface: "wifi1".into(),
            ssid: "MyHotspot".into(),
            security: Some("upstream-sec".into()),
        };
        let result = add_wifi_station(&client, &p).await.unwrap();
        assert_eq!(result["name"], "wifi3");
        assert_eq!(result["configuration.mode"], "station");
    }

    #[tokio::test]
    async fn add_wifi_ap_puts_ap_interface() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/interface/wifi"))
            .and(body_json(json!({
                "name": "spine-guest",
                "master-interface": "wifi2",
                "configuration.mode": "ap",
                "configuration.ssid": "Spine",
                "security": "Quick Set",
                "comment": "alias of Spine 2.4GHz"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*B", "name": "spine-guest", "configuration.mode": "ap"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddWifiApParams {
            name: "spine-guest".into(),
            master_interface: "wifi2".into(),
            ssid: "Spine".into(),
            security: Some("Quick Set".into()),
            comment: Some("alias of Spine 2.4GHz".into()),
        };
        let result = add_wifi_ap(&client, &p).await.unwrap();
        assert_eq!(result["name"], "spine-guest");
        assert_eq!(result["configuration.mode"], "ap");
    }
}
