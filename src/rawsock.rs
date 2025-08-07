use anyhow::{Context, Result};
use socket2::{Domain, Protocol, Socket, Type};
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddrV4};

pub struct RawSock {
    sock: Socket,
    remote: SocketAddrV4,
    proto: u8,
}

impl RawSock {
    pub fn open(proto: u8, bind: Option<Ipv4Addr>, remote: Ipv4Addr) -> Result<Self> {
        // Type::RAW 存在于 socket2 的 Type 中
        let sock = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::from(proto as i32)))
            .context("create raw socket")?;

        if let Some(b) = bind {
            let local = SocketAddrV4::new(b, 0);
            sock.bind(&local.into()).context("bind raw socket")?;
        }

        sock.set_nonblocking(false)?;
        Ok(Self { sock, remote: SocketAddrV4::new(remote, 0), proto: proto })
    }

    pub fn send_payload(&self, payload: &[u8]) -> Result<usize> {
        self.sock.send_to(payload, &self.remote.into()).map_err(anyhow::Error::from)
    }

    pub fn recv_into(&self, buf: &mut [u8]) -> Result<(usize, std::net::SocketAddr)> {
        let uninit: &mut [MaybeUninit<u8>] = unsafe {
            std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut MaybeUninit<u8>, buf.len())
        };
        let (n, addr) = self.sock.recv_from(uninit).map_err(anyhow::Error::from)?;
        let from = addr.as_socket().ok_or_else(|| anyhow::anyhow!("non-inet addr"))?;
        Ok((n, from))
    }

    pub fn try_clone(&self) -> Result<Self> {
        let duped = self
            .sock
            .try_clone()
            .context("clone raw socket")?;
        Ok(RawSock {
            sock: duped,
            remote: self.remote,
            proto: self.proto,
        })
    }
}


/// IPv4 头部辅助
pub mod ipv4 {
    #[inline]
    pub fn header_len(pkt: &[u8]) -> Option<usize> {
        if pkt.len() < 20 { return None; }
        let ihl_words = pkt[0] & 0x0F;
        let len = (ihl_words as usize) * 4;
        if len < 20 || len > pkt.len() { return None; }
        Some(len)
    }
    #[inline]
    pub fn protocol(pkt: &[u8]) -> Option<u8> {
        if pkt.len() < 10 { return None; }
        Some(pkt[9])
    }
    #[inline]
    pub fn is_fragment(pkt: &[u8]) -> bool {
        if pkt.len() < 8 { return false; }
        let off = u16::from_be_bytes([pkt[6], pkt[7]]) & 0x1FFF;
        let mf = (pkt[6] & 0x20) != 0;
        mf || off != 0
    }
}
