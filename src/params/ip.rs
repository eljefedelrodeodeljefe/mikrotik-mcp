use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddIpAddressParams {
    #[schemars(description = "IP address with prefix length (e.g. '192.168.1.1/24')")]
    pub address: String,
    #[schemars(description = "Interface to assign the address to (e.g. 'ether1', 'bridge')")]
    pub interface: String,
    #[schemars(description = "Optional comment")]
    pub comment: Option<String>,
}
