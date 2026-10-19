use schemars::JsonSchema;
use serde::Deserialize;

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
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetCapsmanChannelParams {
    #[schemars(
        description = "Channel profile .id as returned by list_capsman_channels (e.g. '*1')"
    )]
    pub id: String,
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
