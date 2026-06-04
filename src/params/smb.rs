use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetSmbParams {
    #[schemars(description = "Service state: 'yes', 'no', or 'auto'")]
    pub enabled: Option<String>,
    #[schemars(description = "Workgroup / domain name (e.g. 'WORKGROUP')")]
    pub domain: Option<String>,
    #[schemars(
        description = "Interface NAME to bind SMB to — e.g. 'bridge'. Use a LAN interface; \
            this is NOT an interface-list name ('LAN' is rejected). Binding to 'all' exposes \
            TCP 445 on the WAN/5G side — set this to a LAN interface to avoid that."
    )]
    pub interfaces: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddSmbShareParams {
    #[schemars(description = "Share name as it appears to SMB clients")]
    pub name: String,
    #[schemars(
        description = "Directory to share. For a USB partition this is the disk slot, e.g. 'usb1-part1'"
    )]
    pub directory: String,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
    #[schemars(description = "Create the share in disabled state (default: false)")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddSmbUserParams {
    #[schemars(description = "User name")]
    pub name: String,
    #[schemars(description = "Password")]
    pub password: String,
    #[schemars(
        description = "Read-only access (default: true). Set to false to grant read-write."
    )]
    pub read_only: Option<bool>,
    #[schemars(description = "Create the user in disabled state (default: false)")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetSmbUserParams {
    #[schemars(description = "User .id as returned by list_smb_users (e.g. '*1')")]
    pub id: String,
    #[schemars(description = "New password")]
    pub password: Option<String>,
    #[schemars(description = "Read-only access")]
    pub read_only: Option<bool>,
    #[schemars(description = "Disable the user")]
    pub disabled: Option<bool>,
}
