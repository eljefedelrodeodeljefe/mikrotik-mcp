use schemars::JsonSchema;
use serde::Deserialize;

use super::RouterOsId;

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetBridgePortParams {
    #[schemars(description = "Bridge port .id as returned by list_bridge_ports (e.g. '*4')")]
    pub id: RouterOsId,
    #[schemars(
        description = "Bridge to move the port into (e.g. 'bridge', 'bridge-ci'). The port \
            leaves its current bridge"
    )]
    pub bridge: Option<String>,
    #[schemars(description = "Comment for the bridge port entry")]
    pub comment: Option<String>,
}
