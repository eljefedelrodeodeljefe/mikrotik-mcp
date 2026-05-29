use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddFirewallFilterParams {
    #[schemars(description = "Chain: 'input', 'forward', or 'output'")]
    pub chain: String,
    #[schemars(description = "Action: 'accept', 'drop', 'reject', 'log', 'passthrough'")]
    pub action: String,
    #[schemars(description = "Source IP address or CIDR range")]
    pub src_address: Option<String>,
    #[schemars(description = "Destination IP address or CIDR range")]
    pub dst_address: Option<String>,
    #[schemars(description = "Protocol: 'tcp', 'udp', 'icmp', etc.")]
    pub protocol: Option<String>,
    #[schemars(description = "Destination port or range (e.g. '80', '8080-8090')")]
    pub dst_port: Option<String>,
    #[schemars(description = "Match traffic arriving on this interface")]
    pub in_interface: Option<String>,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
    #[schemars(description = "Create the rule in disabled state (default: false)")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddFirewallNatParams {
    #[schemars(description = "Chain: 'srcnat' or 'dstnat'")]
    pub chain: String,
    #[schemars(description = "Action: 'masquerade', 'src-nat', 'dst-nat', 'redirect'")]
    pub action: String,
    #[schemars(description = "Source address to match (CIDR)")]
    pub src_address: Option<String>,
    #[schemars(description = "Destination address to match (CIDR)")]
    pub dst_address: Option<String>,
    #[schemars(description = "Protocol: 'tcp' or 'udp'")]
    pub protocol: Option<String>,
    #[schemars(description = "Destination port to match")]
    pub dst_port: Option<String>,
    #[schemars(description = "Translation target address (for src-nat / dst-nat)")]
    pub to_addresses: Option<String>,
    #[schemars(description = "Translation target port (for dst-nat / redirect)")]
    pub to_ports: Option<String>,
    #[schemars(description = "Out interface (for masquerade / src-nat)")]
    pub out_interface: Option<String>,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddFirewallAddressListParams {
    #[schemars(
        description = "Address list name (referenced by src-address-list / dst-address-list)"
    )]
    pub list: String,
    #[schemars(description = "IP address, CIDR subnet, or range to add to the list")]
    pub address: String,
    #[schemars(description = "Optional timeout (e.g. '1h', '30m'); omit for a permanent entry")]
    pub timeout: Option<String>,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
}
