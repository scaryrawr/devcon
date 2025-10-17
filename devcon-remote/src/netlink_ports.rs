use anyhow::{Context, Result};
use netlink_packet_core::{NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_sock_diag::{
    inet::{ExtensionFlags, InetRequest, SocketId, StateFlags},
    SockDiagMessage,
};
use netlink_sys::{protocols::NETLINK_SOCK_DIAG, Socket, SocketAddr};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
}

/// Get all listening ports using netlink sock_diag
pub async fn get_listening_ports() -> Result<Vec<PortInfo>> {
    let mut ports = HashSet::new();

    // Get TCP ports
    if let Ok(tcp_ports) = get_tcp_listening_ports().await {
        ports.extend(tcp_ports);
    }

    // Get UDP ports
    if let Ok(udp_ports) = get_udp_listening_ports().await {
        ports.extend(udp_ports);
    }

    let mut result: Vec<PortInfo> = ports.into_iter().collect();
    result.sort_by_key(|p| p.port);

    Ok(result)
}

async fn get_tcp_listening_ports() -> Result<Vec<PortInfo>> {
    get_listening_ports_by_protocol(libc::IPPROTO_TCP as u8, String::from("tcp")).await
}

async fn get_udp_listening_ports() -> Result<Vec<PortInfo>> {
    get_listening_ports_by_protocol(libc::IPPROTO_UDP as u8, String::from("udp")).await
}

async fn get_listening_ports_by_protocol(
    protocol: u8,
    protocol_name: String,
) -> Result<Vec<PortInfo>> {
    tokio::task::spawn_blocking(move || {
        let mut socket =
            Socket::new(NETLINK_SOCK_DIAG).context("Failed to create netlink socket")?;

        let addr = SocketAddr::new(0, 0);
        socket.bind(&addr).context("Failed to bind socket")?;

        // Create a request to dump all sockets in LISTEN state
        let mut packet = NetlinkMessage::from(SockDiagMessage::InetRequest(InetRequest {
            family: libc::AF_INET as u8,
            protocol,
            extensions: ExtensionFlags::empty(),
            states: StateFlags::LISTEN,
            socket_id: SocketId::new_v4(),
        }));

        packet.header.flags = NLM_F_REQUEST | NLM_F_DUMP;
        packet.header.sequence_number = 1;
        packet.finalize();

        let mut buf = vec![0; packet.header.length as usize];
        packet.serialize(&mut buf[..]);

        socket
            .send(&buf, 0)
            .context("Failed to send netlink request")?;

        let mut ports = HashSet::new();
        let mut receive_buf = vec![0; 4096];

        loop {
            let n = socket.recv(&mut receive_buf, 0)?;
            if n == 0 {
                break;
            }

            let mut offset = 0;
            loop {
                if offset >= n {
                    break;
                }

                let bytes = &receive_buf[offset..];
                let msg: NetlinkMessage<SockDiagMessage> =
                    NetlinkMessage::deserialize(bytes).context("Failed to deserialize message")?;

                match msg.payload {
                    NetlinkPayload::InnerMessage(SockDiagMessage::InetResponse(response)) => {
                        let port = response.header.socket_id.source_port;
                        if port > 0 {
                            ports.insert(PortInfo {
                                port,
                                protocol: protocol_name.clone(),
                            });
                        }
                    }
                    NetlinkPayload::Done(_) => {
                        return Ok(ports.into_iter().collect());
                    }
                    NetlinkPayload::Error(err) => {
                        anyhow::bail!("Netlink error: {:?}", err);
                    }
                    _ => {}
                }

                offset += msg.header.length as usize;
            }
        }

        Ok(ports.into_iter().collect())
    })
    .await
    .context("Task join error")?
}
