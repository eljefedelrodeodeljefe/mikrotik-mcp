use serde_json::{Value, json};

use super::Props;
use crate::client::RouterosClient;
use crate::params::RouterOsId;
use crate::params::{
    AddFirewallAddressListParams, AddFirewallFilterParams, AddFirewallMangleParams,
    AddFirewallNatParams, MoveFirewallRuleParams, SetFilterRule, SetFirewallFilterParams,
    SetFirewallMangleParams, SetFirewallNatParams, SetMangleRule, SetNatRule,
};

const FILTER: &str = "ip/firewall/filter";
const NAT: &str = "ip/firewall/nat";
const MANGLE: &str = "ip/firewall/mangle";

fn id_str(id: Option<&RouterOsId>) -> Option<&str> {
    id.map(RouterOsId::as_str)
}

pub async fn list_filter(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get(FILTER).await
}

pub async fn add_filter(
    client: &RouterosClient,
    p: &AddFirewallFilterParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("chain", &p.chain)
        .with("action", &p.action)
        .text("src-address", p.src_address.as_deref())
        .text("dst-address", p.dst_address.as_deref())
        .text("protocol", p.protocol.as_deref())
        .text("src-port", p.src_port.as_deref())
        .text("dst-port", p.dst_port.as_deref())
        .text("in-interface", p.in_interface.as_deref())
        .text("out-interface", p.out_interface.as_deref())
        .text("in-interface-list", p.in_interface_list.as_deref())
        .text("out-interface-list", p.out_interface_list.as_deref())
        .text("src-address-list", p.src_address_list.as_deref())
        .text("dst-address-list", p.dst_address_list.as_deref())
        .text("connection-state", p.connection_state.as_deref())
        .flag("log", p.log)
        .text("log-prefix", p.log_prefix.as_deref())
        .text("comment", p.comment.as_deref())
        .flag("disabled", p.disabled)
        .text("place-before", id_str(p.place_before.as_ref()))
        .into_body();
    client.put(FILTER, &body).await
}

fn filter_changes(r: &SetFilterRule) -> anyhow::Result<Value> {
    Props::new()
        .text("chain", r.chain.as_deref())
        .text("action", r.action.as_deref())
        .text("src-address", r.src_address.as_deref())
        .text("dst-address", r.dst_address.as_deref())
        .text("protocol", r.protocol.as_deref())
        .text("src-port", r.src_port.as_deref())
        .text("dst-port", r.dst_port.as_deref())
        .text("in-interface", r.in_interface.as_deref())
        .text("out-interface", r.out_interface.as_deref())
        .text("in-interface-list", r.in_interface_list.as_deref())
        .text("out-interface-list", r.out_interface_list.as_deref())
        .text("src-address-list", r.src_address_list.as_deref())
        .text("dst-address-list", r.dst_address_list.as_deref())
        .text("connection-state", r.connection_state.as_deref())
        .flag("log", r.log)
        .text("log-prefix", r.log_prefix.as_deref())
        .text("comment", r.comment.as_deref())
        .flag("disabled", r.disabled)
        .into_changes("set_firewall_filter")
}

pub async fn set_filter(
    client: &RouterosClient,
    p: &SetFirewallFilterParams,
) -> anyhow::Result<Value> {
    let body = filter_changes(&p.rule)?;
    client.patch(FILTER, &p.id, &body).await
}

pub async fn remove_filter(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete(FILTER, id).await
}

pub async fn list_nat(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get(NAT).await
}

pub async fn add_nat(client: &RouterosClient, p: &AddFirewallNatParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("chain", &p.chain)
        .with("action", &p.action)
        .text("src-address", p.src_address.as_deref())
        .text("dst-address", p.dst_address.as_deref())
        .text("protocol", p.protocol.as_deref())
        .text("dst-port", p.dst_port.as_deref())
        .text("to-addresses", p.to_addresses.as_deref())
        .text("to-ports", p.to_ports.as_deref())
        .text("out-interface", p.out_interface.as_deref())
        .text("in-interface", p.in_interface.as_deref())
        .text("in-interface-list", p.in_interface_list.as_deref())
        .text("out-interface-list", p.out_interface_list.as_deref())
        .text("comment", p.comment.as_deref())
        .flag("disabled", p.disabled)
        .text("place-before", id_str(p.place_before.as_ref()))
        .into_body();
    client.put(NAT, &body).await
}

fn nat_changes(r: &SetNatRule) -> anyhow::Result<Value> {
    Props::new()
        .text("chain", r.chain.as_deref())
        .text("action", r.action.as_deref())
        .text("src-address", r.src_address.as_deref())
        .text("dst-address", r.dst_address.as_deref())
        .text("protocol", r.protocol.as_deref())
        .text("dst-port", r.dst_port.as_deref())
        .text("to-addresses", r.to_addresses.as_deref())
        .text("to-ports", r.to_ports.as_deref())
        .text("in-interface", r.in_interface.as_deref())
        .text("out-interface", r.out_interface.as_deref())
        .text("in-interface-list", r.in_interface_list.as_deref())
        .text("out-interface-list", r.out_interface_list.as_deref())
        .text("comment", r.comment.as_deref())
        .flag("disabled", r.disabled)
        .into_changes("set_firewall_nat")
}

pub async fn set_nat(client: &RouterosClient, p: &SetFirewallNatParams) -> anyhow::Result<Value> {
    let body = nat_changes(&p.rule)?;
    client.patch(NAT, &p.id, &body).await
}

pub async fn remove_nat(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete(NAT, id).await
}

pub async fn list_mangle(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get(MANGLE).await
}

pub async fn add_mangle(
    client: &RouterosClient,
    p: &AddFirewallMangleParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("chain", &p.chain)
        .with("action", &p.action)
        .text("new-mss", p.new_mss.as_deref())
        .flag("passthrough", p.passthrough)
        .text("protocol", p.protocol.as_deref())
        .text("tcp-flags", p.tcp_flags.as_deref())
        .text("src-address", p.src_address.as_deref())
        .text("dst-address", p.dst_address.as_deref())
        .text("in-interface", p.in_interface.as_deref())
        .text("out-interface", p.out_interface.as_deref())
        .text("in-interface-list", p.in_interface_list.as_deref())
        .text("out-interface-list", p.out_interface_list.as_deref())
        .text("connection-mark", p.connection_mark.as_deref())
        .text("new-connection-mark", p.new_connection_mark.as_deref())
        .text("new-routing-mark", p.new_routing_mark.as_deref())
        .text("new-packet-mark", p.new_packet_mark.as_deref())
        .text("comment", p.comment.as_deref())
        .flag("disabled", p.disabled)
        .text("place-before", id_str(p.place_before.as_ref()))
        .into_body();
    client.put(MANGLE, &body).await
}

fn mangle_changes(r: &SetMangleRule) -> anyhow::Result<Value> {
    Props::new()
        .text("chain", r.chain.as_deref())
        .text("action", r.action.as_deref())
        .text("new-mss", r.new_mss.as_deref())
        .flag("passthrough", r.passthrough)
        .text("protocol", r.protocol.as_deref())
        .text("tcp-flags", r.tcp_flags.as_deref())
        .text("src-address", r.src_address.as_deref())
        .text("dst-address", r.dst_address.as_deref())
        .text("in-interface", r.in_interface.as_deref())
        .text("out-interface", r.out_interface.as_deref())
        .text("in-interface-list", r.in_interface_list.as_deref())
        .text("out-interface-list", r.out_interface_list.as_deref())
        .text("connection-mark", r.connection_mark.as_deref())
        .text("new-connection-mark", r.new_connection_mark.as_deref())
        .text("new-routing-mark", r.new_routing_mark.as_deref())
        .text("new-packet-mark", r.new_packet_mark.as_deref())
        .text("comment", r.comment.as_deref())
        .flag("disabled", r.disabled)
        .into_changes("set_firewall_mangle")
}

pub async fn set_mangle(
    client: &RouterosClient,
    p: &SetFirewallMangleParams,
) -> anyhow::Result<Value> {
    let body = mangle_changes(&p.rule)?;
    client.patch(MANGLE, &p.id, &body).await
}

pub async fn remove_mangle(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete(MANGLE, id).await
}

/// Which rule table a `move_firewall_*` tool reorders.
#[derive(Clone, Copy, Debug)]
pub enum RuleTable {
    Filter,
    Nat,
    Mangle,
}

impl RuleTable {
    fn menu(self) -> &'static str {
        match self {
            Self::Filter => FILTER,
            Self::Nat => NAT,
            Self::Mangle => MANGLE,
        }
    }
}

/// `move` a rule before `destination`, or to the end without one. Rules are
/// matched top to bottom, so this changes what the table does.
pub async fn move_rule(
    client: &RouterosClient,
    table: RuleTable,
    p: &MoveFirewallRuleParams,
) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("numbers", p.id.as_str())
        .text("destination", id_str(p.destination.as_ref()))
        .into_body();
    client.post(&format!("{}/move", table.menu()), &body).await
}

pub async fn list_address_list(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/firewall/address-list").await
}

pub async fn add_address_list(
    client: &RouterosClient,
    p: &AddFirewallAddressListParams,
) -> anyhow::Result<Value> {
    let mut body = json!({"list": p.list, "address": p.address});
    if let Some(v) = &p.timeout {
        body["timeout"] = json!(v);
    }
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    client.put("ip/firewall/address-list", &body).await
}

pub async fn remove_address_list(client: &RouterosClient, id: &RouterOsId) -> anyhow::Result<()> {
    client.delete("ip/firewall/address-list", id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_filter_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/filter"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_filter(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_nat_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/nat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_nat(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_nat_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/nat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*5", "chain": "srcnat", "action": "masquerade"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallNatParams {
            chain: "srcnat".into(),
            action: "masquerade".into(),
            src_address: None,
            dst_address: None,
            protocol: None,
            dst_port: None,
            to_addresses: None,
            to_ports: None,
            out_interface: Some("bridge".into()),
            comment: None,
            ..Default::default()
        };
        let result = add_nat(&client, &p).await.unwrap();
        assert_eq!(result["action"], "masquerade");
    }

    #[tokio::test]
    async fn add_filter_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/filter"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*6", "chain": "input", "action": "accept"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallFilterParams {
            chain: "input".into(),
            action: "accept".into(),
            src_address: None,
            dst_address: None,
            protocol: None,
            dst_port: None,
            in_interface: None,
            comment: None,
            disabled: None,
            ..Default::default()
        };
        let result = add_filter(&client, &p).await.unwrap();
        assert_eq!(result["action"], "accept");
    }

    #[tokio::test]
    async fn list_address_list_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/address-list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"list": "local-subnets", "address": "192.168.88.0/24"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_address_list(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["list"], "local-subnets");
    }

    #[tokio::test]
    async fn add_address_list_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/address-list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*7", "list": "local-subnets", "address": "10.0.0.0/8"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallAddressListParams {
            list: "local-subnets".into(),
            address: "10.0.0.0/8".into(),
            timeout: None,
            comment: None,
        };
        let result = add_address_list(&client, &p).await.unwrap();
        assert_eq!(result["list"], "local-subnets");
    }

    #[tokio::test]
    async fn list_mangle_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/firewall/mangle"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_mangle(&client).await.unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_mangle_puts_mss_clamp() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/mangle"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*4", "chain": "forward", "action": "change-mss",
                "new-mss": "clamp-to-pmtu", "out-interface-list": "WAN"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallMangleParams {
            chain: "forward".into(),
            action: "change-mss".into(),
            new_mss: Some("clamp-to-pmtu".into()),
            passthrough: Some(true),
            protocol: Some("tcp".into()),
            tcp_flags: Some("syn".into()),
            src_address: None,
            dst_address: None,
            in_interface: None,
            out_interface: None,
            in_interface_list: None,
            out_interface_list: Some("WAN".into()),
            connection_mark: None,
            new_connection_mark: None,
            new_routing_mark: None,
            new_packet_mark: None,
            comment: None,
            ..Default::default()
        };
        let result = add_mangle(&client, &p).await.unwrap();
        assert_eq!(result["new-mss"], "clamp-to-pmtu");
    }

    #[tokio::test]
    async fn add_filter_places_the_rule_before_another() {
        use wiremock::matchers::body_json;
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/firewall/filter"))
            .and(body_json(json!({
                "chain": "input",
                "action": "accept",
                "protocol": "udp",
                "dst-port": "123",
                "in-interface-list": "LAN",
                "place-before": "*3"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*9"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddFirewallFilterParams {
            chain: "input".into(),
            action: "accept".into(),
            protocol: Some("udp".into()),
            dst_port: Some("123".into()),
            in_interface_list: Some("LAN".into()),
            place_before: Some(RouterOsId::try_from("*3").unwrap()),
            ..Default::default()
        };
        add_filter(&client, &p).await.unwrap();
    }

    #[tokio::test]
    async fn set_filter_patches_only_given_fields() {
        use wiremock::matchers::body_json;
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/firewall/filter/*5"))
            .and(body_json(json!({"in-interface-list": "LAN"})))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({".id": "*5", "in-interface-list": "LAN"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        // The rule properties sit next to the id in the tool arguments.
        let p: SetFirewallFilterParams =
            serde_json::from_value(json!({"id": "*5", "in_interface_list": "LAN"})).unwrap();
        let out = set_filter(&client, &p).await.unwrap();
        assert_eq!(out["in-interface-list"], "LAN");
    }

    #[tokio::test]
    async fn set_nat_and_mangle_patch_their_own_tables() {
        use wiremock::matchers::body_json;
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/firewall/nat/*2"))
            .and(body_json(json!({"to-ports": "8080", "disabled": "false"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*2"})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/rest/ip/firewall/mangle/*3"))
            .and(body_json(json!({"new-mss": "1360"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*3"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let nat = SetFirewallNatParams {
            id: RouterOsId::try_from("*2").unwrap(),
            rule: SetNatRule {
                to_ports: Some("8080".into()),
                disabled: Some(false),
                ..Default::default()
            },
        };
        set_nat(&client, &nat).await.unwrap();
        let mangle = SetFirewallMangleParams {
            id: RouterOsId::try_from("*3").unwrap(),
            rule: SetMangleRule {
                new_mss: Some("1360".into()),
                ..Default::default()
            },
        };
        set_mangle(&client, &mangle).await.unwrap();
    }

    #[tokio::test]
    async fn set_refuses_an_empty_change() {
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        let p = SetFirewallFilterParams {
            id: RouterOsId::try_from("*5").unwrap(),
            rule: SetFilterRule::default(),
        };
        let err = set_filter(&client, &p).await.unwrap_err();
        assert!(err.to_string().contains("set_firewall_filter"), "{err}");
    }

    #[tokio::test]
    async fn move_rule_posts_numbers_and_destination() {
        use wiremock::matchers::body_json;
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/ip/firewall/filter/move"))
            .and(body_json(json!({"numbers": "*5", "destination": "*1"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/ip/firewall/mangle/move"))
            .and(body_json(json!({"numbers": "*7"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = MoveFirewallRuleParams {
            id: RouterOsId::try_from("*5").unwrap(),
            destination: Some(RouterOsId::try_from("*1").unwrap()),
        };
        move_rule(&client, RuleTable::Filter, &p).await.unwrap();
        let p = MoveFirewallRuleParams {
            id: RouterOsId::try_from("*7").unwrap(),
            destination: None,
        };
        move_rule(&client, RuleTable::Mangle, &p).await.unwrap();
    }
}
