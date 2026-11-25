use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PingParams {
    #[schemars(description = "Address or host name to ping (e.g. '192.0.2.1')")]
    pub address: String,
    #[schemars(description = "Number of echo requests, 1..20 (default: 4)")]
    pub count: Option<u32>,
    #[schemars(description = "Send from this interface")]
    pub interface: Option<String>,
    #[schemars(description = "Source address of the requests")]
    pub src_address: Option<String>,
    #[schemars(description = "Packet size in bytes (default: 56)")]
    pub size: Option<u32>,
    #[schemars(description = "Routing table to look the address up in (e.g. 'main')")]
    pub routing_table: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResolveDnsParams {
    #[schemars(description = "Name to resolve (e.g. 'router.example')")]
    pub domain_name: String,
    #[schemars(description = "DNS server to ask; omit to use the router's own resolver (/ip/dns)")]
    pub server: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchFileParams {
    #[schemars(description = "URL to download, http or https (e.g. \
            'http://192.0.2.10/routeros-7.20-arm.npk')")]
    pub url: String,
    #[schemars(description = "File name on the router, relative to Files (e.g. \
            'routeros-7.20-arm.npk')")]
    pub dst_path: String,
    #[schemars(
        description = "Verify the https server certificate: 'yes', 'yes-without-crl' or 'no' \
            (RouterOS default 'no')"
    )]
    pub check_certificate: Option<String>,
}
