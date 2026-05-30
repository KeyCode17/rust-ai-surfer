//! Egress / SSRF policy engine — pure Rust, no browser dependency.

use std::net::{Ipv4Addr, Ipv6Addr};

use ras_types::DomainPattern;
use url::{Host, Url};

/// Errors produced by [`EgressPolicy::check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressError {
    /// The URL scheme is not in the allowed-schemes list.
    ForbiddenScheme(String),
    /// The host resolves to a private / loopback / link-local address.
    PrivateAddress(String),
    /// The host is explicitly denied (deny-list match or metadata endpoint).
    DeniedHost(String),
    /// The port is in the denied-ports list.
    DeniedPort(u16),
    /// An allow-list is configured and the host is not on it.
    NotAllowlisted(String),
    /// The URL has no host component.
    NoHost,
}

/// Always-on egress policy. [`EgressPolicy::check`] returns `Ok(())` only for permitted URLs.
pub struct EgressPolicy {
    allowed_schemes: Vec<String>,
    block_private: bool,
    denied_ports: Vec<u16>,
    deny: Vec<DomainPattern>,
    allow: Vec<DomainPattern>,
}

impl Default for EgressPolicy {
    fn default() -> Self {
        Self {
            allowed_schemes: vec!["http".into(), "https".into()],
            block_private: true,
            denied_ports: vec![],
            deny: vec![],
            allow: vec![],
        }
    }
}

impl EgressPolicy {
    /// Replace the allow-list with the supplied patterns.
    #[must_use]
    pub fn with_allow(mut self, patterns: Vec<DomainPattern>) -> Self {
        self.allow = patterns;
        self
    }

    /// Replace the deny-list with the supplied patterns.
    #[must_use]
    pub fn with_deny(mut self, patterns: Vec<DomainPattern>) -> Self {
        self.deny = patterns;
        self
    }

    /// Replace the denied-ports list.
    #[must_use]
    pub fn with_denied_ports(mut self, ports: Vec<u16>) -> Self {
        self.denied_ports = ports;
        self
    }

    /// Toggle private-address blocking (`true` = allow private addresses through).
    #[must_use]
    pub fn allow_private(mut self, allow: bool) -> Self {
        self.block_private = !allow;
        self
    }

    /// Validate `url` against this policy.
    ///
    /// Steps (in order):
    /// 1. Scheme allow-list
    /// 2. Host presence
    /// 3. Private-address / metadata block
    /// 4. Denied-ports block
    /// 5. Deny-list patterns
    /// 6. Allow-list patterns (if non-empty)
    pub fn check(&self, url: &Url) -> Result<(), EgressError> {
        if !self.allowed_schemes.iter().any(|s| s == url.scheme()) {
            return Err(EgressError::ForbiddenScheme(url.scheme().into()));
        }

        let host = match url.host() {
            None => return Err(EgressError::NoHost),
            Some(h) => h,
        };

        if self.block_private {
            match &host {
                Host::Ipv4(ip) => {
                    if is_forbidden_v4(*ip) {
                        return Err(EgressError::PrivateAddress(ip.to_string()));
                    }
                }
                Host::Ipv6(ip) => {
                    if is_forbidden_v6(*ip) {
                        return Err(EgressError::PrivateAddress(ip.to_string()));
                    }
                }
                Host::Domain(d) => {
                    let lower = d.to_lowercase();
                    if lower == "localhost" || lower.ends_with(".localhost") {
                        return Err(EgressError::PrivateAddress(lower));
                    }
                    if lower == "metadata.google.internal" {
                        return Err(EgressError::DeniedHost(lower));
                    }
                }
            }
        }

        if let Some(port) = url.port_or_known_default()
            && self.denied_ports.contains(&port)
        {
            return Err(EgressError::DeniedPort(port));
        }

        let host_str = url.host_str().map(str::to_string).unwrap_or_default();

        if self.deny.iter().any(|pat| pat.matches_url(url)) {
            return Err(EgressError::DeniedHost(host_str.clone()));
        }

        if !self.allow.is_empty() && !self.allow.iter().any(|pat| pat.matches_url(url)) {
            return Err(EgressError::NotAllowlisted(host_str));
        }

        Ok(())
    }
}

fn is_forbidden_v4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
}

fn is_forbidden_v6(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() {
        return true;
    }
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_forbidden_v4(v4);
    }
    let seg0 = ip.segments()[0];
    let unique_local = (seg0 & 0xfe00) == 0xfc00;
    let link_local = (seg0 & 0xffc0) == 0xfe80;
    unique_local || link_local
}

#[cfg(test)]
#[path = "egress_tests.rs"]
mod tests;
