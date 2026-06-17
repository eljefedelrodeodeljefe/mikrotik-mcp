use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddDhcpStaticLeaseParams {
    #[schemars(description = "Client MAC address (e.g. 'AA:BB:CC:DD:EE:FF')")]
    pub mac_address: String,
    #[schemars(description = "IP address to assign to this MAC")]
    pub address: String,
    #[schemars(description = "Optional hostname / comment")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDhcpClientParams {
    #[schemars(
        description = "Interface whose DHCP client to modify (e.g. 'ether1') — resolved to its \
            .id via list_dhcp_clients"
    )]
    pub interface: String,
    #[schemars(
        description = "Whether the DHCP client installs a default route from the lease (yes/no). \
            Set 'no' to manage the WAN default route statically, e.g. for failover."
    )]
    pub add_default_route: Option<bool>,
    #[schemars(
        description = "Administrative distance for the DHCP-installed default route (1–255)"
    )]
    pub default_route_distance: Option<u8>,
    #[schemars(description = "Whether to use DNS servers advertised by the DHCP server (yes/no)")]
    pub use_peer_dns: Option<bool>,
}
