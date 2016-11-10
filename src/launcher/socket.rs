use std::io;
use std::net::{TcpListener, SocketAddr, Ipv4Addr, ToSocketAddrs};
use std::net::{SocketAddrV4};

use net2::{TcpBuilder};


pub fn parse_and_bind(val: &str) -> io::Result<TcpListener> {
    let addr = if let Ok(port) = val.parse() {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
    } else {
        let mut parts = val.rsplitn(2, ':');
        let port = match parts.next().and_then(|x| x.parse().ok()) {
            Some(x) => x,
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Can't parse port")),
        };
        let host_str = parts.next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                "Address should be just `port`, or `host:port`"))?;
        if host_str == "*" {
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))
        } else if let Ok(addr) = host_str.parse() {
            SocketAddr::new(addr, port)
        } else {
            // TODO(tailhook) use lookup_host
            (host_str, port).to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                "Address could not be resolved"))?
        }
    };
    let bld = match addr {
        SocketAddr::V4(..) => TcpBuilder::new_v4()?,
        SocketAddr::V6(..) => TcpBuilder::new_v6()?,
    };
    bld.reuse_address(true)?;
    bld.bind(addr)?;
    bld.listen(128) // pretty standard value
}
