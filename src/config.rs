use serde::Deserialize;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, Deserialize)]
pub struct FileConfig {
    pub tun_name: Option<String>,
    pub mtu: Option<u16>,

    // TUN IPv4
    pub tun_v4_addr: Option<Ipv4Addr>,
    pub tun_v4_peer: Option<Ipv4Addr>,
    pub tun_v4_mask: Option<Ipv4Addr>,

    // TUN IPv6 (optional)
    pub tun_v6_addr: Option<Ipv6Addr>,
    pub tun_v6_peer: Option<Ipv6Addr>,

    // Raw socket side
    pub raw_remote_v4: Option<Ipv4Addr>,
    pub raw_bind_v4: Option<Ipv4Addr>,
    pub ip_protocol: Option<u8>, // custom IP protocol number (e.g. 222)

    // MTU handling
    pub outer_mtu: Option<u16>,      // estimated path MTU for outer IPv4
    pub drop_if_exceeds: Option<bool>,

    // Logging
    pub log_level: Option<String>, // e.g., "info", "debug"

    // Optional commands to run after tun up/down
    pub post_up: Option<Vec<String>>,  // shell commands
    pub post_down: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub tun_name: Option<String>,
    pub mtu: u16,

    pub tun_v4_addr: Option<Ipv4Addr>,
    pub tun_v4_peer: Option<Ipv4Addr>,
    pub tun_v4_mask: Option<Ipv4Addr>,
    pub tun_v6_addr: Option<Ipv6Addr>,
    pub tun_v6_peer: Option<Ipv6Addr>,

    pub raw_remote_v4: Ipv4Addr,
    pub raw_bind_v4: Option<Ipv4Addr>,
    pub ip_protocol: u8,

    pub outer_mtu: u16,
    pub drop_if_exceeds: bool,

    pub log_level: String,

    pub post_up: Vec<String>,
    pub post_down: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tun_name: None,
            mtu: 1300, // leave headroom for base91 expansion + outer IPv4
            tun_v4_addr: None,
            tun_v4_peer: None,
            tun_v4_mask: None,
            tun_v6_addr: None,
            tun_v6_peer: None,
            raw_remote_v4: Ipv4Addr::new(127, 0, 0, 1),
            raw_bind_v4: None,
            ip_protocol: 222,
            outer_mtu: 1500,
            drop_if_exceeds: true,
            log_level: "info".into(),
            post_up: vec![],
            post_down: vec![],
        }
    }
}

impl Config {
    pub fn merge(file: Option<FileConfig>, cli: CliOverrides) -> anyhow::Result<Self> {
        let mut c = Config::default();
        if let Some(f) = file {
            if let Some(v) = f.tun_name { c.tun_name = Some(v); }
            if let Some(v) = f.mtu { c.mtu = v; }
            if let Some(v) = f.tun_v4_addr { c.tun_v4_addr = Some(v); }
            if let Some(v) = f.tun_v4_peer { c.tun_v4_peer = Some(v); }
            if let Some(v) = f.tun_v4_mask { c.tun_v4_mask = Some(v); }
            if let Some(v) = f.tun_v6_addr { c.tun_v6_addr = Some(v); }
            if let Some(v) = f.tun_v6_peer { c.tun_v6_peer = Some(v); }
            if let Some(v) = f.raw_remote_v4 { c.raw_remote_v4 = v; }
            if let Some(v) = f.raw_bind_v4 { c.raw_bind_v4 = Some(v); }
            if let Some(v) = f.ip_protocol { c.ip_protocol = v; }
            if let Some(v) = f.outer_mtu { c.outer_mtu = v; }
            if let Some(v) = f.drop_if_exceeds { c.drop_if_exceeds = v; }
            if let Some(v) = f.log_level { c.log_level = v; }
            if let Some(v) = f.post_up { c.post_up = v; }
            if let Some(v) = f.post_down { c.post_down = v; }
        }
        // Apply CLI overrides last
        if let Some(v) = cli.tun_name { c.tun_name = Some(v); }
        if let Some(v) = cli.mtu { c.mtu = v; }
        if let Some(v) = cli.raw_remote_v4 { c.raw_remote_v4 = v; }
        if let Some(v) = cli.raw_bind_v4 { c.raw_bind_v4 = Some(v); }
        if let Some(v) = cli.ip_protocol { c.ip_protocol = v; }
        if let Some(v) = cli.outer_mtu { c.outer_mtu = v; }
        if let Some(v) = cli.drop_if_exceeds { c.drop_if_exceeds = v; }
        if let Some(v) = cli.log_level { c.log_level = v; }
        Ok(c)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CliOverrides {
    pub tun_name: Option<String>,
    pub mtu: Option<u16>,
    pub raw_remote_v4: Option<Ipv4Addr>,
    pub raw_bind_v4: Option<Ipv4Addr>,
    pub ip_protocol: Option<u8>,
    pub outer_mtu: Option<u16>,
    pub drop_if_exceeds: Option<bool>,
    pub log_level: Option<String>,
}