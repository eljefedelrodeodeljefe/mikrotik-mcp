//! Management services: the listeners in `/ip/service`, MikroTik cloud
//! (`/ip/cloud`) and the MAC server (`/tool/mac-server`).

use anyhow::Context;
use serde_json::{Value, json};

use super::Props;
use crate::client::RouterosClient;
use crate::params::{RouterOsId, SetIpCloudParams, SetIpServiceParams, SetMacServerParams};

const SERVICES: &str = "ip/service";

pub async fn list_ip_services(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get(SERVICES).await
}

/// Resolves a service given by `.id` or by name. RouterOS 7.2x also lists
/// dynamic entries (ports other features listen on, established
/// connections), some sharing a static service's name; only the static
/// entry can be set.
async fn resolve_service(
    client: &RouterosClient,
    id: Option<&RouterOsId>,
    name: Option<&str>,
) -> anyhow::Result<RouterOsId> {
    let name = name.filter(|n| !n.trim().is_empty());
    match (id, name) {
        (Some(id), None) => Ok(id.clone()),
        (None, Some(name)) => {
            let items: Vec<Value> = client.get(SERVICES).await?;
            let named: Vec<&Value> = items.iter().filter(|i| i["name"] == name).collect();
            anyhow::ensure!(
                !named.is_empty(),
                "no service named {name:?} in /ip/service"
            );
            let item = named
                .into_iter()
                .find(|i| i["dynamic"] != "true" && i["dynamic"] != true)
                .with_context(|| {
                    format!("{name:?} is a dynamic /ip/service entry, which cannot be set")
                })?;
            let raw = item[".id"]
                .as_str()
                .with_context(|| format!("service {name:?} has no .id"))?;
            RouterOsId::try_from(raw).map_err(anyhow::Error::msg)
        }
        _ => anyhow::bail!("pass exactly one of id or name"),
    }
}

pub async fn set_ip_service(
    client: &RouterosClient,
    p: &SetIpServiceParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .flag("disabled", p.disabled)
        .number("port", p.port)
        // `address` is the old name of available-from; RouterOS versions
        // with available-from still take it and write it there.
        .text("address", p.available_from.as_deref())
        .text("certificate", p.certificate.as_deref())
        .text("tls-version", p.tls_version.as_deref())
        .number("max-sessions", p.max_sessions)
        .text("vrf", p.vrf.as_deref())
        .into_changes("set_ip_service")?;
    let id = resolve_service(client, p.id.as_ref(), p.name.as_deref()).await?;
    client.patch(SERVICES, &id, &body).await
}

pub async fn get_ip_cloud(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/cloud").await
}

pub async fn set_ip_cloud(client: &RouterosClient, p: &SetIpCloudParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .text("ddns-enabled", p.ddns_enabled.as_deref())
        .text("ddns-update-interval", p.ddns_update_interval.as_deref())
        .flag("update-time", p.update_time)
        .into_changes("set_ip_cloud")?;
    client.post("ip/cloud/set", &body).await
}

/// MAC Telnet, MAC WinBox and MAC ping settings together.
pub async fn get_mac_server(client: &RouterosClient) -> anyhow::Result<Value> {
    let mac_server: Value = client.get("tool/mac-server").await?;
    let mac_winbox: Value = client.get("tool/mac-server/mac-winbox").await?;
    let ping: Value = client.get("tool/mac-server/ping").await?;
    Ok(json!({"mac-server": mac_server, "mac-winbox": mac_winbox, "ping": ping}))
}

/// Sets each of the three menus that has a change; at least one must.
pub async fn set_mac_server(
    client: &RouterosClient,
    p: &SetMacServerParams,
) -> anyhow::Result<Value> {
    let changes = [
        (
            "tool/mac-server/set",
            Props::new().text(
                "allowed-interface-list",
                p.allowed_interface_list.as_deref(),
            ),
        ),
        (
            "tool/mac-server/mac-winbox/set",
            Props::new().text(
                "allowed-interface-list",
                p.mac_winbox_allowed_interface_list.as_deref(),
            ),
        ),
        (
            "tool/mac-server/ping/set",
            Props::new().flag("enabled", p.ping_enabled),
        ),
    ];
    let mut done = Vec::new();
    for (path, props) in changes {
        if let Ok(body) = props.into_changes("set_mac_server") {
            let _: Value = client
                .post(path, &body)
                .await
                .with_context(|| format!("{path} failed after {done:?} were set"))?;
            done.push(path);
        }
    }
    anyhow::ensure!(
        !done.is_empty(),
        "set_mac_server needs at least one property to change"
    );
    get_mac_server(client).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn service_params() -> SetIpServiceParams {
        SetIpServiceParams {
            id: None,
            name: None,
            disabled: None,
            port: None,
            available_from: None,
            certificate: None,
            tls_version: None,
            max_sessions: None,
            vrf: None,
        }
    }

    async fn services_listing(server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/rest/ip/service"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "www-ssl", "dynamic": "true", "local": "192.0.2.1"},
                {".id": "*9", "name": "www-ssl", "dynamic": "false", "port": "443"},
                {".id": "*2C", "name": "resolver", "dynamic": "true", "port": "53"}
            ])))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn set_ip_service_by_name_skips_dynamic_entries() {
        let server = MockServer::start().await;
        services_listing(&server).await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/service/*9"))
            .and(body_json(json!({
                "address": "192.0.2.0/24",
                "certificate": "www",
                "tls-version": "only-1.2"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*9"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetIpServiceParams {
            name: Some("www-ssl".into()),
            available_from: Some("192.0.2.0/24".into()),
            certificate: Some("www".into()),
            tls_version: Some("only-1.2".into()),
            ..service_params()
        };
        set_ip_service(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn set_ip_service_refuses_a_dynamic_only_name() {
        let server = MockServer::start().await;
        services_listing(&server).await;
        let client = RouterosClient::for_test(&server.uri());
        let p = SetIpServiceParams {
            name: Some("resolver".into()),
            disabled: Some(true),
            ..service_params()
        };
        let err = set_ip_service(&client, &p).await.unwrap_err();
        assert!(err.to_string().contains("dynamic"), "{err}");
        let p = SetIpServiceParams {
            name: Some("gopher".into()),
            port: Some(70),
            ..service_params()
        };
        let err = set_ip_service(&client, &p).await.unwrap_err();
        assert!(err.to_string().contains("no service"), "{err}");
    }

    #[tokio::test]
    async fn set_ip_service_by_id_sends_the_port_as_a_string() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/service/*3"))
            .and(body_json(json!({"disabled": "false", "port": "2222"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*3"})))
            .expect(1)
            .mount(&server)
            .await;
        let client = RouterosClient::for_test(&server.uri());
        let p = SetIpServiceParams {
            id: Some(RouterOsId::try_from("*3").unwrap()),
            disabled: Some(false),
            port: Some(2222),
            ..service_params()
        };
        set_ip_service(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn ip_cloud_get_and_set() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/cloud"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"ddns-enabled": "auto", "update-time": "true"})),
            )
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/ip/cloud/set"))
            .and(body_json(
                json!({"ddns-enabled": "auto", "update-time": "false"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;
        let client = RouterosClient::for_test(&server.uri());
        assert_eq!(get_ip_cloud(&client).await.unwrap()["ddns-enabled"], "auto");
        let p = SetIpCloudParams {
            ddns_enabled: Some("auto".into()),
            ddns_update_interval: None,
            update_time: Some(false),
        };
        set_ip_cloud(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn mac_server_sets_only_the_menus_with_changes() {
        let server = MockServer::start().await;
        for (p, body) in [
            (
                "/rest/tool/mac-server",
                json!({"allowed-interface-list": "LAN"}),
            ),
            (
                "/rest/tool/mac-server/mac-winbox",
                json!({"allowed-interface-list": "LAN"}),
            ),
            ("/rest/tool/mac-server/ping", json!({"enabled": "false"})),
        ] {
            Mock::given(method("GET"))
                .and(path(p))
                .respond_with(ResponseTemplate::new(200).set_body_json(body))
                .mount(&server)
                .await;
        }
        Mock::given(method("POST"))
            .and(path("/rest/tool/mac-server/mac-winbox/set"))
            .and(body_json(json!({"allowed-interface-list": "LAN"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/tool/mac-server/ping/set"))
            .and(body_json(json!({"enabled": "false"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/tool/mac-server/set"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(0)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetMacServerParams {
            allowed_interface_list: None,
            mac_winbox_allowed_interface_list: Some("LAN".into()),
            ping_enabled: Some(false),
        };
        let out = set_mac_server(&client, &p).await.unwrap();
        assert_eq!(out["ping"]["enabled"], "false");
        let empty = SetMacServerParams {
            allowed_interface_list: None,
            mac_winbox_allowed_interface_list: None,
            ping_enabled: None,
        };
        assert!(set_mac_server(&client, &empty).await.is_err());
    }
}
