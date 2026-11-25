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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetClockParams {
    #[schemars(
        description = "Date in the format get_clock shows: 'YYYY-MM-DD' on RouterOS 7.10+ \
            (e.g. '2026-01-31'), 'mmm/DD/YYYY' before (e.g. 'jan/31/2026')"
    )]
    pub date: Option<String>,
    #[schemars(description = "Time of day, 'HH:MM:SS' (e.g. '13:05:00')")]
    pub time: Option<String>,
    #[schemars(description = "Time zone, e.g. 'Europe/Berlin', 'UTC' or 'manual'")]
    pub time_zone_name: Option<String>,
    #[schemars(
        description = "Pick the time zone from the router's public IP (time-zone-autodetect). \
            Turn it off before setting time-zone-name, or the router overrides it"
    )]
    pub time_zone_autodetect: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfirmParams {
    #[schemars(
        description = "Must be true. The device reboots or its boot loader is rewritten: set \
            this only after the user has explicitly approved this action"
    )]
    pub confirm: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPackageUpdateParams {
    #[schemars(
        description = "Run check-for-updates first (about 15 s), which asks the MikroTik \
            download server for the latest version on the update channel (default: false, only \
            read the last result)"
    )]
    pub check_for_updates: Option<bool>,
}
