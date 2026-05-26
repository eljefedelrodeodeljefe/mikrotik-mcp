use schemars::JsonSchema;
use serde::Deserialize;

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
    #[schemars(
        description = "Filter entries whose topics field contains this string (e.g. 'dhcp', 'firewall')"
    )]
    pub topics: Option<String>,
}
