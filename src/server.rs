use anyhow::Context;
use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorCode, ErrorData, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use serde_json::Value;

use crate::client::RouterosClient;
use crate::error::tool_error;
use crate::params::*;
use crate::tools;

pub struct MikrotikServer {
    client: RouterosClient,
    password: String,
    backup_encrypt: bool,
    writes_enabled: bool,
}

impl MikrotikServer {
    pub fn from_env() -> anyhow::Result<Self> {
        let host = std::env::var("MIKROTIK_HOST").context("MIKROTIK_HOST not set")?;
        let port: u16 = std::env::var("MIKROTIK_PORT")
            .unwrap_or_else(|_| "443".into())
            .parse()
            .context("MIKROTIK_PORT must be a valid port number")?;
        let username = std::env::var("MIKROTIK_USER").unwrap_or_else(|_| "admin".into());
        let password = std::env::var("MIKROTIK_PASSWORD").context("MIKROTIK_PASSWORD not set")?;
        let tls_verify = std::env::var("MIKROTIK_TLS_VERIFY")
            .map(|v| !matches!(v.as_str(), "false" | "0" | "no"))
            .unwrap_or(false);
        let backup_encrypt = std::env::var("MIKROTIK_BACKUP_ENCRYPT")
            .map(|v| !matches!(v.as_str(), "false" | "0" | "no"))
            .unwrap_or(true);
        let writes_enabled = std::env::var("MIKROTIK_ALLOW_WRITES")
            .map(|v| matches!(v.as_str(), "true" | "1" | "yes"))
            .unwrap_or(false);

        Ok(Self {
            client: RouterosClient::new(&host, port, &username, &password, tls_verify)?,
            password,
            backup_encrypt,
            writes_enabled,
        })
    }

    fn ok(value: &Value) -> CallToolResult {
        CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
        )])
    }

    fn ok_msg(msg: &str) -> CallToolResult {
        CallToolResult::success(vec![Content::text(msg)])
    }

    fn guard_write(&self) -> Result<(), ErrorData> {
        if self.writes_enabled {
            Ok(())
        } else {
            Err(ErrorData::new(
                ErrorCode::INVALID_REQUEST,
                "write operations are disabled — set MIKROTIK_ALLOW_WRITES=true to enable",
                None,
            ))
        }
    }

    fn require_field<'a>(value: &'a str, field: &str) -> Result<&'a str, ErrorData> {
        if value.trim().is_empty() {
            Err(ErrorData::new(
                ErrorCode::INVALID_PARAMS,
                format!("'{field}' is required and must not be empty"),
                None,
            ))
        } else {
            Ok(value)
        }
    }
}

#[tool_router]
impl MikrotikServer {
    // ── System ────────────────────────────────────────────────────────────────

    #[tool(
        description = "Get RouterOS system resources: CPU load, free memory, uptime, version, board name"
    )]
    async fn get_system_resources(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::system::get_resources(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Get device identity (hostname)")]
    async fn get_system_identity(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::system::get_identity(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Set the device identity (hostname) — /system identity set name=...")]
    async fn set_system_identity(
        &self,
        Parameters(p): Parameters<SetSystemIdentityParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        let data = tools::system::set_identity(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Get recent system log entries, optionally filtered by topic and count")]
    async fn get_logs(
        &self,
        Parameters(p): Parameters<GetLogsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = tools::system::get_logs(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Reboot the device — /system reboot. WARNING: drops all connectivity \
            (incl. this MCP session) for ~1–2 minutes while it restarts."
    )]
    async fn reboot_router(&self) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let msg = tools::system::reboot(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&msg))
    }

    #[tool(
        description = "Save an encrypted binary .backup to the device, download it, and write it to a local path. Encrypted with MIKROTIK_PASSWORD by default."
    )]
    async fn save_backup(
        &self,
        Parameters(p): Parameters<SaveBackupParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let msg = tools::system::save_backup(&self.client, &p, &self.password, self.backup_encrypt)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&msg))
    }

    #[tool(
        description = "Upload a local .backup file to the device and load it. Decrypted with MIKROTIK_PASSWORD by default. WARNING: device will reboot after restore."
    )]
    async fn restore_backup(
        &self,
        Parameters(p): Parameters<RestoreBackupParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let msg =
            tools::system::restore_backup(&self.client, &p, &self.password, self.backup_encrypt)
                .await
                .map_err(tool_error)?;
        Ok(Self::ok_msg(&msg))
    }

    // ── Interfaces ────────────────────────────────────────────────────────────

    #[tool(
        description = "List all network interfaces with type, MAC address, MTU, and running status"
    )]
    async fn list_interfaces(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::interfaces::list_interfaces(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "List wireless registration table — connected clients with MAC, SSID, signal strength, TX/RX rate, and uptime"
    )]
    async fn list_wireless_registrations(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::interfaces::list_wireless_registrations(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Get details for a specific interface by name")]
    async fn get_interface(
        &self,
        Parameters(p): Parameters<GetInterfaceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = tools::interfaces::get_interface(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Enable an interface — /interface enable. Accepts the interface name or .id."
    )]
    async fn enable_interface(
        &self,
        Parameters(p): Parameters<InterfaceNameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.interface, "interface")?;
        tools::interfaces::enable_interface(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("enabled"))
    }

    #[tool(
        description = "Disable an interface — /interface disable. Accepts the interface name or .id. \
            Disabling a WAN interface (e.g. pppoe-out1) withdraws its routes."
    )]
    async fn disable_interface(
        &self,
        Parameters(p): Parameters<InterfaceNameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.interface, "interface")?;
        tools::interfaces::disable_interface(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("disabled"))
    }

    #[tool(
        description = "List interface-list members — which interfaces belong to which list (e.g. WAN, LAN)"
    )]
    async fn list_interface_list_members(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::interfaces::list_interface_list_members(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Add an interface to an interface list (e.g. add lte1 to WAN)")]
    async fn add_interface_list_member(
        &self,
        Parameters(p): Parameters<AddInterfaceListMemberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.list, "list")?;
        Self::require_field(&p.interface, "interface")?;
        let data = tools::interfaces::add_interface_list_member(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "List bridge ports (/interface/bridge/port) — which interfaces are \
            bridged, their bridge, pvid, and status (e.g. in-bridge)."
    )]
    async fn list_bridge_ports(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::interfaces::list_bridge_ports(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Add an interface to a bridge (/interface/bridge/port add). Use after \
            add_wifi_ap so a new virtual-AP SSID can pass LAN traffic. pvid defaults to the \
            bridge default (usually 1) when omitted."
    )]
    async fn add_bridge_port(
        &self,
        Parameters(p): Parameters<AddBridgePortParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.bridge, "bridge")?;
        Self::require_field(&p.interface, "interface")?;
        let data = tools::interfaces::add_bridge_port(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Remove an interface from the bridge (/interface/bridge/port remove), \
            located by interface name. Required before converting a bridged AP wifi interface \
            to station/WAN mode — a WAN interface must not be a bridge slave."
    )]
    async fn remove_bridge_port(
        &self,
        Parameters(p): Parameters<InterfaceNameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.interface, "interface")?;
        tools::interfaces::remove_bridge_port(&self.client, &p.interface)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "{} removed from bridge",
            p.interface
        )))
    }

    #[tool(description = "Remove an interface-list member by .id")]
    async fn remove_interface_list_member(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::interfaces::remove_interface_list_member(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── IP Addresses ──────────────────────────────────────────────────────────

    #[tool(description = "List all IP addresses assigned to interfaces")]
    async fn list_ip_addresses(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::ip::list_addresses(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Assign an IP address (with prefix) to an interface")]
    async fn add_ip_address(
        &self,
        Parameters(p): Parameters<AddIpAddressParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let data = tools::ip::add_address(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove an IP address by its .id (from list_ip_addresses)")]
    async fn remove_ip_address(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::ip::remove_address(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Firewall — filter ─────────────────────────────────────────────────────

    #[tool(description = "List firewall filter rules (input / forward / output chains)")]
    async fn list_firewall_filter(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::firewall::list_filter(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Add a firewall filter rule")]
    async fn add_firewall_filter(
        &self,
        Parameters(p): Parameters<AddFirewallFilterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let data = tools::firewall::add_filter(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a firewall filter rule by .id")]
    async fn remove_firewall_filter(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::firewall::remove_filter(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Firewall — NAT ────────────────────────────────────────────────────────

    #[tool(description = "List NAT rules (srcnat / dstnat chains)")]
    async fn list_firewall_nat(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::firewall::list_nat(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Add a NAT rule (masquerade, port-forward, etc.)")]
    async fn add_firewall_nat(
        &self,
        Parameters(p): Parameters<AddFirewallNatParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let data = tools::firewall::add_nat(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a NAT rule by .id")]
    async fn remove_firewall_nat(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::firewall::remove_nat(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Firewall — mangle ─────────────────────────────────────────────────────

    #[tool(
        description = "List firewall mangle rules (prerouting / forward / postrouting marking, MSS)"
    )]
    async fn list_firewall_mangle(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::firewall::list_mangle(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Add a firewall mangle rule — e.g. action=change-mss new-mss=clamp-to-pmtu \
            protocol=tcp tcp-flags=syn out-interface-list=WAN to clamp MSS on a WAN/LTE uplink, \
            or mark-connection / mark-routing for policy routing"
    )]
    async fn add_firewall_mangle(
        &self,
        Parameters(p): Parameters<AddFirewallMangleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.chain, "chain")?;
        Self::require_field(&p.action, "action")?;
        let data = tools::firewall::add_mangle(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a firewall mangle rule by .id")]
    async fn remove_firewall_mangle(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::firewall::remove_mangle(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Firewall — address list ───────────────────────────────────────────────

    #[tool(description = "List firewall address-list entries (name, address, timeout, dynamic)")]
    async fn list_firewall_address_list(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::firewall::list_address_list(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Add an entry to a firewall address-list — an IP, CIDR subnet, or range \
            grouped under a list name for use with src-address-list / dst-address-list matchers"
    )]
    async fn add_firewall_address_list(
        &self,
        Parameters(p): Parameters<AddFirewallAddressListParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.list, "list")?;
        Self::require_field(&p.address, "address")?;
        let data = tools::firewall::add_address_list(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a firewall address-list entry by .id")]
    async fn remove_firewall_address_list(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::firewall::remove_address_list(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── DHCP ──────────────────────────────────────────────────────────────────

    #[tool(description = "List configured DHCP servers and their interfaces / address pools")]
    async fn list_dhcp_servers(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::dhcp::list_servers(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "List DHCP leases — both dynamic and static bindings")]
    async fn list_dhcp_leases(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::dhcp::list_leases(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Create a static DHCP lease: bind a MAC address to a fixed IP")]
    async fn add_dhcp_static_lease(
        &self,
        Parameters(p): Parameters<AddDhcpStaticLeaseParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let data = tools::dhcp::add_static_lease(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a DHCP lease by .id")]
    async fn remove_dhcp_lease(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::dhcp::remove_lease(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    #[tool(
        description = "List DHCP clients (/ip/dhcp-client) — per-interface WAN DHCP config incl. \
            add-default-route, default-route-distance, and assigned address"
    )]
    async fn list_dhcp_clients(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::dhcp::list_clients(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Update a DHCP client (located by interface) — /ip dhcp-client set. Set \
            add_default_route=false to manage the WAN default route statically (e.g. a \
            check-gateway primary + LTE backup for failover)."
    )]
    async fn set_dhcp_client(
        &self,
        Parameters(p): Parameters<SetDhcpClientParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.interface, "interface")?;
        let msg = tools::dhcp::set_client(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&msg))
    }

    #[tool(
        description = "Remove a DHCP client by interface name — /ip dhcp-client remove. \
            Use when tearing down a WAN interface (e.g. wifi3) to clean up its DHCP config."
    )]
    async fn remove_dhcp_client(
        &self,
        Parameters(p): Parameters<InterfaceNameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.interface, "interface")?;
        tools::dhcp::remove_client(&self.client, &p.interface)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "DHCP client on {} removed",
            p.interface
        )))
    }

    #[tool(
        description = "Add a new DHCP client on an interface — /ip dhcp-client add. Use this to \
            add DHCP on a newly created interface (e.g. a wifi station VIF). Set \
            add_default_route=true and default_route_distance to integrate into failover routing \
            (e.g. distance=3 for a tertiary hotspot behind ether1 primary and lte1 backup)."
    )]
    async fn add_dhcp_client(
        &self,
        Parameters(p): Parameters<AddDhcpClientParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.interface, "interface")?;
        let data = tools::dhcp::add_client(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    // ── DNS ───────────────────────────────────────────────────────────────────

    #[tool(
        description = "Get DNS settings: upstream servers, cache max TTL / size, DoH configuration"
    )]
    async fn get_dns_settings(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::dns::get_settings(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "List static DNS A records configured on the router")]
    async fn list_dns_static(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::dns::list_static(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Add a static DNS A record (useful for local .home.arpa hostnames)")]
    async fn add_dns_static(
        &self,
        Parameters(p): Parameters<AddDnsStaticParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let data = tools::dns::add_static(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a static DNS entry by .id")]
    async fn remove_dns_static(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::dns::remove_static(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Routes & Neighbors ────────────────────────────────────────────────────

    #[tool(
        description = "List IP routing table entries including active routes, gateway, and distance"
    )]
    async fn list_routes(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::network::list_routes(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Add a static route to /ip/route — set distance=2 with check-gateway=ping \
            for a failover route that becomes active only when the primary gateway is unreachable"
    )]
    async fn add_route(
        &self,
        Parameters(p): Parameters<AddRouteParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.dst_address, "dst_address")?;
        Self::require_field(&p.gateway, "gateway")?;
        let data = tools::network::add_route(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a static route by .id (from list_routes)")]
    async fn remove_route(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::network::remove_route(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    #[tool(
        description = "List IP neighbors discovered via neighbor discovery protocols (CDP/LLDP/MNDP) \
            — shows board model, identity, IP address, MAC, interface, and uptime for each neighbor"
    )]
    async fn list_neighbors(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::network::list_neighbors(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    // ── LTE ───────────────────────────────────────────────────────────────────

    #[tool(
        description = "Get LTE/5G modem info for the named interface — signal strength \
            (RSRP, RSRQ, RSSI), operator, band, PIN status, and network registration state"
    )]
    async fn get_lte_info(
        &self,
        Parameters(p): Parameters<GetLteInfoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = tools::lte::get_lte_info(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    // ── LTE — eSIM (eUICC) ─────────────────────────────────────────────────────

    #[tool(description = "Select the active SIM slot, e.g. 'esim' or 'sim' — \
            /interface lte settings set sim-slot=... . Switching to 'esim' is required \
            before downloading an eSIM profile.")]
    async fn set_lte_sim_slot(
        &self,
        Parameters(p): Parameters<SetSimSlotParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.sim_slot, "sim_slot")?;
        let out = tools::lte::set_sim_slot(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "sim-slot set to '{}'\n{out}",
            p.sim_slot
        )))
    }

    #[tool(
        description = "List eSIM (eUICC) profiles installed on the modem — name, ICCID, state, \
            and .id for use with activate/deactivate/remove."
    )]
    async fn list_esim_profiles(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::lte::list_esim_profiles(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Get the modem's eUICC identifier (EID) for the named LTE interface")]
    async fn get_esim_id(
        &self,
        Parameters(p): Parameters<LteInterfaceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let out = tools::lte::esim_id(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(out.trim()))
    }

    #[tool(
        description = "Stage an eSIM profile download from an SM-DP+ server using an activation \
            code (matching-id). WARNING: this CANNOT complete the install — RouterOS gates the \
            final download behind an interactive y/N consent the REST API cannot supply, so the \
            flow ends with 'user didn't approve', and a staged attempt can strand the profile in a \
            'Bad profile state' on the server. To actually install a profile, run \
            '/interface/lte/esim provision ...' in a router terminal and press 'y'. Use this tool \
            for staging/diagnostics only; it reports the SM-DP+ status verbatim. Requires internet \
            connectivity to reach the SM-DP+ server."
    )]
    async fn provision_esim(
        &self,
        Parameters(p): Parameters<ProvisionEsimParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.sm_dp_plus, "sm_dp_plus")?;
        Self::require_field(&p.matching_id, "matching_id")?;
        let out = tools::lte::provision_esim(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "eSIM provisioning result:\n{}",
            out.trim()
        )))
    }

    #[tool(
        description = "Activate an installed eSIM profile by its number/.id (from list_esim_profiles)"
    )]
    async fn activate_esim_profile(
        &self,
        Parameters(p): Parameters<EsimProfileNumberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.number, "number")?;
        let out = tools::lte::activate_esim(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "eSIM profile {} activated\n{}",
            p.number,
            out.trim()
        )))
    }

    #[tool(
        description = "Deactivate an installed eSIM profile by its number/.id (from list_esim_profiles)"
    )]
    async fn deactivate_esim_profile(
        &self,
        Parameters(p): Parameters<EsimProfileNumberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.number, "number")?;
        let out = tools::lte::deactivate_esim(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "eSIM profile {} deactivated\n{}",
            p.number,
            out.trim()
        )))
    }

    #[tool(
        description = "Delete an installed eSIM profile from the modem by its number/.id (from list_esim_profiles)"
    )]
    async fn remove_esim_profile(
        &self,
        Parameters(p): Parameters<EsimProfileNumberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.number, "number")?;
        let out = tools::lte::delete_esim(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "eSIM profile {} deleted\n{}",
            p.number,
            out.trim()
        )))
    }

    #[tool(
        description = "Send a raw AT command to the LTE modem via /interface/lte/at-chat and \
            return its output. Useful for diagnostics: 'AT+CSQ' (signal quality), \
            'AT+QENG=\"servingcell\"' (serving cell / RSRP / band), 'AT+QNWPREFCFG=\"mode_pref\"' \
            (query technology preference)."
    )]
    async fn lte_at_chat(
        &self,
        Parameters(p): Parameters<LteAtChatParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.command, "command")?;
        let data = tools::lte::lte_at_chat(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Force the modem's radio technology — mode 'lte' (4G only), 'nr5g' (5G \
            only), or 'auto'. Forcing 'lte' often fixes attach failures on 5G modems (e.g. RG650E) \
            that wedge during NR negotiation. Sends AT+QNWPREFCFG=\"mode_pref\",<MODE> via at-chat."
    )]
    async fn set_lte_technology(
        &self,
        Parameters(p): Parameters<SetLteTechnologyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.mode, "mode")?;
        let data = tools::lte::set_technology(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "List LTE APN profiles (/interface/lte/apn) — apn, add-default-route, \
            default-route-distance, and which profile is the default"
    )]
    async fn list_lte_apn_profiles(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::lte::list_apn_profiles(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Update an LTE APN profile (by name, or the default profile) — \
            /interface lte apn set. Set default_route_distance=2 (or add_default_route=false) to \
            make LTE a backup WAN behind a distance-1 primary."
    )]
    async fn set_lte_apn_profile(
        &self,
        Parameters(p): Parameters<SetLteApnProfileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let msg = tools::lte::set_apn_profile(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&msg))
    }

    // ── WiFi ──────────────────────────────────────────────────────────────────

    #[tool(
        description = "List wifi interfaces (/interface/wifi) — name, mode (ap/station), \
            master-interface, SSID, MAC address, and running state. Use this to inspect \
            both AP interfaces and any station VIFs added for upstream connectivity."
    )]
    async fn list_wifi_interfaces(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::wifi::list_wifi_interfaces(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Run a passive wifi scan on an interface (/interface/wifi/scan) and \
            return visible APs with SSID, BSSID, channel, signal strength, and security. \
            Use wifi1 (5 GHz) or wifi2 (2.4 GHz). Duration defaults to 5 seconds."
    )]
    async fn scan_wifi(
        &self,
        Parameters(p): Parameters<ScanWifiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::require_field(&p.interface, "interface")?;
        let result = tools::wifi::scan_wifi(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&result))
    }

    #[tool(
        description = "List wifi security profiles (/interface/wifi/security) — name, \
            authentication-types, and passphrase. Use these profile names in add_wifi_station."
    )]
    async fn list_wifi_security(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::wifi::list_wifi_security(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Create a named wifi security profile (/interface/wifi/security add) — \
            stores SSID credentials so they can be referenced by name in add_wifi_station. \
            authentication_types defaults to 'wpa2-psk'; use 'wpa2-psk,wpa3-psk' for mixed."
    )]
    async fn add_wifi_security(
        &self,
        Parameters(p): Parameters<AddWifiSecurityParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        Self::require_field(&p.passphrase, "passphrase")?;
        let data = tools::wifi::add_wifi_security(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Modify an existing wifi interface (/interface/wifi set) — change mode, \
            SSID, or security profile. Use mode='station' + ssid + security to convert an \
            idle AP radio to a standalone WAN uplink (e.g. connecting wifi1 to a phone \
            hotspot). Call remove_bridge_port first if the interface is currently in the \
            LAN bridge."
    )]
    async fn set_wifi_interface(
        &self,
        Parameters(p): Parameters<SetWifiInterfaceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        let msg = tools::wifi::set_wifi_interface(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&msg))
    }

    #[tool(
        description = "Remove a wifi interface by name (/interface/wifi remove). \
            Use this to clean up a station VIF (e.g. wifi3) after migrating to a \
            standalone interface."
    )]
    async fn remove_wifi_interface(
        &self,
        Parameters(p): Parameters<WifiNameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        tools::wifi::remove_wifi_interface(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!(
            "wifi interface '{}' removed",
            p.name
        )))
    }

    #[tool(
        description = "Add a virtual wifi station interface on top of an existing radio \
            (/interface/wifi add mode=station). The master radio keeps serving as an AP; \
            this VIF connects upstream to the named SSID. Typical use: wifi3 on master \
            wifi1 to connect to a phone hotspot as a tertiary WAN. Pair with \
            add_dhcp_client (distance=3) and add_interface_list_member (list=WAN) to \
            complete failover integration."
    )]
    async fn add_wifi_station(
        &self,
        Parameters(p): Parameters<AddWifiStationParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        Self::require_field(&p.master_interface, "master_interface")?;
        Self::require_field(&p.ssid, "ssid")?;
        let data = tools::wifi::add_wifi_station(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Add a virtual AP interface on top of an existing radio \
            (/interface/wifi add mode=ap) to broadcast an additional SSID (second BSSID) \
            on the same radio. Reference the master interface's security profile to make \
            it a true alias (edit the profile once, both SSIDs update). Follow with \
            add_bridge_port to give the new SSID LAN access."
    )]
    async fn add_wifi_ap(
        &self,
        Parameters(p): Parameters<AddWifiApParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        Self::require_field(&p.master_interface, "master_interface")?;
        Self::require_field(&p.ssid, "ssid")?;
        let data = tools::wifi::add_wifi_ap(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    // ── Files ───────────────────────────────────────────────────────────────────

    #[tool(
        description = "List files on the device (/file) — name, type, size, creation time. \
            Use to find backup files left on the device (e.g. after save_backup) before \
            cleaning them up with remove_file."
    )]
    async fn list_files(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::files::list_files(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Remove a file from the device by name (/file remove), resolved via \
            list_files. Typical use: delete the on-device backup file left behind after \
            save_backup has downloaded it locally."
    )]
    async fn remove_file(
        &self,
        Parameters(p): Parameters<RemoveFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        tools::files::remove_file(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg(&format!("file '{}' removed", p.name)))
    }

    // ── Disk ──────────────────────────────────────────────────────────────────

    #[tool(
        description = "List storage disks and partitions (/disk) — slot, type (hardware/partition), \
            filesystem, model, size, and free space. Use this to find USB storage (e.g. usb1-part1)."
    )]
    async fn list_disks(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::disk::list_disks(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    // ── SMB (file sharing) ────────────────────────────────────────────────────

    #[tool(
        description = "Get SMB service state (/ip/smb): enabled, domain, bound interfaces, status"
    )]
    async fn get_smb(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::smb::get_smb(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Configure the SMB service (/ip/smb set): enabled (yes/no/auto), domain, \
            interfaces. SECURITY: 'interfaces' takes an interface NAME (e.g. 'bridge'), not an \
            interface-list name. Bind to a LAN interface — 'all' exposes TCP 445 on the WAN/5G side."
    )]
    async fn set_smb(
        &self,
        Parameters(p): Parameters<SetSmbParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        let data = tools::smb::set_smb(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "List SMB shares (/ip/smb/shares). RouterOS auto-creates one share per USB \
            partition (directory == disk slot), so existing shares appear here without setup."
    )]
    async fn list_smb_shares(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::smb::list_shares(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Add an SMB share (/ip/smb/shares) — name + directory (e.g. 'usb1-part1')"
    )]
    async fn add_smb_share(
        &self,
        Parameters(p): Parameters<AddSmbShareParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        Self::require_field(&p.directory, "directory")?;
        let data = tools::smb::add_share(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove an SMB share by .id (from list_smb_shares)")]
    async fn remove_smb_share(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::smb::remove_share(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    #[tool(description = "List SMB users (/ip/smb/users) — name, read-only, disabled state")]
    async fn list_smb_users(&self) -> Result<CallToolResult, ErrorData> {
        let data = tools::smb::list_users(&self.client)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Add an SMB user (/ip/smb/users) — name + password. Access is read-only by \
            default; set read_only=false to grant read-write."
    )]
    async fn add_smb_user(
        &self,
        Parameters(p): Parameters<AddSmbUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.name, "name")?;
        Self::require_field(&p.password, "password")?;
        let data = tools::smb::add_user(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(
        description = "Update an SMB user (/ip/smb/users/set) by .id — password, read_only, disabled"
    )]
    async fn set_smb_user(
        &self,
        Parameters(p): Parameters<SetSmbUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        Self::require_field(&p.id, "id")?;
        let data = tools::smb::set_user(&self.client, &p)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove an SMB user by .id (from list_smb_users)")]
    async fn remove_smb_user(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_write()?;
        tools::smb::remove_user(&self.client, &p.id)
            .await
            .map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }
}

#[tool_handler]
impl ServerHandler for MikrotikServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "MikroTik RouterOS management via REST API (RouterOS 7.1+). \
                Configure with MIKROTIK_HOST, MIKROTIK_USER, MIKROTIK_PASSWORD env vars. \
                Optional: MIKROTIK_PORT (default 443), MIKROTIK_TLS_VERIFY (default false), \
                MIKROTIK_ALLOW_WRITES (default false — must be 'true' to enable mutating tools).",
        )
    }
}
