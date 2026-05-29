use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetInterfaceParams {
    #[schemars(description = "Interface name (e.g. 'ether1', 'bridge')")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InterfaceNameParams {
    #[schemars(description = "Interface name or .id (e.g. 'pppoe-out1', 'ether1', '*10')")]
    pub interface: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddInterfaceListMemberParams {
    #[schemars(description = "Interface list name (e.g. 'WAN', 'LAN')")]
    pub list: String,
    #[schemars(description = "Interface to add to the list")]
    pub interface: String,
}
