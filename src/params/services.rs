use schemars::JsonSchema;
use serde::Deserialize;

use super::RouterOsId;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetIpServiceParams {
    #[schemars(
        description = "Service .id as returned by list_ip_services (e.g. '*5'). Pass either \
            id or name"
    )]
    pub id: Option<RouterOsId>,
    #[schemars(
        description = "Service name: 'telnet', 'ftp', 'www', 'ssh', 'www-ssl', 'api', \
            'winbox', 'api-ssl'. Pass either id or name"
    )]
    pub name: Option<String>,
    #[schemars(description = "Disable (true) or enable (false) the service")]
    pub disabled: Option<bool>,
    #[schemars(description = "TCP port the service listens on (1..65535)")]
    pub port: Option<u32>,
    #[schemars(
        description = "Comma-separated IPv4/IPv6 prefixes allowed to use the service \
            (available-from; RouterOS versions that list this as address need that name \
            instead), e.g. '192.0.2.0/24,2001:db8::/32'. Empty allows any address"
    )]
    pub available_from: Option<String>,
    #[schemars(
        description = "Certificate name for www-ssl and api-ssl, or 'none' (from /certificate)"
    )]
    pub certificate: Option<String>,
    #[schemars(
        description = "TLS versions accepted by www-ssl and api-ssl: 'any', 'only-1.2', and on \
            newer RouterOS 'only-1.3', 'min-1.2', 'min-1.3'"
    )]
    pub tls_version: Option<String>,
    #[schemars(description = "Maximum simultaneous sessions (1..1000)")]
    pub max_sessions: Option<u32>,
    #[schemars(description = "VRF the service listens in (default 'main')")]
    pub vrf: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetIpCloudParams {
    #[schemars(
        description = "DDNS: 'yes' or 'auto' (on while Back To Home is enabled) on current \
            RouterOS; older versions take 'yes' or 'no'"
    )]
    pub ddns_enabled: Option<String>,
    #[schemars(
        description = "Extra DDNS update interval, e.g. '10m', or 'none' (default) for the \
            router's own updates only"
    )]
    pub ddns_update_interval: Option<String>,
    #[schemars(
        description = "Set the clock from the cloud server at startup while the NTP client \
            is disabled"
    )]
    pub update_time: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetMacServerParams {
    #[schemars(
        description = "Interface list MAC Telnet is allowed on (/tool mac-server \
            allowed-interface-list), e.g. 'LAN' or 'none'"
    )]
    pub allowed_interface_list: Option<String>,
    #[schemars(
        description = "Interface list MAC WinBox is allowed on (/tool mac-server mac-winbox \
            allowed-interface-list), e.g. 'LAN' or 'none'"
    )]
    pub mac_winbox_allowed_interface_list: Option<String>,
    #[schemars(description = "Answer MAC ping (/tool mac-server ping enabled)")]
    pub ping_enabled: Option<bool>,
}
