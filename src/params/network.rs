use schemars::JsonSchema;
use serde::Deserialize;

fn default_distance() -> u8 {
    1
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddRouteParams {
    #[schemars(description = "(required) Destination address/prefix, e.g. '0.0.0.0/0'")]
    pub dst_address: String,
    #[schemars(
        description = "(required) Gateway: next-hop IP or interface name, e.g. '192.168.188.1' or 'pppoe-out1'"
    )]
    pub gateway: String,
    #[serde(default = "default_distance")]
    #[schemars(
        description = "Administrative distance 1–255 — use 2 for a failover route (default: 1)"
    )]
    pub distance: u8,
    #[schemars(
        description = "Gateway check method: 'ping' or 'arp' — marks route inactive when gateway is unreachable"
    )]
    pub check_gateway: Option<String>,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
}
