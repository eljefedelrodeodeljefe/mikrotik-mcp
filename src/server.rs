use anyhow::Context;
use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, ErrorData, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
};
use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::error::tool_error;
use crate::params::*;

pub struct MikrotikServer {
    client: RouterosClient,
    password: String,
    backup_encrypt: bool,
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

        Ok(Self {
            client: RouterosClient::new(&host, port, &username, &password, tls_verify)?,
            password,
            backup_encrypt,
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
}

#[tool_router]
impl MikrotikServer {
    // ── System ────────────────────────────────────────────────────────────────

    #[tool(description = "Get RouterOS system resources: CPU load, free memory, uptime, version, board name")]
    async fn get_system_resources(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("system/resource").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Get device identity (hostname)")]
    async fn get_system_identity(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("system/identity").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Save an encrypted binary .backup to the device, download it, and write it to a local path. Encrypted with MIKROTIK_PASSWORD by default.")]
    async fn save_backup(
        &self,
        Parameters(p): Parameters<SaveBackupParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut body = json!({"name": p.name});
        if self.backup_encrypt {
            let pw = p.password.as_deref().unwrap_or(&self.password);
            body["password"] = json!(pw);
        }

        self.client.post_void("system/backup/save", &body).await
            .map_err(|e| tool_error(e.context("step 1: POST system/backup/save failed")))?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let filename = format!("{}.backup", p.name);
        let bytes = self.client.ftp_download(&filename).await
            .map_err(|e| tool_error(e.context("step 2: FTP download failed")))?;

        let path = std::path::Path::new(&p.output_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| tool_error(anyhow::anyhow!("step 3: create_dir_all: {e}")))?;
        }
        std::fs::write(path, &bytes)
            .map_err(|e| tool_error(anyhow::anyhow!("step 3: write file: {e}")))?;

        Ok(Self::ok_msg(&format!(
            "backup saved to {} ({} bytes, {})",
            p.output_path,
            bytes.len(),
            if self.backup_encrypt { "encrypted" } else { "unencrypted" },
        )))
    }

    #[tool(description = "Upload a local .backup file to the device and load it. Decrypted with MIKROTIK_PASSWORD by default. WARNING: device will reboot after restore.")]
    async fn restore_backup(
        &self,
        Parameters(p): Parameters<RestoreBackupParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = std::path::Path::new(&p.input_path);
        let remote_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| tool_error(anyhow::anyhow!("invalid input_path")))?
            .to_string();

        self.client.ftp_upload(&p.input_path, &remote_name).await
            .map_err(|e| tool_error(e.context("step 1: FTP upload failed")))?;

        let name_without_ext = remote_name.strip_suffix(".backup").unwrap_or(&remote_name);
        let mut body = json!({"name": name_without_ext});
        if self.backup_encrypt {
            let pw = p.password.as_deref().unwrap_or(&self.password);
            body["password"] = json!(pw);
        }

        self.client.post_void("system/backup/load", &body).await
            .map_err(|e| tool_error(e.context("step 2: POST system/backup/load failed")))?;

        Ok(Self::ok_msg(&format!(
            "backup {} loaded — device is rebooting",
            remote_name
        )))
    }

    #[tool(description = "Get recent system log entries, optionally filtered by topic and count")]
    async fn get_logs(
        &self,
        Parameters(p): Parameters<GetLogsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut entries: Vec<Value> = self.client.get("log").await.map_err(tool_error)?;
        if let Some(topic) = &p.topics {
            entries.retain(|e| {
                e.get("topics")
                    .and_then(|t| t.as_str())
                    .is_some_and(|t| t.contains(topic.as_str()))
            });
        }
        entries.truncate(p.count.unwrap_or(50) as usize);
        Ok(Self::ok(&Value::Array(entries)))
    }

    // ── Interfaces ────────────────────────────────────────────────────────────

    #[tool(description = "List all network interfaces with type, MAC address, MTU, and running status")]
    async fn list_interfaces(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("interface").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "List wireless registration table — connected clients with MAC, SSID, signal strength, TX/RX rate, and uptime")]
    async fn list_wireless_registrations(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("interface/wireless/registration-table").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Get details for a specific interface by name")]
    async fn get_interface(
        &self,
        Parameters(p): Parameters<GetInterfaceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = format!("interface?name={}", p.name);
        let data: Value = self.client.get(&path).await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    // ── IP Addresses ──────────────────────────────────────────────────────────

    #[tool(description = "List all IP addresses assigned to interfaces")]
    async fn list_ip_addresses(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/address").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Assign an IP address (with prefix) to an interface")]
    async fn add_ip_address(
        &self,
        Parameters(p): Parameters<AddIpAddressParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut body = json!({"address": p.address, "interface": p.interface});
        if let Some(c) = p.comment { body["comment"] = json!(c); }
        let data: Value = self.client.post("ip/address", &body).await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove an IP address by its .id (from list_ip_addresses)")]
    async fn remove_ip_address(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.client.delete("ip/address", &p.id).await.map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Firewall — filter ─────────────────────────────────────────────────────

    #[tool(description = "List firewall filter rules (input / forward / output chains)")]
    async fn list_firewall_filter(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/firewall/filter").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Add a firewall filter rule")]
    async fn add_firewall_filter(
        &self,
        Parameters(p): Parameters<AddFirewallFilterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut body = json!({"chain": p.chain, "action": p.action});
        if let Some(v) = p.src_address  { body["src-address"]   = json!(v); }
        if let Some(v) = p.dst_address  { body["dst-address"]   = json!(v); }
        if let Some(v) = p.protocol     { body["protocol"]      = json!(v); }
        if let Some(v) = p.dst_port     { body["dst-port"]      = json!(v); }
        if let Some(v) = p.in_interface { body["in-interface"]  = json!(v); }
        if let Some(v) = p.comment      { body["comment"]       = json!(v); }
        if let Some(v) = p.disabled     { body["disabled"]      = json!(v); }
        let data: Value = self.client.post("ip/firewall/filter", &body).await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a firewall filter rule by .id")]
    async fn remove_firewall_filter(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.client.delete("ip/firewall/filter", &p.id).await.map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Firewall — NAT ────────────────────────────────────────────────────────

    #[tool(description = "List NAT rules (srcnat / dstnat chains)")]
    async fn list_firewall_nat(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/firewall/nat").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Add a NAT rule (masquerade, port-forward, etc.)")]
    async fn add_firewall_nat(
        &self,
        Parameters(p): Parameters<AddFirewallNatParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut body = json!({"chain": p.chain, "action": p.action});
        if let Some(v) = p.src_address   { body["src-address"]   = json!(v); }
        if let Some(v) = p.dst_address   { body["dst-address"]   = json!(v); }
        if let Some(v) = p.protocol      { body["protocol"]      = json!(v); }
        if let Some(v) = p.dst_port      { body["dst-port"]      = json!(v); }
        if let Some(v) = p.to_addresses  { body["to-addresses"]  = json!(v); }
        if let Some(v) = p.to_ports      { body["to-ports"]      = json!(v); }
        if let Some(v) = p.out_interface { body["out-interface"] = json!(v); }
        if let Some(v) = p.comment       { body["comment"]       = json!(v); }
        let data: Value = self.client.post("ip/firewall/nat", &body).await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a NAT rule by .id")]
    async fn remove_firewall_nat(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.client.delete("ip/firewall/nat", &p.id).await.map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── DHCP ──────────────────────────────────────────────────────────────────

    #[tool(description = "List configured DHCP servers and their interfaces / address pools")]
    async fn list_dhcp_servers(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/dhcp-server").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "List DHCP leases — both dynamic and static bindings")]
    async fn list_dhcp_leases(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/dhcp-server/lease").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Create a static DHCP lease: bind a MAC address to a fixed IP")]
    async fn add_dhcp_static_lease(
        &self,
        Parameters(p): Parameters<AddDhcpStaticLeaseParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut body = json!({"mac-address": p.mac_address, "address": p.address});
        if let Some(c) = p.comment { body["comment"] = json!(c); }
        let data: Value = self.client.post("ip/dhcp-server/lease", &body).await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a DHCP lease by .id")]
    async fn remove_dhcp_lease(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.client.delete("ip/dhcp-server/lease", &p.id).await.map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── DNS ───────────────────────────────────────────────────────────────────

    #[tool(description = "Get DNS settings: upstream servers, cache max TTL / size, DoH configuration")]
    async fn get_dns_settings(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/dns").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "List static DNS A records configured on the router")]
    async fn list_dns_static(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/dns/static").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Add a static DNS A record (useful for local .home.arpa hostnames)")]
    async fn add_dns_static(
        &self,
        Parameters(p): Parameters<AddDnsStaticParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut body = json!({"name": p.name, "address": p.address});
        if let Some(ttl) = p.ttl { body["ttl"] = json!(format!("{}s", ttl)); }
        if let Some(c)   = p.comment { body["comment"] = json!(c); }
        let data: Value = self.client.post("ip/dns/static", &body).await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }

    #[tool(description = "Remove a static DNS entry by .id")]
    async fn remove_dns_static(
        &self,
        Parameters(p): Parameters<RemoveByIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.client.delete("ip/dns/static", &p.id).await.map_err(tool_error)?;
        Ok(Self::ok_msg("removed"))
    }

    // ── Routes ────────────────────────────────────────────────────────────────

    #[tool(description = "List IP routing table entries including active routes, gateway, and distance")]
    async fn list_routes(&self) -> Result<CallToolResult, ErrorData> {
        let data: Value = self.client.get("ip/route").await.map_err(tool_error)?;
        Ok(Self::ok(&data))
    }
}

#[tool_handler]
impl ServerHandler for MikrotikServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "MikroTik RouterOS management via REST API (RouterOS 7.1+). \
                Configure with MIKROTIK_HOST, MIKROTIK_USER, MIKROTIK_PASSWORD env vars. \
                Optional: MIKROTIK_PORT (default 443), MIKROTIK_TLS_VERIFY (default false).",
            )
    }
}
