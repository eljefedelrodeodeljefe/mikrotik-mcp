//! Server configuration, read from `MIKROTIK_*` environment variables.
//!
//! Parsing goes through a lookup function rather than `std::env` directly so
//! tests can feed values without touching the process environment.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::client::SftpTarget;

/// How the REST API is reached. `Http` sends the Basic auth header in
/// cleartext and must be chosen explicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

impl Scheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// Certificate checking for `https`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tls {
    /// Verify the router's certificate: against `ca_file` alone when set (a
    /// PEM CA, or the router's own certificate), otherwise against the
    /// built-in public roots.
    Verify { ca_file: Option<PathBuf> },
    /// Accept any certificate. Explicit opt-in only.
    Insecure,
}

pub struct Config {
    pub host: String,
    pub scheme: Scheme,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub tls: Tls,
    pub backup_encrypt: bool,
    pub writes_enabled: bool,
    pub backup_export: Option<SftpTarget>,
    /// Startup warnings to log once a subscriber is up (insecure or
    /// deprecated settings). Never contains secrets.
    pub warnings: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let mut warnings = Vec::new();
        let host = lookup("MIKROTIK_HOST").context("MIKROTIK_HOST not set")?;
        let explicit_scheme = match lookup("MIKROTIK_SCHEME") {
            None => None,
            Some(v) => match v.to_ascii_lowercase().as_str() {
                "https" => Some(Scheme::Https),
                "http" => Some(Scheme::Http),
                _ => bail!("MIKROTIK_SCHEME={v:?} is not a scheme: use https or http"),
            },
        };
        let port = parse_port_env("MIKROTIK_PORT", lookup("MIKROTIK_PORT"))?;
        if port == Some(80) && explicit_scheme.is_none() {
            bail!(
                "MIKROTIK_PORT=80 would send the RouterOS password in cleartext over http; \
                set MIKROTIK_SCHEME=http to accept that, or use the www-ssl port"
            );
        }
        let scheme = explicit_scheme.unwrap_or(Scheme::Https);
        let port = port.unwrap_or(match scheme {
            Scheme::Https => 443,
            Scheme::Http => 80,
        });
        let username = lookup("MIKROTIK_USER").unwrap_or_else(|| "admin".into());
        let password = lookup("MIKROTIK_PASSWORD").context("MIKROTIK_PASSWORD not set")?;
        let tls = tls_from_lookup(&lookup, scheme, &mut warnings)?;
        if scheme == Scheme::Http {
            warnings.push(
                "MIKROTIK_SCHEME=http: the RouterOS password is sent in cleartext on every request"
                    .into(),
            );
        }
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
            scheme,
            password,
            tls,
            backup_encrypt,
            writes_enabled,
            backup_export,
            warnings,
        })
    }
}

/// Resolves the TLS settings. Verification is the default; turning it off
/// takes `MIKROTIK_TLS_INSECURE=true` (or the deprecated
/// `MIKROTIK_TLS_VERIFY=false`), and contradictory settings are refused.
fn tls_from_lookup(
    lookup: &impl Fn(&str) -> Option<String>,
    scheme: Scheme,
    warnings: &mut Vec<String>,
) -> Result<Tls> {
    let ca_file = lookup("MIKROTIK_CA_FILE")
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from);
    let insecure = parse_bool_env("MIKROTIK_TLS_INSECURE", lookup("MIKROTIK_TLS_INSECURE"))?;
    let legacy_verify = parse_bool_env("MIKROTIK_TLS_VERIFY", lookup("MIKROTIK_TLS_VERIFY"))?;

    if let Some(verify) = legacy_verify {
        warnings.push(format!(
            "MIKROTIK_TLS_VERIFY is deprecated: {}",
            if verify {
                "verification is the default now; remove it"
            } else {
                "use MIKROTIK_TLS_INSECURE=true, or better MIKROTIK_CA_FILE with the router's certificate"
            }
        ));
        if insecure.is_some_and(|i| i == verify) {
            bail!(
                "MIKROTIK_TLS_INSECURE and the deprecated MIKROTIK_TLS_VERIFY contradict each other"
            );
        }
    }
    let insecure = insecure.or(legacy_verify.map(|v| !v)).unwrap_or(false);

    if scheme == Scheme::Http {
        if ca_file.is_some() {
            bail!("MIKROTIK_CA_FILE is set but MIKROTIK_SCHEME=http uses no TLS");
        }
        return Ok(Tls::Insecure);
    }
    if insecure {
        if ca_file.is_some() {
            bail!("MIKROTIK_CA_FILE and MIKROTIK_TLS_INSECURE=true contradict each other");
        }
        warnings.push(
            "TLS certificate verification is disabled: anyone on the path can impersonate the \
            router and read the RouterOS password. Set MIKROTIK_CA_FILE instead"
                .into(),
        );
        return Ok(Tls::Insecure);
    }
    Ok(Tls::Verify { ca_file })
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
        assert_eq!(
            config(&[("MIKROTIK_TLS_VERIFY", "False")]).unwrap().tls,
            Tls::Insecure
        );
    }

    #[test]
    fn verification_is_the_default() {
        let c = config(&[]).unwrap();
        assert_eq!(c.scheme, Scheme::Https);
        assert_eq!(c.port, 443);
        assert_eq!(c.tls, Tls::Verify { ca_file: None });
        assert!(c.warnings.is_empty(), "{:?}", c.warnings);
    }

    #[test]
    fn ca_file_verifies_against_that_file() {
        let c = config(&[("MIKROTIK_CA_FILE", "/etc/r.pem")]).unwrap();
        assert_eq!(
            c.tls,
            Tls::Verify {
                ca_file: Some("/etc/r.pem".into())
            }
        );
    }

    #[test]
    fn insecure_needs_the_explicit_opt_in_and_warns() {
        let c = config(&[("MIKROTIK_TLS_INSECURE", "true")]).unwrap();
        assert_eq!(c.tls, Tls::Insecure);
        assert!(
            c.warnings
                .iter()
                .any(|w| w.contains("verification is disabled"))
        );
    }

    #[test]
    fn legacy_tls_verify_false_still_works_and_warns_deprecated() {
        let c = config(&[("MIKROTIK_TLS_VERIFY", "false")]).unwrap();
        assert_eq!(c.tls, Tls::Insecure);
        assert!(c.warnings.iter().any(|w| w.contains("deprecated")));
        let c = config(&[("MIKROTIK_TLS_VERIFY", "true")]).unwrap();
        assert_eq!(c.tls, Tls::Verify { ca_file: None });
        assert!(c.warnings.iter().any(|w| w.contains("deprecated")));
    }

    #[test]
    fn contradictory_tls_settings_are_refused() {
        for vars in [
            &[
                ("MIKROTIK_TLS_INSECURE", "true"),
                ("MIKROTIK_TLS_VERIFY", "true"),
            ][..],
            &[
                ("MIKROTIK_TLS_INSECURE", "false"),
                ("MIKROTIK_TLS_VERIFY", "false"),
            ],
            &[
                ("MIKROTIK_TLS_INSECURE", "true"),
                ("MIKROTIK_CA_FILE", "/r.pem"),
            ],
            &[("MIKROTIK_SCHEME", "http"), ("MIKROTIK_CA_FILE", "/r.pem")],
        ] {
            assert!(config(vars).is_err(), "{vars:?} accepted");
        }
    }

    #[test]
    fn port_80_alone_is_refused() {
        let err = config(&[("MIKROTIK_PORT", "80")])
            .err()
            .expect("port 80 accepted");
        assert!(err.to_string().contains("MIKROTIK_SCHEME=http"), "{err}");
    }

    #[test]
    fn explicit_http_is_allowed_with_a_warning() {
        let c = config(&[("MIKROTIK_SCHEME", "http")]).unwrap();
        assert_eq!((c.scheme, c.port), (Scheme::Http, 80));
        assert!(c.warnings.iter().any(|w| w.contains("cleartext")));
        let c = config(&[("MIKROTIK_SCHEME", "HTTP"), ("MIKROTIK_PORT", "8080")]).unwrap();
        assert_eq!((c.scheme, c.port), (Scheme::Http, 8080));
        assert!(config(&[("MIKROTIK_SCHEME", "ftp")]).is_err());
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
