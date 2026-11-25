use schemars::JsonSchema;
use serde::Deserialize;

use super::RouterOsId;

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
pub struct DhcpLeaseIdParams {
    #[schemars(description = "Lease .id as returned by list_dhcp_leases (e.g. '*1')")]
    pub id: RouterOsId,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDhcpLeaseParams {
    #[schemars(description = "Lease .id as returned by list_dhcp_leases (e.g. '*1')")]
    pub id: RouterOsId,
    #[schemars(description = "New IP address for this lease")]
    pub address: Option<String>,
    #[schemars(description = "New client MAC address (e.g. 'AA:BB:CC:DD:EE:FF')")]
    pub mac_address: Option<String>,
    #[schemars(description = "Comment / hostname label for this lease")]
    pub comment: Option<String>,
    #[schemars(
        description = "DHCP server this lease belongs to (e.g. 'defconf'). Leave unset for 'all'"
    )]
    pub server: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddDhcpClientParams {
    #[schemars(description = "Interface to run the DHCP client on (e.g. 'ether1')")]
    pub interface: String,
    #[schemars(
        description = "Install a default route from the server's gateway: 'yes', 'no' or \
            'special-classless' (default 'yes')"
    )]
    pub add_default_route: Option<String>,
    #[schemars(
        description = "Distance of that default route (1..255); a higher one is a backup to \
            a lower one"
    )]
    pub default_route_distance: Option<u32>,
    #[schemars(description = "Use the DNS servers the DHCP server offers")]
    pub use_peer_dns: Option<bool>,
    #[schemars(description = "Use the NTP servers the DHCP server offers")]
    pub use_peer_ntp: Option<bool>,
    #[schemars(description = "Create the client disabled (default: false)")]
    pub disabled: Option<bool>,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDhcpClientParams {
    #[schemars(description = "DHCP client .id as returned by list_dhcp_clients (e.g. '*1')")]
    pub id: RouterOsId,
    #[schemars(description = "Interface the client runs on")]
    pub interface: Option<String>,
    #[schemars(description = "'yes', 'no' or 'special-classless'")]
    pub add_default_route: Option<String>,
    #[schemars(description = "Distance of the default route (1..255)")]
    pub default_route_distance: Option<u32>,
    #[schemars(description = "Use the DNS servers the DHCP server offers")]
    pub use_peer_dns: Option<bool>,
    #[schemars(description = "Use the NTP servers the DHCP server offers")]
    pub use_peer_ntp: Option<bool>,
    #[schemars(description = "Disable (true) or enable (false) the client")]
    pub disabled: Option<bool>,
    #[schemars(description = "Comment")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDhcpNetworkParams {
    #[schemars(description = "Network .id as returned by list_dhcp_networks (e.g. '*1')")]
    pub id: RouterOsId,
    #[schemars(description = "Default gateway handed to clients")]
    pub gateway: Option<String>,
    #[schemars(description = "Comma-separated DNS servers handed to clients (e.g. '192.0.2.1')")]
    pub dns_server: Option<String>,
    #[schemars(description = "Comma-separated NTP servers handed to clients")]
    pub ntp_server: Option<String>,
    #[schemars(description = "DNS domain handed to clients (e.g. 'home.arpa')")]
    pub domain: Option<String>,
    #[schemars(description = "Comment")]
    pub comment: Option<String>,
}
