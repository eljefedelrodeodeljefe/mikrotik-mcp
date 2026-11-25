use schemars::JsonSchema;
use serde::Deserialize;

use super::RouterOsId;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetWirelessInterfaceParams {
    #[schemars(description = "Wireless interface name (e.g. 'wlan1')")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetWirelessInterfaceParams {
    #[schemars(
        description = "Interface .id as returned by list_wireless_interfaces (e.g. '*5'). \
            Pass either id or name"
    )]
    pub id: Option<RouterOsId>,
    #[schemars(description = "Interface name (e.g. 'wlan1'). Pass either id or name")]
    pub name: Option<String>,
    #[schemars(
        description = "Operating mode: 'station', 'station-bridge', 'station-pseudobridge', \
            'ap-bridge', 'bridge', 'wds-slave', 'alignment-only', 'nstreme-dual-slave'"
    )]
    pub mode: Option<String>,
    #[schemars(description = "SSID to broadcast (AP modes) or to connect to (station modes)")]
    pub ssid: Option<String>,
    #[schemars(description = "Security profile name from /interface/wireless/security-profiles")]
    pub security_profile: Option<String>,
    #[schemars(
        description = "Accept clients not in the access list (AP) or connect to APs not in \
            the connect list (station)"
    )]
    pub default_authentication: Option<bool>,
    #[schemars(description = "WPS mode: 'disabled', 'push-button', 'push-button-virtual-only'")]
    pub wps_mode: Option<String>,
    #[schemars(description = "Band, e.g. '2ghz-b/g/n', '5ghz-a/n/ac'")]
    pub band: Option<String>,
    #[schemars(description = "Channel frequency in MHz (e.g. '5180') or 'auto'")]
    pub frequency: Option<String>,
    #[schemars(description = "Channel width, e.g. '20mhz', '20/40mhz-Ce', '20/40/80mhz-Ceee'")]
    pub channel_width: Option<String>,
    #[schemars(description = "Country whose regulatory domain applies (e.g. 'germany')")]
    pub country: Option<String>,
    #[schemars(description = "Disable the interface")]
    pub disabled: Option<bool>,
    #[schemars(description = "Comment on the interface")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddWirelessSecurityProfileParams {
    #[schemars(description = "Profile name (e.g. 'example-wpa2')")]
    pub name: String,
    #[schemars(
        description = "Security mode: 'none', 'dynamic-keys', 'static-keys-required', \
            'static-keys-optional'. WPA/WPA2 needs 'dynamic-keys'"
    )]
    pub mode: Option<String>,
    #[schemars(description = "Comma-separated: 'wpa-psk', 'wpa2-psk', 'wpa-eap', 'wpa2-eap'")]
    pub authentication_types: Option<String>,
    #[schemars(description = "Comma-separated unicast ciphers: 'aes-ccm', 'tkip'")]
    pub unicast_ciphers: Option<String>,
    #[schemars(description = "Comma-separated group ciphers: 'aes-ccm', 'tkip'")]
    pub group_ciphers: Option<String>,
    #[schemars(description = "WPA pre-shared key, 8..63 characters. Write-only: never returned")]
    pub wpa_pre_shared_key: Option<String>,
    #[schemars(description = "WPA2 pre-shared key, 8..63 characters. Write-only: never returned")]
    pub wpa2_pre_shared_key: Option<String>,
    #[schemars(description = "Comment on the profile")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetWirelessSecurityProfileParams {
    #[schemars(
        description = "Profile .id as returned by list_wireless_security_profiles (e.g. '*1'). \
            Pass either id or name"
    )]
    pub id: Option<RouterOsId>,
    #[schemars(description = "Profile name (e.g. 'default'). Pass either id or name")]
    pub name: Option<String>,
    #[schemars(
        description = "Security mode: 'none', 'dynamic-keys', 'static-keys-required', \
            'static-keys-optional'"
    )]
    pub mode: Option<String>,
    #[schemars(description = "Comma-separated: 'wpa-psk', 'wpa2-psk', 'wpa-eap', 'wpa2-eap'")]
    pub authentication_types: Option<String>,
    #[schemars(description = "Comma-separated unicast ciphers: 'aes-ccm', 'tkip'")]
    pub unicast_ciphers: Option<String>,
    #[schemars(description = "Comma-separated group ciphers: 'aes-ccm', 'tkip'")]
    pub group_ciphers: Option<String>,
    #[schemars(description = "WPA pre-shared key, 8..63 characters. Write-only: never returned")]
    pub wpa_pre_shared_key: Option<String>,
    #[schemars(description = "WPA2 pre-shared key, 8..63 characters. Write-only: never returned")]
    pub wpa2_pre_shared_key: Option<String>,
    #[schemars(description = "Comment on the profile")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddWirelessConnectListParams {
    #[schemars(description = "Station interface the rule applies to (e.g. 'wlan1')")]
    pub interface: String,
    #[schemars(description = "Match this SSID; omit to match any")]
    pub ssid: Option<String>,
    #[schemars(description = "Match this AP MAC address (e.g. '02:00:00:00:00:01')")]
    pub mac_address: Option<String>,
    #[schemars(description = "Security profile to use with a matching AP")]
    pub security_profile: Option<String>,
    #[schemars(description = "Connect to a matching AP (true) or never connect to it (false)")]
    pub connect: Option<bool>,
    #[schemars(description = "Signal strength range to match, e.g. '-75..120'")]
    pub signal_range: Option<String>,
    #[schemars(
        description = "Insert before this rule .id; rules are matched top to bottom. Omit to \
            append"
    )]
    pub place_before: Option<RouterOsId>,
    #[schemars(description = "Comment on the rule")]
    pub comment: Option<String>,
}
