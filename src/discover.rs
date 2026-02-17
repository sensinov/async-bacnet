use std::{net::SocketAddr, time::Duration};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        services::who_is::WhoIs,
        unconfirmed::UnconfirmedRequest,
    },
    common::io::{Reader, Writer},
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{DestinationAddress, MessagePriority, NetworkMessage, NetworkPdu},
    },
};
use log::{debug, info};
use tokio::{
    sync::mpsc::{self, Receiver},
    time::timeout,
};

use crate::{error::Error, io::TokioUdpIo};

/// A BACnet device found during discovery.
#[derive(Debug, Copy, Clone)]
pub struct Device {
    /// The device identifier (instance number of the BACnet device object).
    pub id: u32,
    /// The device vendor identifier.
    pub vendor_id: u16,
    /// The IP address of the device.
    pub addr: SocketAddr,
}

/// Send a WHO-IS broadcast and return a channel that yields discovered devices.
///
/// The `addr` should be a broadcast address (e.g. `192.168.1.255:47808`).
/// Discovery runs for `duration` (default: 2 minutes) or until the channel is dropped.
pub async fn discover(
    addr: SocketAddr,
    duration: Option<Duration>,
) -> Result<Receiver<Result<Device, Error>>, Error> {
    let io = TokioUdpIo::new_broadcast(addr).await?;
    let socket = io.socket();

    let who_is = WhoIs {};
    let apdu = ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::WhoIs(who_is));
    let dst = Some(DestinationAddress::new(0xffff, None));
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, dst, false, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalBroadcastNpdu, Some(npdu));

    let mut buffer = vec![0u8; 1500];
    let mut writer = Writer::new(&mut buffer);
    data_link.encode(&mut writer);

    let buf = writer.to_bytes();
    let send_timeout = Duration::from_secs(5);
    match timeout(send_timeout, socket.send_to(buf, addr)).await {
        Ok(result) => result,
        Err(err) => Err(std::io::Error::from(err)),
    }?;
    debug!("Sent WHO-IS to {}", addr);

    let who_is_duration = duration.unwrap_or(Duration::from_secs(120));
    let (sender, receiver) = mpsc::channel(1000);

    // Move io ownership into the spawned task
    tokio::spawn(async move {
        let socket = io.socket();
        let mut buf = vec![0u8; 1500];
        loop {
            let result = match timeout(who_is_duration, socket.recv_from(&mut buf)).await {
                Ok(result) => result,
                Err(_) => {
                    info!("Discovery finished");
                    break;
                }
            };
            let (n, peer) = match result {
                Ok(data) => data,
                Err(err) => {
                    let _ = sender.send(Err(err.into())).await;
                    continue;
                }
            };
            let payload = &buf[..n];
            debug!("Received: {:02x?} from {:?}", payload, peer);

            let mut reader = Reader::default();
            let message = match DataLink::decode(&mut reader, payload) {
                Ok(m) => m,
                Err(err) => {
                    let _ = sender.send(Err(Error::Bacnet(err.into()))).await;
                    continue;
                }
            };

            // Extract IAm from DataLink via pattern matching
            let iam = message
                .npdu
                .and_then(|npdu| match npdu.network_message {
                    NetworkMessage::Apdu(ApplicationPdu::UnconfirmedRequest(
                        UnconfirmedRequest::IAm(iam),
                    )) => Some(iam),
                    _ => None,
                });

            match iam {
                Some(iam) => {
                    let device = Device {
                        id: iam.device_id.id,
                        vendor_id: iam.vendor_id,
                        addr: peer,
                    };
                    if sender.send(Ok(device)).await.is_err() {
                        break; // receiver dropped
                    }
                }
                None => continue, // skip non-IAm packets
            }
        }
    });

    Ok(receiver)
}
