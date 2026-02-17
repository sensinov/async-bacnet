use std::net::SocketAddr;

use embedded_bacnet::{
    application_protocol::services::{
        i_am::IAm,
        read_property::{ReadProperty, ReadPropertyAck},
        read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleAck},
        write_property::WriteProperty,
    },
    simple::Bacnet,
};

use crate::{error::Error, io::TokioUdpIo};

const BUF_SIZE: usize = 1500;

/// Async BACnet client wrapping `embedded_bacnet::simple::Bacnet<TokioUdpIo>`.
///
/// Manages an internal buffer so callers don't need to provide one.
/// With the `alloc` feature enabled in embedded-bacnet, returned types are
/// fully owned and don't borrow from the buffer.
pub struct Client {
    inner: Bacnet<TokioUdpIo>,
    buf: Vec<u8>,
}

impl Client {
    /// Create a new client connected to the given BACnet device address.
    pub async fn new(peer: SocketAddr) -> Result<Self, Error> {
        let io = TokioUdpIo::new(peer).await?;
        Ok(Self {
            inner: Bacnet::new(io),
            buf: vec![0u8; BUF_SIZE],
        })
    }

    /// Access the inner `Bacnet<TokioUdpIo>` for advanced operations.
    pub fn inner(&mut self) -> &mut Bacnet<TokioUdpIo> {
        &mut self.inner
    }

    /// Access the internal buffer for advanced operations.
    pub fn buffer(&mut self) -> &mut [u8] {
        &mut self.buf
    }

    /// Read a single property from a BACnet object.
    pub async fn read_property(&mut self, request: ReadProperty) -> Result<ReadPropertyAck<'_>, Error> {
        let ack = self.inner.read_property(&mut self.buf, request).await?;
        Ok(ack)
    }

    /// Read multiple properties from multiple BACnet objects.
    pub async fn read_property_multiple(
        &mut self,
        request: ReadPropertyMultiple<'_>,
    ) -> Result<ReadPropertyMultipleAck<'_>, Error> {
        let ack = self
            .inner
            .read_property_multiple(&mut self.buf, request)
            .await?;
        Ok(ack)
    }

    /// Write a property value to a BACnet object.
    pub async fn write_property(&mut self, request: WriteProperty<'_>) -> Result<(), Error> {
        self.inner.write_property(&mut self.buf, request).await?;
        Ok(())
    }

    /// Send a WHO-IS request and return the first I-Am response, if any.
    pub async fn who_is(&mut self) -> Result<Option<IAm>, Error> {
        let iam = self.inner.who_is(&mut self.buf).await?;
        Ok(iam)
    }
}
