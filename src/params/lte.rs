use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLteInfoParams {
    #[schemars(description = "LTE interface name (default: 'lte1')")]
    pub interface: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LteInterfaceParams {
    #[schemars(description = "LTE interface name (default: 'lte1')")]
    pub interface: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetSimSlotParams {
    #[schemars(
        description = "SIM slot to make active, e.g. 'esim', 'sim', or a physical slot name \
            like 'up'/'down' on multi-slot devices. Maps to /interface lte settings set sim-slot="
    )]
    pub sim_slot: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProvisionEsimParams {
    #[schemars(description = "LTE interface holding the eSIM (default: 'lte1')")]
    pub interface: Option<String>,
    #[schemars(
        description = "SM-DP+ server hostname from the operator (the part after 'LPA:1$' in an \
            activation QR string), e.g. 'wbg.prod.ondemandconnectivity.com'"
    )]
    pub sm_dp_plus: String,
    #[schemars(
        description = "Activation code / matching ID — the token after the second '$' in an \
            'LPA:1$<sm-dp+>$<matching-id>' string"
    )]
    pub matching_id: String,
    #[schemars(
        description = "Optional confirmation code (one-time password) some operators require"
    )]
    pub confirmation_code: Option<String>,
    #[schemars(description = "Activate the profile immediately after download (default: true)")]
    pub activate: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LteAtChatParams {
    #[schemars(description = "LTE interface name (default: 'lte1')")]
    pub interface: Option<String>,
    #[schemars(
        description = "Raw AT command to send to the modem, e.g. 'AT+CSQ' (signal quality), \
            'AT+QENG=\"servingcell\"' (serving-cell / RSRP / band), or \
            'AT+QNWPREFCFG=\"mode_pref\"' (query the technology preference)"
    )]
    pub command: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetLteTechnologyParams {
    #[schemars(description = "LTE interface name (default: 'lte1')")]
    pub interface: Option<String>,
    #[schemars(
        description = "Radio access technology preference: 'lte' (4G only), 'nr5g' (5G only), \
            or 'auto' (LTE+5G). Forcing 'lte' can fix attach failures on 5G modems that wedge \
            during NR negotiation. Maps to the Quectel AT command \
            AT+QNWPREFCFG=\"mode_pref\",<MODE> via /interface/lte/at-chat."
    )]
    pub mode: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetLteApnProfileParams {
    #[schemars(
        description = "APN profile name (from list_lte_apn_profiles). Omit to target the \
            default profile (default=yes)."
    )]
    pub name: Option<String>,
    #[schemars(
        description = "Whether this APN installs a default route when LTE connects (yes/no). \
            Set 'no' (or use default_route_distance) to make LTE a backup WAN."
    )]
    pub add_default_route: Option<bool>,
    #[schemars(
        description = "Administrative distance for the LTE default route (1–255) — set to 2 \
            so LTE acts as a backup behind a distance-1 primary WAN"
    )]
    pub default_route_distance: Option<u8>,
    #[schemars(description = "APN (access point name) string, e.g. 'internet'")]
    pub apn: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EsimProfileNumberParams {
    #[schemars(
        description = "Profile selector as shown by list_esim_profiles — the row number/index \
            (e.g. '0') or its .id"
    )]
    pub number: String,
}
