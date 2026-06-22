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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddBridgePortParams {
    #[schemars(description = "Bridge to add the port to (e.g. 'bridge')")]
    pub bridge: String,
    #[schemars(
        description = "Interface to add as a bridge port (e.g. 'spine-guest', 'ether5'). \
            Required for a virtual AP to pass LAN traffic."
    )]
    pub interface: String,
    #[schemars(
        description = "Optional untagged VLAN id for this port (pvid). Defaults to the \
            bridge default (usually 1) when omitted."
    )]
    pub pvid: Option<u16>,
    #[schemars(description = "Optional comment stored on the bridge port")]
    pub comment: Option<String>,
}
