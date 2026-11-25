use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddLoggingRuleParams {
    #[schemars(
        description = "Comma-separated topics, '!' to exclude one (e.g. 'wireless,!debug', \
            'firewall', 'dhcp,info')"
    )]
    pub topics: String,
    #[schemars(
        description = "Logging action name from list_logging_actions (e.g. 'memory', 'disk', \
            'echo', 'remote')"
    )]
    pub action: String,
    #[schemars(description = "Text prepended to each message")]
    pub prefix: Option<String>,
    #[schemars(description = "Create the rule disabled (default: false)")]
    pub disabled: Option<bool>,
}
