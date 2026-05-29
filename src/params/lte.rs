use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLteInfoParams {
    #[schemars(description = "LTE interface name (default: 'lte1')")]
    pub interface: Option<String>,
}
