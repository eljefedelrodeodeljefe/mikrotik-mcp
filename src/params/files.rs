use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveFileParams {
    #[schemars(description = "Name of the file to remove as shown by list_files \
            (e.g. 'chateau-2026-06-22.backup'). Useful for cleaning up backup \
            files left on the device after save_backup downloads them.")]
    pub name: String,
}
