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
pub struct DhcpLeaseIdParams {
    #[schemars(description = "Lease .id as returned by list_dhcp_leases (e.g. '*1')")]
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDhcpLeaseParams {
    #[schemars(description = "Lease .id as returned by list_dhcp_leases (e.g. '*1')")]
    pub id: String,
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
