use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetInterfaceParams {
    #[schemars(description = "Interface name (e.g. 'ether1', 'bridge')")]
    pub name: String,
}
