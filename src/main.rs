mod config;
mod tun_dev;
mod rawsock;
mod base91codec;

use anyhow::{bail, Context, Result};
use clap::Parser;
use config::{CliOverrides, Config, FileConfig};
use log::{debug, error, info, warn};
use std::fs;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

#[derive(Parser, Debug)]
#[command(name = "raw91-tun", version, about = "P2P TUN tunnel over IPv4 RawSocket with base91 encoding")]
struct Cli {
    /// Path to TOML config
    #[arg(short = 'c', long = "config")]
    config: Option<PathBuf>,

    /// Override: remote IPv4 that receives raw packets
    #[arg(long = "peer", value_parser)]
    raw_remote_v4: Option<Ipv4Addr>,

    /// Override: bind local IPv4
    #[arg(long = "bind", value_parser)]
    raw_bind_v4: Option<Ipv4Addr>,

    /// Override: custom IP protocol number (0-255)
    #[arg(long = "proto")]
    ip_protocol: Option<u8>,

    /// Override: TUN interface name
    #[arg(long = "tun")]
    tun_name: Option<String>,

    /// Override: TUN MTU
    #[arg(long = "mtu")]
    mtu: Option<u16>,

    /// Outer path MTU (for raw IPv4). Used to pre-drop oversize encoded payloads.
    #[arg(long = "outer-mtu")]
    outer_mtu: Option<u16>,

    /// Drop packets if encoded size exceeds outer MTU (default true)
    #[arg(long = "drop-if-exceeds")]
    drop_if_exceeds: Option<bool>,

    /// Log level (error|warn|info|debug|trace)
    #[arg(long = "log-level")]
    log_level: Option<String>,
}

fn init_logger(level: &str) {
    std::env::set_var("RUST_LOG", level);
    let _ = env_logger::try_init();
}

fn read_toml(path: &PathBuf) -> Result<FileConfig> {
    let s = fs::read_to_string(path).with_context(|| format!("read config {}", path.display()))?;
    let cfg: FileConfig = toml::from_str(&s).context("parse TOML")?;
    Ok(cfg)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Pre-init logger with a default; will re-init after merging config
    init_logger("info");

    let file_cfg = match &cli.config { Some(p) => Some(read_toml(p)?), None => None };

    let overrides = CliOverrides {
        tun_name: cli.tun_name.clone(),
        mtu: cli.mtu,
        raw_remote_v4: cli.raw_remote_v4,
        raw_bind_v4: cli.raw_bind_v4,
        ip_protocol: cli.ip_protocol,
        outer_mtu: cli.outer_mtu,
        drop_if_exceeds: cli.drop_if_exceeds,
        log_level: cli.log_level.clone(),
    };

    let cfg = Config::merge(file_cfg, overrides)?;
    init_logger(&cfg.log_level);
    info!("starting raw91-tun with proto {} -> {}", cfg.ip_protocol, cfg.raw_remote_v4);

// Open TUN
let mut tun = tun_dev::Tun::open(
    cfg.tun_name.as_deref(),
    cfg.mtu,
    cfg.tun_v4_addr,
    cfg.tun_v4_peer,
    cfg.tun_v4_mask,
)?;
info!("TUN up: {} (mtu={})", tun.name, cfg.mtu);

// 拆分读写端：Reader 给 TX 线程使用（读 TUN -> 发送 raw），Writer 留给 RX（写回 TUN）
let (mut tun_reader, mut tun_writer) = tun.dev.split();

// Raw socket
let raw = rawsock::RawSock::open(cfg.ip_protocol, cfg.raw_bind_v4, cfg.raw_remote_v4)?;
let raw2 = raw.try_clone()?;

info!("raw socket ready -> {} proto {}", cfg.raw_remote_v4, cfg.ip_protocol);

let running = Arc::new(AtomicBool::new(true));
// TX: TUN -> base91 -> RAW
let running_tx = running.clone();
let cfg_tx = cfg.clone();
let tx = std::thread::spawn(move || -> Result<()> {
    let mut buf = vec![0u8; (cfg_tx.mtu as usize) + 64];
    while running_tx.load(Ordering::SeqCst) {
        let n = match tun_reader.read(&mut buf) { Ok(n) => n, Err(e) => { warn!("TUN read: {}", e); continue; } };
        if n == 0 { continue; }
        let pkt = &buf[..n];

        let enc_needed = base91codec::worst_case_len(pkt.len());
        let total_len = enc_needed + 20; // IPv4 header
        if cfg_tx.drop_if_exceeds && (total_len as u16) > cfg_tx.outer_mtu {
            warn!("drop packet len={} (encoded~{} + 20) exceeds outer_mtu {}",
                  pkt.len(), enc_needed, cfg_tx.outer_mtu);
            continue;
        }

        let enc = base91codec::encode(pkt);
        match raw.send_payload(&enc) {
            Ok(sent) => debug!("TX raw bytes: {} (from {} bytes)", sent, n),
            Err(e) => warn!("raw send failed: {}", e),
        }
    }
    Ok(())
});

// RX: RAW -> debase91 -> TUN
let running_rx = running.clone();
let cfg_rx = cfg.clone();
let mut rbuf = vec![0u8; 65535];
while running_rx.load(Ordering::SeqCst) {
    let (n, from) = match raw2.recv_into(&mut rbuf) { Ok(v) => v, Err(e) => { warn!("raw recv: {}", e); continue; } };
    if n < 20 { continue; }
    let frame = &rbuf[..n];

    if rawsock::ipv4::is_fragment(frame) { warn!("drop fragmented outer IPv4 from {} (n={})", from, n); continue; }
    if let Some(p) = rawsock::ipv4::protocol(frame) { if p != cfg_rx.ip_protocol { continue; } } else { continue; }
    let hlen = match rawsock::ipv4::header_len(frame) { Some(l) => l, None => continue };
    if hlen >= frame.len() { continue; }
    let payload = &frame[hlen..];

    let orig = base91codec::decode(payload);
    if orig.is_empty() { continue; }
    if let Err(e) = tun_writer.write_all(&orig) {
        warn!("TUN write: {}", e);
    } else {
        debug!("RX wrote {} bytes to TUN", orig.len());
    }
}


    info!("shutting down...");
    if let Err(e) = tx.join().unwrap_or(Ok(())) { warn!("TX thread exit: {}", e); }

    // post_down hooks
    for cmd in &cfg.post_down {
        info!("post_down: {}", cmd);
        if let Err(e) = run_shell(cmd) { warn!("post_down failed: {}", e); }
    }

    Ok(())
}

fn run_shell(cmd: &str) -> Result<()> {
    use std::process::Command;
    let status = Command::new("/bin/sh").arg("-c").arg(cmd).status()?;
    if !status.success() { bail!("command failed: {} -> {:?}", cmd, status); }
    Ok(())
}