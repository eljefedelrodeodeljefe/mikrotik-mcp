use schemars::JsonSchema;
use serde::Deserialize;

use super::RouterOsId;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WifiMonitorParams {
    #[schemars(
        description = "WiFi interface name or .id (e.g. 'wifi1', or a CAPsMAN-managed 'wcap')"
    )]
    pub interface: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProvisionWifiRadioParams {
    #[schemars(
        description = "Radio .id as returned by list_wifi_radios (e.g. '*5'). A [find ...] \
            expression is not accepted over REST"
    )]
    pub id: RouterOsId,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetCapsmanChannelParams {
    #[schemars(
        description = "Channel profile .id as returned by list_capsman_channels (e.g. '*1')"
    )]
    pub id: RouterOsId,
    #[schemars(
        description = "Control channel frequency in MHz (e.g. '2412'). Empty string clears it \
            back to automatic selection"
    )]
    pub frequency: Option<String>,
    #[schemars(
        description = "Extension channel placement, e.g. 'disabled' (20 MHz), 'Ce', 'eC', 'Ceee'"
    )]
    pub extension_channel: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetCapsmanConfigurationParams {
    #[schemars(
        description = "Configuration profile .id as returned by list_capsman_configurations \
            (e.g. '*1'). Pass either id or name"
    )]
    pub id: Option<RouterOsId>,
    #[schemars(description = "Configuration profile name (e.g. 'cfg-5g'). Pass either id or name")]
    pub name: Option<String>,
    #[schemars(description = "SSID (service set identifier) broadcast by the AP")]
    pub ssid: Option<String>,
    #[schemars(description = "Channel profile name from /caps-man/channel")]
    pub channel: Option<String>,
    #[schemars(description = "Security profile name from /caps-man/security")]
    pub security: Option<String>,
    #[schemars(description = "Datapath profile name from /caps-man/datapath")]
    pub datapath: Option<String>,
    #[schemars(description = "Country name whose regulatory domain applies (e.g. 'germany')")]
    pub country: Option<String>,
    #[schemars(description = "Installation: 'any', 'indoor' or 'outdoor'")]
    pub installation: Option<String>,
    #[schemars(description = "Hide the SSID from beacons and probe responses (hide-ssid)")]
    pub hide_ssid: Option<bool>,
    #[schemars(description = "Comment on the profile")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetWifiConfigurationParams {
    #[schemars(
        description = "Configuration profile .id as returned by list_wifi_configurations \
            (e.g. '*1'). Pass either id or name"
    )]
    pub id: Option<RouterOsId>,
    #[schemars(description = "Configuration profile name (e.g. 'cfg-5g'). Pass either id or name")]
    pub name: Option<String>,
    #[schemars(description = "SSID (service set identifier) broadcast by the AP")]
    pub ssid: Option<String>,
    #[schemars(description = "Channel profile name from /interface/wifi/channel")]
    pub channel: Option<String>,
    #[schemars(description = "Security profile name from /interface/wifi/security")]
    pub security: Option<String>,
    #[schemars(description = "Datapath profile name from /interface/wifi/datapath")]
    pub datapath: Option<String>,
    #[schemars(description = "Country name whose regulatory domain applies (e.g. 'Germany')")]
    pub country: Option<String>,
    #[schemars(description = "Installation: 'indoor' or 'outdoor'")]
    pub installation: Option<String>,
    #[schemars(description = "Mode: 'ap', 'station', 'station-bridge' and the other wifi modes")]
    pub mode: Option<String>,
    #[schemars(description = "Hide the SSID from beacons and probe responses (hide-ssid)")]
    pub hide_ssid: Option<bool>,
    #[schemars(description = "Disable the profile")]
    pub disabled: Option<bool>,
    #[schemars(description = "Comment on the profile")]
    pub comment: Option<String>,
}
