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
