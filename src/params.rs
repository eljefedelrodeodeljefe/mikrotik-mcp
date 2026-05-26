use schemars::JsonSchema;
use serde::Deserialize;

// Shared

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveByIdParams {
    #[schemars(description = "Item .id as returned by the list command (e.g. '*1')")]
    pub id: String,
}

// System

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveBackupParams {
    #[schemars(description = "Backup name without extension (e.g. 'r-ap-1-2026-05-16')")]
    pub name: String,
    #[schemars(description = "Absolute path on the local machine to write the .backup file")]
    pub output_path: String,
    #[schemars(description = "Encryption password — defaults to MIKROTIK_PASSWORD when omitted")]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RestoreBackupParams {
    #[schemars(description = "Absolute path to the local .backup file to restore")]
    pub input_path: String,
    #[schemars(description = "Decryption password — defaults to MIKROTIK_PASSWORD when omitted")]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLogsParams {
    #[schemars(description = "Maximum number of log entries to return (default: 50)")]
    pub count: Option<u32>,
    #[schemars(description = "Filter entries whose topics field contains this string (e.g. 'dhcp', 'firewall')")]
    pub topics: Option<String>,
}

// Interfaces

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetInterfaceParams {
    #[schemars(description = "Interface name (e.g. 'ether1', 'bridge')")]
    pub name: String,
}

// IP addresses

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddIpAddressParams {
    #[schemars(description = "IP address with prefix length (e.g. '192.168.1.1/24')")]
    pub address: String,
    #[schemars(description = "Interface to assign the address to (e.g. 'ether1', 'bridge')")]
    pub interface: String,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
}

// Firewall

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

// DHCP

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddDhcpStaticLeaseParams {
    #[schemars(description = "Client MAC address (e.g. 'AA:BB:CC:DD:EE:FF')")]
    pub mac_address: String,
    #[schemars(description = "IP address to assign to this MAC")]
    pub address: String,
    #[schemars(description = "Optional hostname / comment")]
    pub comment: Option<String>,
}

// DNS

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddDnsStaticParams {
    #[schemars(description = "Hostname to resolve (e.g. 'nas.home.arpa')")]
    pub name: String,
    #[schemars(description = "IPv4 address to resolve to")]
    pub address: String,
    #[schemars(description = "TTL in seconds (default: 86400)")]
    pub ttl: Option<u32>,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
}
