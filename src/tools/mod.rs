//! RouterOS tool logic, one module per menu area, plus helpers shared by
//! several of them.

pub mod capsman;
pub mod dhcp;
pub mod diagnostics;
pub mod disk;
pub mod dns;
pub mod firewall;
pub mod interfaces;
pub mod ip;
pub mod logging;
pub mod lte;
pub mod network;
pub mod services;
pub mod smb;
pub mod system;
pub mod wifi;
pub mod wireless;

use anyhow::Context;
use serde_json::{Map, Value, json};

use crate::client::RouterosClient;
use crate::params::RouterOsId;

/// Whether `key` holds a credential. Matched by suffix so inline dotted
/// properties (`security.passphrase`), variants such as `private-passphrase`
/// or `eap-password`, and fields RouterOS adds later are all covered. The
/// legacy wireless security profiles add `wpa2-pre-shared-key`,
/// `management-protection-key`, `static-sta-private-key` and the WEP keys
/// `static-key-0` to `static-key-3`.
fn is_secret_key(key: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "passphrase",
        "password",
        "pre-shared-key",
        "private-key",
        "protection-key",
        "secret",
    ];
    SUFFIXES.iter().any(|s| key.ends_with(s)) || key.starts_with("static-key-")
}

/// Drops every credential field, recursively, and puts a `<key>-set` boolean
/// in its place, so a caller can still tell whether one is configured.
pub(crate) fn redact_secrets(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(redact_secrets).collect()),
        Value::Object(obj) => {
            let mut out = Map::new();
            for (k, v) in obj {
                if is_secret_key(&k) {
                    let set = v.as_str().map_or(!v.is_null(), |s| !s.is_empty());
                    out.insert(format!("{k}-set"), json!(set));
                } else {
                    out.insert(k, redact_secrets(v));
                }
            }
            Value::Object(out)
        }
        other => other,
    }
}

/// Resolves a profile given by `.id` or by `name` to its `.id`. Names never
/// reach the request path: they are looked up in the menu's listing. An empty
/// name counts as not given.
pub(crate) async fn resolve_profile_id(
    client: &RouterosClient,
    menu: &str,
    id: Option<&RouterOsId>,
    name: Option<&str>,
) -> anyhow::Result<RouterOsId> {
    let name = name.filter(|n| !n.trim().is_empty());
    match (id, name) {
        (Some(id), None) => Ok(id.clone()),
        (None, Some(name)) => {
            let items: Value = client.get(menu).await?;
            let item = items
                .as_array()
                .into_iter()
                .flatten()
                .find(|i| i["name"] == name)
                .with_context(|| format!("no profile named {name:?} in /{menu}"))?;
            let raw = item[".id"]
                .as_str()
                .with_context(|| format!("profile {name:?} in /{menu} has no .id"))?;
            RouterOsId::try_from(raw).map_err(anyhow::Error::msg)
        }
        _ => anyhow::bail!("pass exactly one of id or name"),
    }
}

/// The properties of an `add` (`PUT`) or `set` (`PATCH`, `POST …/set`) body,
/// built from a tool's optional parameters. Only the parameters given are
/// sent, so a `set` changes nothing else. Values go out as strings: numbers
/// in decimal, booleans as `"true"`/`"false"`.
#[derive(Debug, Default)]
pub(crate) struct Props(Map<String, Value>);

impl Props {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Sets `key` to `value` unconditionally (a required property).
    pub(crate) fn with(mut self, key: &str, value: &str) -> Self {
        self.0.insert(key.into(), json!(value));
        self
    }

    pub(crate) fn text(mut self, key: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            self.0.insert(key.into(), json!(v));
        }
        self
    }

    pub(crate) fn flag(mut self, key: &str, value: Option<bool>) -> Self {
        if let Some(v) = value {
            self.0.insert(key.into(), json!(v.to_string()));
        }
        self
    }

    pub(crate) fn number(mut self, key: &str, value: Option<u32>) -> Self {
        if let Some(v) = value {
            self.0.insert(key.into(), json!(v.to_string()));
        }
        self
    }

    /// The body as given, which may be empty.
    pub(crate) fn into_body(self) -> Value {
        Value::Object(self.0)
    }

    /// The body of a `set`, which must change at least one property.
    pub(crate) fn into_changes(self, tool: &str) -> anyhow::Result<Value> {
        anyhow::ensure!(
            !self.0.is_empty(),
            "{tool} needs at least one property to change"
        );
        Ok(Value::Object(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn redact_secrets_replaces_credentials_with_set_flags() {
        let out = redact_secrets(json!([
            {
                ".id": "*1",
                "name": "cfg-5g",
                "ssid": "ExampleNet",
                "security.passphrase": "not-a-real-secret",
                "security.authentication-types": "wpa2-psk"
            },
            {
                ".id": "*2",
                "security": {"passphrase": ""},
                "eap-password": "x",
                "private-passphrase": "not-a-real-secret"
            }
        ]));
        let text = out.to_string();
        assert!(!text.contains("not-a-real-secret"), "{text}");
        assert_eq!(out[0]["security.passphrase-set"], true);
        assert!(out[0].get("security.passphrase").is_none());
        assert_eq!(out[0]["security.authentication-types"], "wpa2-psk");
        assert_eq!(out[0]["ssid"], "ExampleNet");
        assert_eq!(out[1]["security"]["passphrase-set"], false);
        assert_eq!(out[1]["eap-password-set"], true);
        assert_eq!(out[1]["private-passphrase-set"], true);
    }

    #[test]
    fn redact_secrets_covers_legacy_wireless_security_profiles() {
        let out = redact_secrets(json!({
            "name": "example-wpa2",
            "authentication-types": "wpa2-psk",
            "wpa-pre-shared-key": "not-a-real-secret",
            "wpa2-pre-shared-key": "not-a-real-secret",
            "mschapv2-password": "not-a-real-secret",
            "mschapv2-username": "user",
            "management-protection-key": "not-a-real-secret",
            "static-key-0": "not-a-real-secret",
            "static-algo-0": "none",
            "static-sta-private-key": "",
            "radius-mac-accounting": "no"
        }));
        assert!(!out.to_string().contains("not-a-real-secret"), "{out}");
        assert_eq!(out["wpa2-pre-shared-key-set"], true);
        assert_eq!(out["mschapv2-password-set"], true);
        assert_eq!(out["static-key-0-set"], true);
        assert_eq!(out["static-sta-private-key-set"], false);
        assert_eq!(out["management-protection-key-set"], true);
        assert_eq!(out["static-algo-0"], "none");
        assert_eq!(out["mschapv2-username"], "user");
        assert_eq!(out["authentication-types"], "wpa2-psk");
    }

    #[test]
    fn props_sends_only_given_values_as_routeros_strings() {
        let body = Props::new()
            .with("interface", "ether1")
            .text("comment", None)
            .text("ssid", Some("ExampleNet"))
            .flag("disabled", Some(false))
            .flag("use-peer-dns", None)
            .number("default-route-distance", Some(2))
            .number("port", None)
            .into_body();
        assert_eq!(
            body,
            json!({
                "interface": "ether1",
                "ssid": "ExampleNet",
                "disabled": "false",
                "default-route-distance": "2"
            })
        );
    }

    #[test]
    fn props_refuses_an_empty_set() {
        let err = Props::new()
            .text("comment", None)
            .into_changes("set_x")
            .unwrap_err();
        assert!(
            err.to_string().contains("set_x needs at least one"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn resolve_profile_id_errors_on_entry_without_id() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/wifi/configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"name": "cfg-5g", "ssid": "ExampleNet"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let err = resolve_profile_id(
            &client,
            "interface/wifi/configuration",
            None,
            Some("cfg-5g"),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("has no .id"), "{err}");
    }
}
