use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{
    EsimProfileNumberParams, GetLteInfoParams, LteAtChatParams, LteInterfaceParams,
    ProvisionEsimParams, SetLteApnProfileParams, SetLteTechnologyParams, SetSimSlotParams,
};
use crate::tools::dhcp::yes_no;

pub async fn get_lte_info(client: &RouterosClient, p: &GetLteInfoParams) -> anyhow::Result<Value> {
    let iface = p.interface.as_deref().unwrap_or("lte1");
    client
        .post(
            "interface/lte/monitor",
            &json!({"numbers": iface, "once": "yes"}),
        )
        .await
}

/// Selects the active SIM slot — `/interface lte settings set sim-slot=...`.
/// Switching to `esim` is the prerequisite for downloading an eSIM profile.
pub async fn set_sim_slot(client: &RouterosClient, p: &SetSimSlotParams) -> anyhow::Result<String> {
    client
        .post_text(
            "interface/lte/settings/set",
            &json!({"sim-slot": p.sim_slot}),
        )
        .await
}

/// Lists the eSIM (eUICC) profiles installed on the modem — `/interface/lte/esim print`.
pub async fn list_esim_profiles(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/lte/esim").await
}

/// Lists LTE APN profiles (`/interface/lte/apn`) — incl. `apn`,
/// `add-default-route`, `default-route-distance`, and which is the default.
pub async fn list_apn_profiles(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("interface/lte/apn").await
}

/// Sends a raw AT command to the modem over `/interface/lte/at-chat` and
/// returns the modem's output. Used directly for diagnostics (signal, serving
/// cell, …) and as the transport for higher-level helpers like `set_technology`.
pub async fn at_chat(client: &RouterosClient, iface: &str, command: &str) -> anyhow::Result<Value> {
    client
        .post(
            "interface/lte/at-chat",
            &json!({"number": iface, "input": command}),
        )
        .await
}

/// Sends an arbitrary AT command to the modem (`lte_at_chat` tool).
pub async fn lte_at_chat(client: &RouterosClient, p: &LteAtChatParams) -> anyhow::Result<Value> {
    let iface = p.interface.as_deref().unwrap_or("lte1");
    at_chat(client, iface, &p.command).await
}

/// Sets the modem's radio access technology preference via the Quectel AT
/// command `AT+QNWPREFCFG="mode_pref",<MODE>` over `/interface/lte/at-chat`.
/// Forcing `lte` (4G only) can fix attach failures on 5G modems (e.g. the
/// RG650E) that wedge during NR negotiation.
pub async fn set_technology(
    client: &RouterosClient,
    p: &SetLteTechnologyParams,
) -> anyhow::Result<Value> {
    let iface = p.interface.as_deref().unwrap_or("lte1");
    let mode = match p.mode.trim().to_lowercase().as_str() {
        "lte" | "4g" => "LTE".to_string(),
        "nr5g" | "nr" | "5g" => "NR5G".to_string(),
        "auto" | "both" | "lte:nr5g" => "AUTO".to_string(),
        _ => p.mode.trim().to_uppercase(),
    };
    at_chat(
        client,
        iface,
        &format!("AT+QNWPREFCFG=\"mode_pref\",{mode}"),
    )
    .await
}

/// Updates an LTE APN profile, located by name (or the default profile when no
/// name is given) — `/interface lte apn set`. Set `default_route_distance=2`
/// (or `add_default_route=no`) to make LTE a backup WAN behind a distance-1
/// primary.
pub async fn set_apn_profile(
    client: &RouterosClient,
    p: &SetLteApnProfileParams,
) -> anyhow::Result<String> {
    let profiles = list_apn_profiles(client).await?;
    let target = profiles
        .as_array()
        .into_iter()
        .flatten()
        .find(|prof| match &p.name {
            Some(name) => prof.get("name").and_then(Value::as_str) == Some(name.as_str()),
            None => prof.get("default").and_then(Value::as_str) == Some("true"),
        });
    let label = p.name.clone().unwrap_or_else(|| "default".to_string());
    let id = target
        .and_then(|prof| prof.get(".id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("no LTE APN profile matching '{label}'"))?
        .to_string();

    let mut body = json!({ "numbers": id });
    if let Some(b) = p.add_default_route {
        body["add-default-route"] = json!(yes_no(b));
    }
    if let Some(d) = p.default_route_distance {
        body["default-route-distance"] = json!(d.to_string());
    }
    if let Some(apn) = &p.apn {
        body["apn"] = json!(apn);
    }
    client.post_text("interface/lte/apn/set", &body).await?;
    Ok(format!("LTE APN profile '{label}' updated"))
}

/// Returns the modem's eUICC identifier (EID) — `/interface/lte/esim esim-id`.
pub async fn esim_id(client: &RouterosClient, p: &LteInterfaceParams) -> anyhow::Result<String> {
    let iface = p.interface.as_deref().unwrap_or("lte1");
    client
        .post_text("interface/lte/esim/esim-id", &json!({"interface": iface}))
        .await
}

/// Stages an eSIM profile download from an SM-DP+ server using an activation
/// code — `/interface/lte/esim provision`. Requires working internet
/// connectivity (e.g. the wired WAN) to reach the SM-DP+ server.
///
/// IMPORTANT: this CANNOT complete the install over REST. RouterOS gates the
/// final download behind an interactive end-user consent (the `y/N` prompt in
/// the CLI), which the REST API has no parameter to supply — the flow ends with
/// `user didn't approve`. Worse, a staged-but-unapproved attempt advances the
/// profile's state on the SM-DP+ and can leave it stuck in a non-`Released`
/// state (`Bad profile state`), blocking retries until the server session
/// expires or the operator resets it. To actually install a profile, run the
/// `provision` command from a router terminal and press `y`. This function
/// surfaces those terminal states as errors with guidance instead of silently
/// returning a half-finished status stream.
pub async fn provision_esim(
    client: &RouterosClient,
    p: &ProvisionEsimParams,
) -> anyhow::Result<String> {
    let iface = p.interface.as_deref().unwrap_or("lte1");
    let mut body = json!({
        "interface": iface,
        "sm-dp-plus": p.sm_dp_plus,
        "matching-id": p.matching_id,
        "activate": if p.activate.unwrap_or(true) { "yes" } else { "no" },
    });
    if let Some(cc) = &p.confirmation_code {
        body["confirmation-code"] = json!(cc);
    }
    let raw = client
        .post_text("interface/lte/esim/provision", &body)
        .await?;

    if let Some(status) = final_esim_status(&raw) {
        let lower = status.to_lowercase();
        if lower.contains("didn't approve") || lower.contains("did not approve") {
            anyhow::bail!(
                "profile staged but NOT installed — last status: \"{status}\". RouterOS \
                requires interactive consent (a y/N prompt) that the REST API cannot supply. \
                Complete the install from a router terminal:\n  /interface/lte/esim provision \
                interface={iface} sm-dp-plus=\"{}\" matching-id=\"{}\"\nand press 'y' when \
                prompted. NOTE: this REST attempt may have advanced the profile state on the \
                SM-DP+; if a later attempt reports 'Bad profile state', wait for the server \
                session to expire or ask the operator to release the profile.",
                p.sm_dp_plus,
                p.matching_id
            );
        }
        if lower.contains("failed")
            || lower.contains("bad profile state")
            || lower.contains("didn't authenticate")
            || lower.contains("did not authenticate")
            || lower.contains("error")
        {
            anyhow::bail!("eSIM provisioning rejected by the SM-DP+ server — {status}");
        }
    }
    Ok(raw)
}

/// Extracts the terminal status from a RouterOS eSIM provision response. The
/// command streams an array of `{".section","status",...}` objects; the last
/// `status` is the outcome. Falls back to a single object's `status`. Returns
/// `None` if the body is not JSON (e.g. an empty success body).
fn final_esim_status(raw: &str) -> Option<String> {
    let v: Value = serde_json::from_str(raw).ok()?;
    match &v {
        Value::Array(items) => items.iter().rev().find_map(|it| {
            it.get("status")
                .and_then(|s| s.as_str())
                .map(str::to_string)
        }),
        Value::Object(_) => v.get("status").and_then(|s| s.as_str()).map(str::to_string),
        _ => None,
    }
}

/// Activates an installed eSIM profile — `/interface/lte/esim activate number=...`.
pub async fn activate_esim(
    client: &RouterosClient,
    p: &EsimProfileNumberParams,
) -> anyhow::Result<String> {
    client
        .post_text("interface/lte/esim/activate", &json!({"number": p.number}))
        .await
}

/// Deactivates an installed eSIM profile — `/interface/lte/esim deactivate number=...`.
pub async fn deactivate_esim(
    client: &RouterosClient,
    p: &EsimProfileNumberParams,
) -> anyhow::Result<String> {
    client
        .post_text(
            "interface/lte/esim/deactivate",
            &json!({"number": p.number}),
        )
        .await
}

/// Deletes an installed eSIM profile from the modem — `/interface/lte/esim delete number=...`.
pub async fn delete_esim(
    client: &RouterosClient,
    p: &EsimProfileNumberParams,
) -> anyhow::Result<String> {
    client
        .post_text("interface/lte/esim/delete", &json!({"number": p.number}))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
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

    #[tokio::test]
    async fn set_sim_slot_posts_to_settings_set() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/settings/set"))
            .and(body_json(json!({"sim-slot": "esim"})))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetSimSlotParams {
            sim_slot: "esim".into(),
        };
        set_sim_slot(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn list_esim_profiles_gets_esim_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/lte/esim"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
                ".id": "*1",
                "iccid": "8985235042600607951",
                "state": "enabled"
            }])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_esim_profiles(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["state"], "enabled");
    }

    #[tokio::test]
    async fn provision_esim_posts_activation_code() {
        let server = MockServer::start().await;
        // sm-dp-plus / matching-id / activate are sent in RouterOS' hyphenated form.
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/esim/provision"))
            .and(body_json(json!({
                "interface": "lte1",
                "sm-dp-plus": "wbg.prod.ondemandconnectivity.com",
                "matching-id": "6L152J9F00SXYE3P",
                "activate": "yes"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string("downloaded"))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = ProvisionEsimParams {
            interface: None,
            sm_dp_plus: "wbg.prod.ondemandconnectivity.com".into(),
            matching_id: "6L152J9F00SXYE3P".into(),
            confirmation_code: None,
            activate: None,
        };
        let out = provision_esim(&client, &p).await.unwrap();
        assert_eq!(out, "downloaded");
    }

    #[tokio::test]
    async fn provision_esim_errors_on_user_did_not_approve() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/esim/provision"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".section": "0", "status": "Authenticating eSIM"},
                {".section": "1", "status": "Profile download will start shortly..."},
                {".section": "2", "status": "user didn't approve"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = ProvisionEsimParams {
            interface: None,
            sm_dp_plus: "wbg.prod.ondemandconnectivity.com".into(),
            matching_id: "6L152J9F00SXYE3P".into(),
            confirmation_code: None,
            activate: None,
        };
        let err = provision_esim(&client, &p).await.unwrap_err().to_string();
        assert!(err.contains("NOT installed"), "got: {err}");
        assert!(err.contains("router terminal"), "got: {err}");
    }

    #[tokio::test]
    async fn provision_esim_errors_on_bad_profile_state() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/esim/provision"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".section": "0", "status":
                    "Server didn't authenticate eSIM (Failed, reason 1.2, Bad profile state)"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = ProvisionEsimParams {
            interface: None,
            sm_dp_plus: "wbg.prod.ondemandconnectivity.com".into(),
            matching_id: "6L152J9F00SXYE3P".into(),
            confirmation_code: None,
            activate: None,
        };
        let err = provision_esim(&client, &p).await.unwrap_err().to_string();
        assert!(err.contains("SM-DP+"), "got: {err}");
        assert!(err.contains("Bad profile state"), "got: {err}");
    }

    #[tokio::test]
    async fn activate_esim_posts_number_as_numbers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/esim/activate"))
            .and(body_json(json!({"number": "0"})))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = EsimProfileNumberParams { number: "0".into() };
        activate_esim(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn lte_at_chat_sends_raw_command() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/at-chat"))
            .and(body_json(json!({"number": "lte1", "input": "AT+CSQ"})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"output": "+CSQ: 17,99"})),
            )
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = LteAtChatParams {
            interface: None,
            command: "AT+CSQ".into(),
        };
        let result = lte_at_chat(&client, &p).await.unwrap();
        assert_eq!(result["output"], "+CSQ: 17,99");
    }

    #[tokio::test]
    async fn set_technology_sends_qnwprefcfg_at_command() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/at-chat"))
            .and(body_json(json!({
                "number": "lte1",
                "input": "AT+QNWPREFCFG=\"mode_pref\",LTE"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"output": "OK"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetLteTechnologyParams {
            interface: None,
            mode: "lte".into(),
        };
        let result = set_technology(&client, &p).await.unwrap();
        assert_eq!(result["output"], "OK");
    }

    #[tokio::test]
    async fn list_apn_profiles_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/lte/apn"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "default", "default": "true", "apn": "internet"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_apn_profiles(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["name"], "default");
    }

    #[tokio::test]
    async fn set_apn_profile_targets_default_and_sets_distance() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/interface/lte/apn"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "default", "default": "true"},
                {".id": "*2", "name": "other", "default": "false"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/interface/lte/apn/set"))
            .and(body_json(json!({
                "numbers": "*1",
                "add-default-route": "yes",
                "default-route-distance": "2"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetLteApnProfileParams {
            name: None,
            add_default_route: Some(true),
            default_route_distance: Some(2),
            apn: None,
        };
        let msg = set_apn_profile(&client, &p).await.unwrap();
        assert!(msg.contains("default"), "got: {msg}");
    }
}
