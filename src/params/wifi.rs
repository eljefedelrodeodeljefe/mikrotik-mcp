use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetWifiInterfaceParams {
    #[schemars(description = "Name of the wifi interface to modify (e.g. 'wifi1', 'wifi3')")]
    pub name: String,
    #[schemars(
        description = "New mode: 'ap' (access point) or 'station' (connect to upstream AP). \
            Changing to 'station' on a master interface converts it from AP to WAN uplink — \
            remove it from the bridge first with remove_bridge_port."
    )]
    pub mode: Option<String>,
    #[schemars(description = "New SSID — for AP mode the network name to advertise; \
        for station mode the SSID to connect to")]
    pub ssid: Option<String>,
    #[schemars(description = "Name of a /interface/wifi/security profile to apply")]
    pub security: Option<String>,
    #[schemars(
        description = "Master interface for VIF mode. Set to empty string to make \
        this a standalone (non-VIF) interface."
    )]
    pub master_interface: Option<String>,
    #[schemars(
        description = "WPA2 passphrase to set directly on the interface. When provided, \
            overrides any existing inline passphrase and disables Fast Transition (ft), \
            which is AP-only and breaks station-mode connections."
    )]
    pub passphrase: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WifiNameParams {
    #[schemars(description = "Name of the wifi interface (e.g. 'wifi3')")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScanWifiParams {
    #[schemars(
        description = "Interface to scan from (e.g. 'wifi1' for 5 GHz, 'wifi2' for 2.4 GHz)"
    )]
    pub interface: String,
    #[schemars(
        description = "Scan duration in seconds (default 5). Longer gives more complete results."
    )]
    pub duration: Option<u8>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddWifiSecurityParams {
    #[schemars(description = "Name for this security profile (e.g. 'bde-hotspot')")]
    pub name: String,
    #[schemars(description = "WPA2/WPA3 passphrase for the network")]
    pub passphrase: String,
    #[schemars(
        description = "Authentication types — defaults to 'wpa2-psk' if omitted. \
            Use 'wpa2-psk,wpa3-psk' for mixed WPA2/3."
    )]
    pub authentication_types: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddWifiStationParams {
    #[schemars(
        description = "Name for the new virtual station interface, e.g. 'wifi3'. \
            Must be unique across all interfaces."
    )]
    pub name: String,
    #[schemars(
        description = "Master (radio) interface to create this station VIF on top of \
            (e.g. 'wifi1' for 2.4 GHz, 'wifi2' for 5 GHz). The master continues to \
            operate as an AP while this VIF connects upstream."
    )]
    pub master_interface: String,
    #[schemars(description = "SSID of the upstream network to connect to")]
    pub ssid: String,
    #[schemars(description = "Name of a /interface/wifi/security profile to use for \
            authentication. Create one first with add_wifi_security.")]
    pub security: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddWifiApParams {
    #[schemars(
        description = "Name for the new virtual AP interface, e.g. 'spine-guest'. \
            Must be unique across all interfaces."
    )]
    pub name: String,
    #[schemars(
        description = "Master (radio) interface to broadcast this extra SSID on \
            (e.g. 'wifi2'). The master keeps its own SSID; this VIF adds a second \
            BSSID on the same radio. Add it to the LAN bridge with add_bridge_port \
            for the new SSID to pass traffic."
    )]
    pub master_interface: String,
    #[schemars(description = "SSID (network name) this AP will advertise")]
    pub ssid: String,
    #[schemars(
        description = "Name of a /interface/wifi/security profile to apply. Reference \
            the master's profile to make this a true alias (edit once, both update); \
            omit for an open network."
    )]
    pub security: Option<String>,
    #[schemars(description = "Optional comment stored on the interface")]
    pub comment: Option<String>,
}
