//! Server configuration, read from `MIKROTIK_*` environment variables.
//!
//! Parsing goes through a lookup function rather than `std::env` directly so
//! tests can feed values without touching the process environment.

use anyhow::{Context, Result, bail};

use crate::client::SftpTarget;

pub struct Config {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub tls_verify: bool,
    pub backup_encrypt: bool,
    pub writes_enabled: bool,
    pub backup_export: Option<SftpTarget>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let host = lookup("MIKROTIK_HOST").context("MIKROTIK_HOST not set")?;
        let port = parse_port_env("MIKROTIK_PORT", lookup("MIKROTIK_PORT"))?.unwrap_or(443);
        let username = lookup("MIKROTIK_USER").unwrap_or_else(|| "admin".into());
        let password = lookup("MIKROTIK_PASSWORD").context("MIKROTIK_PASSWORD not set")?;
        let tls_verify =
            parse_bool_env("MIKROTIK_TLS_VERIFY", lookup("MIKROTIK_TLS_VERIFY"))?.unwrap_or(false);
        let backup_encrypt =
            parse_bool_env("MIKROTIK_BACKUP_ENCRYPT", lookup("MIKROTIK_BACKUP_ENCRYPT"))?
                .unwrap_or(true);
        let writes_enabled =
            parse_bool_env("MIKROTIK_ALLOW_WRITES", lookup("MIKROTIK_ALLOW_WRITES"))?
                .unwrap_or(false);

        // Optional headless backup export: when a destination host is set, the
        // router pushes each backup out over SFTP (egress only), which works
        // even on a hardened device with inbound FTP disabled.
        let backup_export = match lookup("MIKROTIK_BACKUP_SFTP_HOST") {
            Some(h) if !h.trim().is_empty() => Some(SftpTarget {
                host: h,
                port: parse_port_env(
                    "MIKROTIK_BACKUP_SFTP_PORT",
                    lookup("MIKROTIK_BACKUP_SFTP_PORT"),
                )?
                .unwrap_or(22),
                user: lookup("MIKROTIK_BACKUP_SFTP_USER").context(
                    "MIKROTIK_BACKUP_SFTP_USER not set (required with MIKROTIK_BACKUP_SFTP_HOST)",
                )?,
                password: lookup("MIKROTIK_BACKUP_SFTP_PASSWORD").context(
                    "MIKROTIK_BACKUP_SFTP_PASSWORD not set (required with MIKROTIK_BACKUP_SFTP_HOST)",
                )?,
                path: lookup("MIKROTIK_BACKUP_SFTP_PATH").unwrap_or_default(),
            }),
            _ => None,
        };

        Ok(Self {
            host,
            port,
            username,
            password,
            tls_verify,
            backup_encrypt,
            writes_enabled,
            backup_export,
        })
    }
}

/// Parses a boolean variable: `true`/`false`, `1`/`0` or `yes`/`no`, in any
/// case. Anything else is an error naming the variable, so a typo can't
/// silently flip a safety setting. `None` (unset) stays `None`.
pub fn parse_bool_env(name: &str, value: Option<String>) -> Result<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(Some(true)),
        "false" | "0" | "no" => Ok(Some(false)),
        _ => bail!("{name}={value:?} is not a boolean: use true/false, 1/0 or yes/no"),
    }
}

/// Parses a TCP port variable. `None` (unset) stays `None`; a value that is
/// not a port number is an error naming the variable.
pub fn parse_port_env(name: &str, value: Option<String>) -> Result<Option<u16>> {
    value
        .map(|v| {
            v.parse::<u16>()
                .ok()
                .filter(|p| *p != 0)
                .with_context(|| format!("{name}={v:?} is not a valid port number"))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn config(vars: &[(&str, &str)]) -> Result<Config> {
        let mut env: HashMap<String, String> =
            [("MIKROTIK_HOST", "r"), ("MIKROTIK_PASSWORD", "pw")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
        for (k, v) in vars {
            env.insert(k.to_string(), v.to_string());
        }
        Config::from_lookup(|name| env.get(name).cloned())
    }

    #[test]
    fn bool_env_accepts_the_allowlist_in_any_case() {
        for v in ["true", "TRUE", "True", "1", "yes", "YES"] {
            assert_eq!(
                parse_bool_env("X", Some(v.into())).unwrap(),
                Some(true),
                "{v}"
            );
        }
        for v in ["false", "FALSE", "False", "0", "no", "No"] {
            assert_eq!(
                parse_bool_env("X", Some(v.into())).unwrap(),
                Some(false),
                "{v}"
            );
        }
        assert_eq!(parse_bool_env("X", None).unwrap(), None);
    }

    #[test]
    fn bool_env_refuses_anything_else_and_names_the_variable() {
        for v in ["", "on", "off", "enabled", "y", "2", " true"] {
            let err = parse_bool_env("MIKROTIK_ALLOW_WRITES", Some(v.into())).unwrap_err();
            assert!(
                err.to_string().contains("MIKROTIK_ALLOW_WRITES"),
                "{v:?}: {err}"
            );
        }
    }

    #[test]
    fn allow_writes_upper_case_enables_writes() {
        assert!(
            config(&[("MIKROTIK_ALLOW_WRITES", "TRUE")])
                .unwrap()
                .writes_enabled
        );
        assert!(!config(&[]).unwrap().writes_enabled);
    }

    #[test]
    fn tls_verify_mixed_case_false_disables_verification() {
        assert!(
            !config(&[("MIKROTIK_TLS_VERIFY", "False")])
                .unwrap()
                .tls_verify
        );
    }

    #[test]
    fn backup_encrypt_typo_is_a_startup_error() {
        assert!(config(&[("MIKROTIK_BACKUP_ENCRYPT", "nope")]).is_err());
        assert!(
            !config(&[("MIKROTIK_BACKUP_ENCRYPT", "False")])
                .unwrap()
                .backup_encrypt
        );
    }

    #[test]
    fn bad_sftp_port_is_a_startup_error() {
        let sftp = [
            ("MIKROTIK_BACKUP_SFTP_HOST", "nas"),
            ("MIKROTIK_BACKUP_SFTP_USER", "u"),
            ("MIKROTIK_BACKUP_SFTP_PASSWORD", "p"),
        ];
        let mut vars = sftp.to_vec();
        vars.push(("MIKROTIK_BACKUP_SFTP_PORT", "22a"));
        let err = config(&vars).err().expect("bad port accepted");
        assert!(
            err.to_string().contains("MIKROTIK_BACKUP_SFTP_PORT"),
            "{err}"
        );

        let target = config(&sftp).unwrap().backup_export.unwrap();
        assert_eq!(target.port, 22);
    }
}
