use schemars::JsonSchema;
use serde::Deserialize;

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
