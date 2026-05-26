use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveByIdParams {
    #[schemars(description = "Item .id as returned by the list command (e.g. '*1')")]
    pub id: String,
}
