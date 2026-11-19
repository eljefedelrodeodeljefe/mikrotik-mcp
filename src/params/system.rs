use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetSystemIdentityParams {
    #[schemars(description = "New device identity (hostname), e.g. 'r-atl-5g'")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveBackupParams {
    #[schemars(description = "Backup name without extension (e.g. 'r-ap-1-2026-05-16')")]
    pub name: String,
    #[schemars(
        description = "Absolute path on the local machine to write the .backup file. Required for the legacy FTP download; ignored when an SFTP export destination (MIKROTIK_BACKUP_SFTP_HOST) is configured, in which case the router pushes the file to that host instead."
    )]
    pub output_path: Option<String>,
    #[schemars(description = "Encryption password — defaults to MIKROTIK_PASSWORD when omitted")]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RestoreBackupParams {
    #[schemars(description = "Absolute path to the local .backup file to restore")]
    pub input_path: String,
    #[schemars(description = "Decryption password — defaults to MIKROTIK_PASSWORD when omitted")]
    pub password: Option<String>,
    #[schemars(
        description = "Must be true. The device reboots and its configuration is replaced: set \
            this only after the user has explicitly approved this restore"
    )]
    pub confirm: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLogsParams {
    #[schemars(description = "Number of most recent log entries to return (default: 50)")]
    pub count: Option<u32>,
    #[schemars(
        description = "Filter entries whose topics field contains this string (e.g. 'dhcp', 'firewall')"
    )]
    pub topics: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetNtpClientParams {
    #[schemars(description = "Client on or off: 'yes' or 'no'")]
    pub enabled: Option<String>,
    #[schemars(
        description = "Upstream servers, comma-separated host names or addresses \
            (e.g. '0.pool.ntp.org,1.pool.ntp.org')"
    )]
    pub servers: Option<String>,
    #[schemars(
        description = "Client mode: 'unicast' (default), 'broadcast', 'multicast' or 'manycast'"
    )]
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetNtpServerParams {
    #[schemars(
        description = "Serve time on UDP 123 to the clients the input firewall lets in: 'yes' or 'no'"
    )]
    pub enabled: Option<String>,
    #[schemars(description = "Also broadcast time to the LAN: 'yes' or 'no'")]
    pub broadcast: Option<String>,
}
