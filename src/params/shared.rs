use std::borrow::Cow;
use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

/// A RouterOS item `.id`: `*` followed by hex digits, e.g. `*1A` or
/// `*80000001`.
///
/// Ids end up in the request path (`DELETE /rest/<menu>/<id>`), and the URL
/// parser resolves `..` segments, so an unchecked id like
/// `1/../../../firewall/filter/*5` would address a different menu. Parsing
/// on deserialization means a tool can only ever be handed a well-formed id.
/// The leading `*` is optional on input and always present once parsed.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct RouterOsId(String);

impl RouterOsId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RouterOsId {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let hex = raw.strip_prefix('*').unwrap_or(&raw);
        if !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(format!("*{hex}")))
        } else {
            Err(format!(
                "invalid RouterOS .id {raw:?}: expected '*' followed by hex digits, e.g. '*1A'"
            ))
        }
    }
}

impl TryFrom<&str> for RouterOsId {
    type Error = String;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::try_from(raw.to_string())
    }
}

impl fmt::Display for RouterOsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for RouterOsId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl JsonSchema for RouterOsId {
    fn schema_name() -> Cow<'static, str> {
        "RouterOsId".into()
    }

    fn inline_schema() -> bool {
        true
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^\*?[0-9A-Fa-f]+$",
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveByIdParams {
    #[schemars(description = "Item .id as returned by the list command (e.g. '*1')")]
    pub id: RouterOsId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_star_hex_and_adds_missing_star() {
        for (input, want) in [
            ("*1", "*1"),
            ("*1A", "*1A"),
            ("1a", "*1a"),
            ("*80000001", "*80000001"),
        ] {
            let id: RouterOsId = serde_json::from_value(json!(input)).unwrap();
            assert_eq!(id.as_str(), want);
        }
    }

    #[test]
    fn refuses_anything_but_hex() {
        for input in [
            "",
            "*",
            "1/../../../firewall/filter/*5",
            "*1/..",
            "*1?x=y",
            "*1#",
            "ether1",
            "*1 ",
            "%2e%2e",
        ] {
            let res: Result<RemoveByIdParams, _> = serde_json::from_value(json!({"id": input}));
            assert!(res.is_err(), "{input:?} was accepted");
        }
    }
}
