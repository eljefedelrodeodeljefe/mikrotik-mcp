use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddDhcpStaticLeaseParams {
    #[schemars(description = "Client MAC address (e.g. 'AA:BB:CC:DD:EE:FF')")]
    pub mac_address: String,
    #[schemars(description = "IP address to assign to this MAC")]
    pub address: String,
    #[schemars(description = "Optional hostname / comment")]
    pub comment: Option<String>,
}
