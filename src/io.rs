use std::{fmt::Debug, net::SocketAddr, time::Duration};

use embedded_bacnet::simple::NetworkIo;
use tokio::{net::UdpSocket, time::timeout};

/// A tokio-based UDP I/O implementation for `embedded_bacnet::simple::Bacnet<T>`.
pub struct TokioUdpIo {
    socket: UdpSocket,
    peer: SocketAddr,
    timeout: Duration,
}

impl Debug for TokioUdpIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioUdpIo")
            .field("local_addr", &self.socket.local_addr().ok())
            .field("peer", &self.peer)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl TokioUdpIo {
    pub async fn new(peer: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        Ok(Self {
            socket,
            peer,
            timeout: Duration::from_secs(5),
        })
    }

    pub async fn new_broadcast(peer: SocketAddr) -> Result<Self, std::io::Error> {
        use socket2::{Domain, Socket, Type};

        let domain = match peer {
            SocketAddr::V4(_) => Domain::IPV4,
            SocketAddr::V6(_) => Domain::IPV6,
        };
        let socket = Socket::new(domain, Type::DGRAM, None)?;
        socket.set_nonblocking(true)?;
        socket.set_reuse_address(true)?;
        #[cfg(not(target_os = "windows"))]
        socket.set_reuse_port(true)?;

        let local_addr: SocketAddr = format!("0.0.0.0:{}", peer.port()).parse().unwrap();
        socket.bind(&local_addr.into())?;

        let socket = UdpSocket::from_std(socket.into())?;
        socket.set_broadcast(true)?;

        Ok(Self {
            socket,
            peer,
            timeout: Duration::from_secs(5),
        })
    }

    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }

    pub fn peer(&self) -> SocketAddr {
        self.peer
    }

    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout = duration;
    }
}

impl NetworkIo for TokioUdpIo {
    type Error = std::io::Error;

    async fn read(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let result = timeout(self.timeout, self.socket.recv_from(buf)).await;
        match result {
            Ok(Ok((n, _peer))) => Ok(n),
            Ok(Err(e)) => Err(e),
            Err(_elapsed) => Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "read timed out",
            )),
        }
    }

    async fn write(&self, buf: &[u8]) -> Result<usize, Self::Error> {
        let result = timeout(self.timeout, self.socket.send_to(buf, self.peer)).await;
        match result {
            Ok(Ok(n)) => Ok(n),
            Ok(Err(e)) => Err(e),
            Err(_elapsed) => Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "write timed out",
            )),
        }
    }
}
