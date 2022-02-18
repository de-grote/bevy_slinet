//! TCP [`Protocol`] implementation based on [`std::net`]. You can enable it by adding `tcp` feature.

use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::ToSocketAddrs;

use crate::protocol::{
    ClientStream, Listener, NetworkStream, Protocol, ReadStream, ServerStream, WriteStream,
};

/// TCP protocol.
pub struct TcpProtocol;

#[async_trait]
impl Protocol for TcpProtocol {
    type Listener = TcpNetworkListener;
    type ServerStream = TcpNetworkStream;
    type ClientStream = TcpNetworkStream;

    async fn bind<A>(addr: A) -> io::Result<Self::Listener>
    where
        A: ToSocketAddrs + Send,
    {
        Ok(TcpNetworkListener(TcpListener::bind(addr).await?))
    }
}

/// A wrapped [TCP listener](std::net::TcpListener).
pub struct TcpNetworkListener(TcpListener);

#[async_trait]
impl Listener<TcpNetworkStream> for TcpNetworkListener {
    async fn accept(&self) -> io::Result<TcpNetworkStream> {
        let (stream, _) = self.0.accept().await?;
        Ok(TcpNetworkStream(stream))
    }

    fn address(&self) -> SocketAddr {
        self.0.local_addr().unwrap()
    }
}

/// A wrapped [TCP stream](std::net::TcpStream).
pub struct TcpNetworkStream(TcpStream);

#[async_trait]
impl NetworkStream for TcpNetworkStream {
    type ReadHalf = OwnedReadHalf;
    type WriteHalf = OwnedWriteHalf;

    async fn into_split(self) -> io::Result<(Self::ReadHalf, Self::WriteHalf)> {
        Ok(self.0.into_split())
    }

    fn peer_addr(&self) -> SocketAddr {
        self.0.peer_addr().unwrap()
    }

    fn local_addr(&self) -> SocketAddr {
        self.0.local_addr().unwrap()
    }
}

#[async_trait]
impl ReadStream for OwnedReadHalf {
    async fn read_exact(&mut self, buffer: &mut [u8]) -> io::Result<()> {
        AsyncReadExt::read_exact(self, buffer).await.map(|_| ())
    }
}

#[async_trait]
impl WriteStream for OwnedWriteHalf {
    async fn write_all(&mut self, buffer: &[u8]) -> io::Result<()> {
        AsyncWriteExt::write_all(self, buffer).await
    }
}

#[async_trait]
impl ClientStream for TcpNetworkStream {
    async fn connect<A>(addr: A) -> io::Result<Self>
    where
        Self: Sized,
        A: ToSocketAddrs + Send,
    {
        Ok(TcpNetworkStream(TcpStream::connect(addr).await?))
    }
}

impl ServerStream for TcpNetworkStream {}
