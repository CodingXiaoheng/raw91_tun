use anyhow::{Context, Result};
use std::io::{Read, Write};
use tun::Configuration;
// 关键：引入 trait 才能用 tun_name()
use tun::AbstractDevice as _;

pub struct Tun {
    pub dev: tun::Device,
    pub name: String,
}

impl Tun {
    pub fn open(
        name_hint: Option<&str>,
        mtu: u16,
        v4_addr: Option<std::net::Ipv4Addr>,
        v4_peer: Option<std::net::Ipv4Addr>,
        v4_mask: Option<std::net::Ipv4Addr>,
    ) -> Result<Tun> {
        let mut cfg = Configuration::default();
        cfg.mtu(mtu);
        cfg.up();
        if let Some(h) = name_hint {
            cfg.tun_name(h); // name() 已废弃，推荐 tun_name()
        }
        if let Some(addr) = v4_addr { cfg.address(addr); }
        if let Some(dst)  = v4_peer { cfg.destination(dst); }
        if let Some(msk)  = v4_mask { cfg.netmask(msk); }

        let dev = tun::create(&cfg).context("create TUN device")?;
        let name = dev.tun_name()?; // 来自 AbstractDevice
        Ok(Tun { dev, name })
    }
}

// 仍然实现 Read/Write 便于直连使用
impl Read for Tun {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.dev.read(buf) }
}
impl Write for Tun {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.dev.write(buf) }
    fn flush(&mut self) -> std::io::Result<()> { self.dev.flush() }
}
