//! RouterOS tool logic, one module per menu area, plus helpers shared by
//! several of them.

pub mod capsman;
pub mod dhcp;
pub mod disk;
pub mod dns;
pub mod firewall;
pub mod interfaces;
pub mod ip;
pub mod lte;
pub mod network;
pub mod smb;
pub mod system;
pub mod wifi;

use anyhow::Context;
use serde_json::{Map, Value, json};

use crate::client::RouterosClient;
use crate::params::RouterOsId;

/// Whether `key` holds a credential. Matched by suffix so inline dotted
/// properties (`security.passphrase`), variants such as `private-passphrase`
/// or `eap-password`, and fields RouterOS adds later are all covered.
fn is_secret_key(key: &str) -> bool {
    key.ends_with("passphrase") || key.ends_with("password")
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
